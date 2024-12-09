# nostrsss

A Rust application that bridges RSS feeds to the Nostr network by automatically publishing new RSS items as Nostr events.

## Overview

nostrsss monitors an RSS feed and publishes new items to a Nostr relay as text notes (kind 1 events). Each post includes the item's title and link.

## Features

- Monitors RSS feeds for new content
- Automatically publishes new items to Nostr relays
- Configurable through environment variables
- Implements Nostr NIP-01 protocol for event creation and relay communication

## Installation

1. Make sure you have Rust and Cargo installed
2. Clone this repository
3. Build the project:
```bash
cargo build --release
```

## Configuration

The following environment variables must be set:

- `NOSTRSSS_SECRET_KEY`: Your Nostr private key (hex format)
- `NOSTRSSS_FEED_URL`: The URL of the RSS feed to monitor
- `NOSTRSSS_RELAY_URL`: The WebSocket URL of the Nostr relay to publish to

## Usage

After setting the environment variables, run:

```bash
cargo run --release
```

The application will:
1. Connect to the specified RSS feed
2. Check for new items every ~2.8 hours (10000 seconds)
3. Publish new items to the configured Nostr relay
4. Print status messages about successful publications or errors

## Technical Details

### Dependencies

The project uses several key dependencies (see Cargo.toml):

```6:17:nostrsss/Cargo.toml
[dependencies]
tokio = { version = "1.28", features = ["full"] }
reqwest = { version = "0.11", features = ["json"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
rss = "2.0"
secp256k1 = { version = "0.27", features = ["rand", "rand-std"] }
hex = "0.4"
sha2 = "0.10"
tokio-tungstenite = { version = "0.19", features = ["native-tls"] }
futures-util = "0.3"
url = "2.4"
```


### Architecture

The project is split into three main modules:

1. **RSS Module**: Handles fetching and parsing RSS feeds
2. **Nostr Module**: Implements Nostr event creation and signing according to NIP-01
3. **Relay Module**: Manages WebSocket connections and communication with Nostr relays

### Event Format

Each RSS item is published as a Nostr event with:
- Kind: 1 (text note)
- Content: Item title followed by item link
- Signature: Schnorr signature using secp256k1

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is open source. Please add appropriate license information.

## Security Notes

- Keep your `NOSTRSSS_SECRET_KEY` secure and never share it
- Consider running the application in a secure environment
- Review the RSS feed source as it will be automatically published to your Nostr account
