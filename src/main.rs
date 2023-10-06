use rdkafka::producer::{FutureProducer, FutureRecord};
use rdkafka::ClientConfig;
use tokio::io::AsyncReadExt;
use std::time::Duration;
use uuid::Uuid;
use clap::{App, Arg};
use std::sync::Arc;

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
        Arg::with_name("LISTENER_TYPE")
            .short("t")
            .long("type")
            .value_name("LISTENER_TYPE")
            .help("Type of listener: tcp, udp, or http. Default is tcp.")
            .takes_value(true),
    );


    let matches = app.get_matches();
       
    let kafka_broker = matches.value_of("KAFKA_BROKER").unwrap_or("localhost:9092").to_string();
    let kafka_topic = Arc::new(matches.value_of("KAFKA_TOPIC").unwrap_or("gelf_messages").to_string());
    let gelf_listen = Arc::new(matches.value_of("GELF_LISTEN_ADDR").unwrap_or("0.0.0.0:12201").to_string());
    let verbose = matches.is_present("VERBOSE");
    let listener_type = matches.value_of("LISTENER_TYPE").unwrap_or("tcp");

    // let cloned_kafka_topic = kafka_topic.as_str().clone(); // Pour le spawn
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
                let cloned_kafka_topic = kafka_topic.clone();
        
                tokio::spawn(async move {
                    let mut buffer = vec![0u8; 65536];
                    loop {
                        match socket.read(&mut buffer).await {
                            Ok(0) => return, // Connexion fermée
                            Ok(n) => {
                                let key = Uuid::new_v4().to_string();
                                let record = FutureRecord::to(cloned_kafka_topic.as_str())
                                    .key(&key)
                                    .payload(&buffer[..n]);
                                let _ = producer_clone.send(record, Duration::from_secs(5)).await;
                                if verbose {
                                    println!("Message envoyé sur Kafka: {}", key);
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
                let cloned_kafka_topic = kafka_topic.clone();
        
                tokio::spawn(async move {
                    let key = Uuid::new_v4().to_string();
                    let record = FutureRecord::to(cloned_kafka_topic.as_str())
                        .key(&key)
                        .payload(&buffer[..size]);
                    let _ = producer_clone.send(record, Duration::from_secs(5)).await;
                    if verbose {
                        println!("Message envoyé sur Kafka via UDP: {}", key);
                    }
                });
            }
        },
        "http" => {
            use warp::Filter;

            // Define a warp filter that handles POST requests to "/"
            let gelf = warp::post()
                .and(warp::path::end())
                .and(warp::body::bytes())
                .map(move |data: bytes::Bytes| {

                    // Here, data contains the body of the POST request.
                    // You can send this data to Kafka as you do for the TCP listener.
        
                    let producer_clone = producer.clone();
                    let cloned_kafka_topic = kafka_topic.clone();
        
                    tokio::spawn(async move {
                        let key = Uuid::new_v4().to_string();
                        let binding = data.to_vec();
                        let record = FutureRecord::to(cloned_kafka_topic.as_str())
                            .key(&key)
                            .payload(&binding);
                        let _ = producer_clone.send(record, Duration::from_secs(5)).await;
                        if verbose {
                            println!("Message envoyé sur Kafka: {}", key);
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