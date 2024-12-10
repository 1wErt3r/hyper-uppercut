use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use futures_util::{SinkExt, StreamExt};
use url::Url;
use serde_json::json;
use crate::nostr::Event;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Clone)]
pub struct RelayPool {
    relays: Arc<Vec<String>>,
    outbox: Arc<Mutex<Vec<Event>>>,
}

impl RelayPool {
    pub fn new(relay_urls: Vec<String>) -> Self {
        Self {
            relays: Arc::new(relay_urls),
            outbox: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub async fn add_to_outbox(&self, event: Event) {
        let mut outbox = self.outbox.lock().await;
        outbox.push(event);
    }

    pub async fn process_outbox(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut outbox = self.outbox.lock().await;
        println!("Processing outbox with {} events", outbox.len());
        
        for event in outbox.iter() {
            for relay_url in self.relays.iter() {
                println!("Attempting to publish to relay: {}", relay_url);
                match self.publish_to_relay(relay_url, event.clone()).await {
                    Ok(_) => println!("Successfully published to {}", relay_url),
                    Err(e) => eprintln!("Failed to publish to {}: {}", relay_url, e),
                }
            }
        }

        outbox.clear();
        Ok(())
    }

    async fn publish_to_relay(&self, relay_url: &str, event: Event) -> Result<(), Box<dyn std::error::Error>> {
        println!("Connecting to relay: {}", relay_url);
        let url = Url::parse(relay_url)?;
        let (mut ws_stream, _) = connect_async(url).await?;
        println!("Connected to relay, sending event");

        let message = json!(["EVENT", event]);
        ws_stream.send(Message::Text(message.to_string())).await?;
        println!("Event sent, waiting for response");

        if let Some(msg) = ws_stream.next().await {
            println!("Received response from relay");
            match msg? {
                Message::Text(text) => {
                    let response: serde_json::Value = serde_json::from_str(&text)?;
                    if let Some(array) = response.as_array() {
                        if array.len() >= 3 {
                            if array[0].as_str() == Some("OK") {
                                let success = array[2].as_bool().unwrap_or(false);
                                let message = array[3].as_str().unwrap_or("");
                                if !success {
                                    return Err(format!("Relay rejected event: {}", message).into());
                                }
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        Ok(())
    }
}