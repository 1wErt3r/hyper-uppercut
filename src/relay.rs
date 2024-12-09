use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use futures_util::{SinkExt, StreamExt};  // Add back StreamExt
use url::Url;
use serde_json::json;
use crate::nostr::Event;

pub struct RelayClient {
    url: String,
}

impl RelayClient {
    pub fn new(url: String) -> Self {
        Self { url }
    }

    pub async fn publish_event(&self, event: Event) -> Result<(), Box<dyn std::error::Error>> {
        let url = Url::parse(&self.url)?;
        let (mut ws_stream, _) = connect_async(url).await?;

        let message = json!(["EVENT", event]);
        ws_stream.send(Message::Text(message.to_string())).await?;

        // Wait for and process the OK message
        if let Some(msg) = ws_stream.next().await {
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