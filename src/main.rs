use rdkafka::producer::{FutureProducer, FutureRecord};
use rdkafka::ClientConfig ;
use tokio::io::AsyncReadExt;
use std::time::Duration;
use uuid::Uuid;
use clap::{App, Arg};
use std::sync::Arc;
use daemonize::Daemonize;
mod gelf;
use gelf::decode_gelf_message;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let app = App::new("gelf-kafka-listener")
    .version("1.0")
    .author("Stephane Benoit")
    .about("Reçoit des messages GELF sur TCP et les envoie sur Kafka")
    .arg(
        Arg::with_name("KAFKA_BROKER")
            .short("b")
            .long("broker")
            .value_name("KAFKA_BROKER")
            .help("Adresse du broker Kafka, localhost:9092 par défaut")
            .takes_value(true),
    )
    .arg(
        Arg::with_name("KAFKA_TOPIC")
            .short("t")
            .long("topic")
            .value_name("KAFKA_TOPIC")
            .help("Nom du topic Kafka, wzgelf par défaut")
            .takes_value(true),
    )
    .arg(
        Arg::with_name("GELF_LISTEN_ADDR")
            .short("l")
            .long("listen")
            .value_name("GELF_LISTEN_ADDR")
            .help("Adresse et port d'écoute TCP pour les messages GELF, 0.0.0.0:12201 par défaut")
            .takes_value(true),
    )
    .arg(
        Arg::with_name("VERBOSE")
            .short("v")
            .long("verbose")
            .value_name("VERBOSE")
            .required(false)
            .takes_value(false)
            .help("Affiche les messages de log sur stdout"),
    )
    .arg(
        Arg::with_name("LISTENER_PROTO")
            .short("p")
            .long("protocol")
            .value_name("LISTENER_PROTO")
            .help("Protcol of the listener: tcp, udp, or http. Default is tcp.")
            .takes_value(true),
    )
    .arg(
        Arg::with_name("daemonize")
            .short("d")
            .long("daemonize")
            .help("Détache le programme et enregistre les logs dans /tmp/gelf-kafka-listener.out et /tmp/gelf-kafka-listener.err")
    )
    .arg(
        Arg::with_name("log-path-prefix")
            .short("L")
            .long("log-path-prefix")
            .value_name("LOG_PATH_PREFIX")
            .help("Préfixe du chemin pour les fichiers de logs (par défaut : /tmp)")
            .takes_value(true),
    );


    let matches = app.get_matches();
       
    let kafka_broker = matches.value_of("KAFKA_BROKER").unwrap_or("localhost:9092").to_string();
    let kafka_topic = Arc::new(matches.value_of("KAFKA_TOPIC").unwrap_or("gelf_messages").to_string());
    let gelf_listen = Arc::new(matches.value_of("GELF_LISTEN_ADDR").unwrap_or("0.0.0.0:12201").to_string());
    let verbose = matches.is_present("VERBOSE");
    let listener_type = matches.value_of("LISTENER_PROTO").unwrap_or("tcp");
    let log_path_prefix = matches.value_of("log-path-prefix").unwrap_or("/tmp");

    if matches.is_present("daemonize") {
        use std::fs::File;
        let log_filename = format!("gelf-kafka-listener_{}_{}_{}.out", listener_type, gelf_listen.replace(":", "-"), kafka_topic);
        let err_filename = format!("gelf-kafka-listener_{}_{}_{}.err", listener_type, gelf_listen.replace(":", "-"), kafka_topic);
        let pid_filename = format!("gelf-kafka-listener_{}_{}_{}.pid", listener_type, gelf_listen.replace(":", "-"), kafka_topic);
        let stdout_path = format!("{}/{}", log_path_prefix, log_filename);
        let stderr_path = format!("{}/{}", log_path_prefix, err_filename);
        let pid_file = format!("{}/{}", log_path_prefix, pid_filename);
        
        let stdout = File::create(stdout_path).unwrap();
        let stderr = File::create(stderr_path).unwrap();
    
        let daemonize = Daemonize::new()
            .pid_file(pid_file)                       // Emplacement du fichier PID
            .chown_pid_file(true)                    // Change le propriétaire du fichier PID
            .working_directory(log_path_prefix)       // Répertoire de travail
            .stdout(stdout)                          // Redirige stdout vers un fichier
            .stderr(stderr)                          // Redirige stderr vers un fichier
            .privileged_action(|| "Executed before drop privileges");
    
        match daemonize.start() {
            Ok(_) => println!("Démarrage réussi"),
            Err(e) => eprintln!("Erreur : {}", e),
        }
    }

    // Configuration Kafka
    let producer: FutureProducer = ClientConfig::new()
        .set("bootstrap.servers", kafka_broker)
        .create().expect("Producer creation error");

    match listener_type {
        "tcp" => {        
            // Écouteur TCP
            use tokio::net::TcpListener;

            let listener = TcpListener::bind(&*gelf_listen).await.expect("Failed to bind TCP listener");
            println!("Listening on: {}", gelf_listen);
            loop {
                let (mut socket, _) = listener.accept().await.expect("Failed to accept TCP connection");
                let producer_clone = producer.clone();
                let kafka_topic_clone = kafka_topic.clone();
        
                tokio::spawn(async move {
                    let mut buffer = vec![0u8; 65536];
                    loop {
                        match socket.read(&mut buffer).await {
                            Ok(0) => return, // Connexion fermée
                            Ok(n) => {
                                let key = Uuid::new_v4().to_string();
                                let mut gelf_message = decode_gelf_message(&buffer[..n]).unwrap();   
                                gelf_message.insert("gelf_kafka_listener_key".to_string(), serde_json::Value::String(key.clone()));
                                gelf_message.insert("source_ip".to_string(), serde_json::Value::String(socket.peer_addr().unwrap().to_string()));
                                let payload: Vec<u8> = serde_json::to_vec(&gelf_message).unwrap();
                                let record = FutureRecord::to(kafka_topic_clone.as_str())
                                    .key(&key)
                                    .payload(&payload);
                                let _ = producer_clone.send(record, Duration::from_secs(5)).await;
                                if verbose {
                                    println!("Message tcp envoyé sur Kafka: {}", key);
                                }
                            }
                            Err(_) => return,
                        }
                    }
                });
            }
        },
        "udp" => {
            use tokio::net::UdpSocket;

            let socket = UdpSocket::bind(&*gelf_listen).await.expect("Failed to bind UDP socket");
            println!("Listening on UDP: {}", gelf_listen);
        
            loop {
                let mut buffer = vec![0u8; 65536];
                let (size, _) = socket.recv_from(&mut buffer).await.expect("Failed to read from UDP socket");
        
                let producer_clone = producer.clone();
                let kafka_topic_clone = kafka_topic.clone();
                let peer_addr = socket.peer_addr().unwrap();
        
                tokio::spawn(async move {
                    let key = Uuid::new_v4().to_string();
                    let mut gelf_message = decode_gelf_message(&buffer[..size]).unwrap();   
                    gelf_message.insert("gelf_kafka_listener_key".to_string(), serde_json::Value::String(key.clone()));
                    gelf_message.insert("source_ip".to_string(), serde_json::Value::String(peer_addr.to_string()));
                    let payload: Vec<u8> = serde_json::to_vec(&gelf_message).unwrap();
                    let record = FutureRecord::to(kafka_topic_clone.as_str())
                        .key(&key)
                        .payload(&payload);
                    let _ = producer_clone.send(record, Duration::from_secs(5)).await;
                    if verbose {
                        println!("Message udp envoyé sur Kafka via UDP: {}", key);
                    }
                });
            }
        },
        "http" => {
            use warp::Filter;
            use std::net::SocketAddr;

            // Define a warp filter that handles POST requests to "/"
            let gelf = warp::post()
                .and(warp::path("gelf"))
                .and(warp::addr::remote()) 
                .and(warp::body::bytes())
                .map(move |peer_addr: Option<SocketAddr>, data: bytes::Bytes| {

                    // Here, data contains the body of the POST request.
                    // You can send this data to Kafka as you do for the TCP listener.
        
                    let producer_clone = producer.clone();
                    let kafka_topic_clone = kafka_topic.clone();
        
                    tokio::spawn(async move {
                        let key = Uuid::new_v4().to_string();
                        let mut gelf_message = decode_gelf_message(&(data.to_vec())).unwrap();   
                        gelf_message.insert("gelf_kafka_listener_key".to_string(), serde_json::Value::String(key.clone()));
                        gelf_message.insert("source_ip".to_string(), serde_json::Value::String(peer_addr.unwrap().to_string()));
                        let payload: Vec<u8> = serde_json::to_vec(&gelf_message).unwrap();
                        let record = FutureRecord::to(kafka_topic_clone.as_str())
                            .key(&key)
                            .payload(&payload);
                        let _ = producer_clone.send(record, Duration::from_secs(5)).await;
                        if verbose {
                            println!("Message http envoyé sur Kafka: {}", key);
                        }
                    });
        
                    warp::reply::with_status("Received", warp::http::StatusCode::OK)
                });
        
            let addr: std::net::SocketAddr = (&*gelf_listen).parse().expect("Failed to parse listening address");
            // Start the warp server on the specified address
            warp::serve(gelf).run(addr).await;
        },
        _ => {
            eprintln!("Unsupported listener type: {}", listener_type);
            std::process::exit(1);
        }
    }
    Ok(())
}