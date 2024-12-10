mod nostr;
mod rss;
mod relay;
mod summarize;

use secp256k1::SecretKey;
use tokio;
use std::str::FromStr;
use relay::RelayClient;
use serde_json::json;
use summarize::Summarizer;
use dotenv::dotenv;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load environment variables from .env file
    dotenv().ok();

    // Required environment variables
    let secret_key_str = std::env::var("HYPER_UPPERCUT_SECRET_KEY")
        .map_err(|_| "HYPER_UPPERCUT_SECRET_KEY environment variable not set")?;
    let secret_key = SecretKey::from_str(&secret_key_str)?;

    let feed_url = std::env::var("HYPER_UPPERCUT_FEED_URL")
        .map_err(|_| "HYPER_UPPERCUT_FEED_URL environment variable not set")?;
    let feed_reader = rss::FeedReader::new(feed_url);
    
    let relay_url = std::env::var("HYPER_UPPERCUT_RELAY_URL")
        .map_err(|_| "HYPER_UPPERCUT_RELAY_URL environment variable not set")?;
    let relay_client = RelayClient::new(relay_url);

    // Optional environment variables with defaults
    let feed_check_seconds = std::env::var("HYPER_UPPERCUT_FEED_CHECK_SECONDS")
        .unwrap_or_else(|_| "10000".to_string())
        .parse::<u64>()?;

    let note_delay_seconds = std::env::var("HYPER_UPPERCUT_NOTE_DELAY_SECONDS")
        .unwrap_or_else(|_| "60".to_string())
        .parse::<u64>()?;

    // Profile metadata
    let profile_name = std::env::var("HYPER_UPPERCUT_PROFILE_NAME")
        .unwrap_or_else(|_| "RSS Bot".to_string());
    let profile_about = std::env::var("HYPER_UPPERCUT_PROFILE_ABOUT")
        .unwrap_or_else(|_| "I post RSS feed updates to nostr".to_string());
    let profile_picture = std::env::var("HYPER_UPPERCUT_PROFILE_PICTURE")
        .unwrap_or_else(|_| "".to_string());

    // Publish profile metadata (kind 0 event)
    let profile = json!({
        "name": profile_name,
        "about": profile_about,
        "picture": profile_picture,
        "nip05": std::env::var("HYPER_UPPERCUT_NIP05").unwrap_or_else(|_| "".to_string())
    }).to_string();

    let profile_event = nostr::Event::new(
        &secret_key,
        profile,
        0,  // kind 0 for metadata
        vec![]
    );

    match relay_client.publish_event(profile_event).await {
        Ok(_) => println!("Successfully published profile metadata"),
        Err(e) => eprintln!("Failed to publish profile: {}", e),
    }

    println!("Starting RSS feed monitoring...");
    
    let summarizer = if let Ok(ollama_url) = std::env::var("HYPER_UPPERCUT_OLLAMA_URL") {
        Some(Summarizer::new(ollama_url))
    } else {
        None
    };

    loop {
        let items = feed_reader.fetch_latest().await?;
        println!("Fetched {} items from feed", items.len());
        
        let content = if let Some(summarizer) = &summarizer {
            match summarizer.summarize_feed(&items).await {
                Ok(summary) => {
                    let links = items.iter()
                        .filter_map(|item| item.link())
                        .collect::<Vec<_>>()
                        .join("\n");
                    
                    format!(
                        "Feed Summary:\n\n{}\n\nSources:\n{}",
                        summary,
                        links
                    )
                },
                Err(e) => {
                    eprintln!("Failed to summarize content: {}", e);
                    format!(
                        "Latest items:\n\n{}",
                        items.iter()
                            .filter_map(|item| {
                                Some(format!("{}\n{}", 
                                    item.title().unwrap_or("No title"),
                                    item.link().unwrap_or("No link")))
                            })
                            .collect::<Vec<_>>()
                            .join("\n\n")
                    )
                }
            }
        } else {
            format!(
                "Latest items:\n\n{}",
                items.iter()
                    .filter_map(|item| {
                        Some(format!("{}\n{}", 
                            item.title().unwrap_or("No title"),
                            item.link().unwrap_or("No link")))
                    })
                    .collect::<Vec<_>>()
                    .join("\n\n")
            )
        };

        // Create a single event for the entire feed update
        let mut tags = vec![
            vec!["client".to_string(), "hyper-uppercut".to_string()],
            vec!["alt".to_string(), "RSS Feed Summary".to_string()]
        ];

        // Add lightning address if configured
        if let Ok(lightning_address) = std::env::var("HYPER_UPPERCUT_LIGHTNING_ADDRESS") {
            tags.push(vec!["lud06".to_string(), lightning_address.clone()]);
            tags.push(vec!["lud16".to_string(), lightning_address.clone()]);
            tags.push(vec!["zap".to_string(), lightning_address]);
        }

        let event = nostr::Event::new(
            &secret_key,
            content,
            1, // kind 1 = text note
            tags
        );

        match relay_client.publish_event(event).await {
            Ok(_) => println!("Successfully published event to relay"),
            Err(e) => eprintln!("Failed to publish event: {}", e),
        }

        tokio::time::sleep(tokio::time::Duration::from_secs(note_delay_seconds)).await;

        println!("Sleeping for {} seconds before next feed check", feed_check_seconds);
        tokio::time::sleep(tokio::time::Duration::from_secs(feed_check_seconds)).await;
    }
} 