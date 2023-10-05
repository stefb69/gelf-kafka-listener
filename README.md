---
# gelf-kafka-listener

`gelf-kafka-listener` is a service that listens for GELF messages over TCP and forwards them to Kafka.

## Description
This program is designed to receive GELF messages on a specific TCP port and relay them to a Kafka topic. It's written in Rust and utilizes `tokio` for asynchronous handling and `rdkafka` to interface with Kafka.

## Prerequisites
- Rust (recommended version: 1.5x or higher)
- An accessible Kafka broker

## Installation
1. Clone the repository:

```bash

git clone https://github.com/stefb69/gelf-kafka-listener.git
cd gelf-kafka-listener
```

2. Build the program:

```bash

cargo build --release
```
The executable will be available under `target/release/gelf-kafka-listener`.

## Usage

```bash

./gelf-kafka-listener -b <kafka_broker_address> -t <kafka_topic_name> -l <gelf_listen_address> [-v]
```



Options: 
- `-b` or `--broker`: Address of the Kafka broker (default: `localhost:9092`). 
- `-t` or `--topic`: Name of the Kafka topic to send GELF messages to (default: `wzgelf`). 
- `-l` or `--listen`: TCP listening address for GELF messages (default: `0.0.0.0:12201`). 
- `-v` or `--verbose`: Enables verbose logging. This will display log messages on stdout.

## Contributing

Contributions are welcome! Please open an issue or submit a pull request on GitHub.
## License

This project is licensed under the MIT License. See the `LICENSE` file for more details.---