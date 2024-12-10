# hyper-uppercut

A Rust application that bridges RSS feeds to the Nostr network by automatically publishing summarized feed content as Nostr events.

## Overview

hyper-uppercut monitors RSS feeds defined in an OPML file and publishes summarized content to Nostr relays as text notes (kind 1 events). Each post includes a summary of recent feed items along with their source links.

## Features

- Parses OPML files to extract RSS/Atom feed URLs
- Uses Ollama AI to generate concise summaries of feed content
- Automatically publishes summaries to multiple Nostr relays
- Supports both local and remote OPML files
- Implements Nostr NIP-01, NIP-05, NIP-10, and NIP-40 protocols
- Configurable through environment variables
- Optional Lightning address support for zaps

## Installation

1. Make sure you have Rust and Cargo installed
2. Install Ollama (https://ollama.ai) and pull the qwen2.5 model:
```bash
ollama pull qwen2.5
```
3. Build the application:
```bash
cargo build --release
```

## Configuration

The following environment variables can be set:

Required:
- `HYPER_UPPERCUT_SECRET_KEY`: Your Nostr private key (hex format)
- `HYPER_UPPERCUT_OPML_SOURCE`: URL or local path to your OPML file
- `HYPER_UPPERCUT_RELAY_URLS`: Comma-separated list of Nostr relay WebSocket URLs
- `HYPER_UPPERCUT_OLLAMA_URL`: URL of your Ollama instance (e.g., "http://localhost")

Optional:
- `HYPER_UPPERCUT_FEED_CHECK_SECONDS`: Time between OPML checks (default: 10000)
- `HYPER_UPPERCUT_NOTE_DELAY_SECONDS`: Time between publishing notes (default: 60)
- `HYPER_UPPERCUT_PROFILE_NAME`: Name to show in your profile (default: "RSS Bot")
- `HYPER_UPPERCUT_NIP05`: Your NIP-05 identifier (default: none)
- `HYPER_UPPERCUT_LIGHTNING_ADDRESS`: Your Lightning address for receiving zaps (default: none)

## License

GNU Affero General Public License v3.0

## Security Notes

- Keep your `HYPER_UPPERCUT_SECRET_KEY` secure and never share it
- Consider running Ollama locally rather than exposing it to the internet
