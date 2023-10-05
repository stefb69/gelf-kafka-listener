use rdkafka::producer::{FutureProducer, FutureRecord};
use rdkafka::ClientConfig;
use tokio::net::TcpListener;
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
    );


    let matches = app.get_matches();
       
    let kafka_broker = matches.value_of("KAFKA_BROKER").unwrap_or("localhost:9092").to_string();
    let kafka_topic = Arc::new(matches.value_of("KAFKA_TOPIC").unwrap_or("wzgelf").to_string());
    let gelf_listen = matches.value_of("GELF_LISTEN_ADDR").unwrap_or("0.0.0.0:12201").to_string();
    let verbose = matches.is_present("VERBOSE");

    // let cloned_kafka_topic = kafka_topic.as_str().clone(); // Pour le spawn
    // Configuration Kafka
    let producer: FutureProducer = ClientConfig::new()
        .set("bootstrap.servers", kafka_broker)
        .create().expect("Producer creation error");

    // Écouteur TCP
    let listener = TcpListener::bind(gelf_listen).await?;
    loop {
        let (mut socket, _) = listener.accept().await?;
        let producer_clone = producer.clone();
        let cloned_kafka_topic = kafka_topic.clone();
  
        tokio::spawn(async move {
            let mut buffer = vec![0u8; 4096];
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
}
