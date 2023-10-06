---
# gelf-kafka-listener

This Rust application listens for Graylog Extended Log Format (GELF) messages over TCP, HTTP, or UDP and forwards them to a Kafka topic.

## Description
This program is designed to receive GELF messages on a specific TCP/UDP/HTTP port and relay them to a Kafka topic. It's written in Rust and utilizes `tokio` for asynchronous handling and `rdkafka` to interface with Kafka.

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

gelf-kafka-listener [FLAGS] [OPTIONS]
```

### Flags: 
- `-v, --verbose`: Outputs log messages to stdout.
- `-d, --daemonize`: Detaches the program to run in the background and logs messages to files.

### Options: 
- `-b, --broker <KAFKA_BROKER>`: Address of the Kafka broker (default: `localhost:9092`). 
- `-t, --topic <KAFKA_TOPIC>`: Name of the Kafka topic to send messages to (default: `gelf_messages`). 
- `-l, --listen <GELF_LISTEN_ADDR>`: Address and port to listen for GELF messages (default: `0.0.0.0:12201`). 
- `-p, --protocol <LISTENER_PROTO>`: Protocol for listening: `tcp`, `http`, or `udp`. Default is `tcp`. 
- `-L, --log-path-prefix <LOG_PATH_PREFIX>`: Prefix for the log file paths (default: `/tmp`). Log filenames will be formatted as `gelf-kafka-listener_<LISTENER_PROTO>_<GELF_LISTEN_ADDR>_<KAFKA_TOPIC>.out` and `.err`.

### Examples: 
1. Start the listener with default settings:

```bash

gelf-kafka-listener
``` 
2. Start the listener with a specific Kafka broker and topic, listening for GELF messages over UDP, and log messages to /var/log::

```bash

gelf-kafka-listener -b my.kafka.broker:9092 -t my_topic -p udp -L /var/log -d
``` 
3. Start the listener in verbose mode, listening for GELF messages over HTTP:

```bash

gelf-kafka-listener -v -p http
```
### Notes: 
- When using the UDP listener, ensure that the GELF messages being sent fit within the maximum datagram size (default buffer size is set to `65536` bytes). 
- For the HTTP listener, the application expects GELF messages to be sent as the body of a POST request to the root path (`/`).

---
## Contributing

Contributions are welcome! Please open an issue or submit a pull request on GitHub.
## License

This project is licensed under the MIT License. See the `LICENSE` file for more details.