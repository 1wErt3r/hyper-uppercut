mod nostr;
mod rss;
mod relay;

use secp256k1::SecretKey;
use tokio;
use std::str::FromStr;
use relay::RelayClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let secret_key_str = std::env::var("NOSTRSSS_SECRET_KEY")?;
    let secret_key = SecretKey::from_str(&secret_key_str)?;

    let feed_url = "http://www.xinhuanet.com/english/rss/oddrss.xml";
    let feed_reader = rss::FeedReader::new(feed_url.to_string());
    
    let relay_url = "wss://relay.damus.io";
    let relay_client = RelayClient::new(relay_url.to_string());
    
    loop {
        let items = feed_reader.fetch_latest().await?;
        
        for item in items {
            let content = format!(
                "{}\n\n{}",
                item.title().unwrap_or("No title"),
                item.link().unwrap_or("No link")
            );

            let event = nostr::Event::new(
                &secret_key,
                content,
                1, // kind 1 = text note
                vec![] // no tags for now
            );

            match relay_client.publish_event(event).await {
                Ok(_) => println!("Successfully published event to relay"),
                Err(e) => eprintln!("Failed to publish event: {}", e),
            }
        }

        tokio::time::sleep(tokio::time::Duration::from_secs(10000)).await;
    }
} 