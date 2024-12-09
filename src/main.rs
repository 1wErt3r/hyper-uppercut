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
use std::time::Duration;

struct FeedConfig {
    url: String,
    title: Option<String>,
}

async fn parse_opml_with_ollama(opml_path: &str, summarizer: &Summarizer) -> Result<Vec<FeedConfig>, Box<dyn std::error::Error>> {
    let content = std::fs::read_to_string(opml_path)?;
    let urls = summarizer.extract_feed_urls(&content).await?;
    
    Ok(urls.into_iter()
        .map(|url| FeedConfig {
            url,
            title: None,
        })
        .collect())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Required environment variables
    let secret_key_str = std::env::var("HYPER_UPPERCUT_SECRET_KEY")
        .map_err(|_| "HYPER_UPPERCUT_SECRET_KEY environment variable not set")?;
    let secret_key = SecretKey::from_str(&secret_key_str)?;

    let opml_path = std::env::var("HYPER_UPPERCUT_OPML_PATH")
        .map_err(|_| "HYPER_UPPERCUT_OPML_PATH environment variable not set")?;
    
    let post_interval_seconds = std::env::var("HYPER_UPPERCUT_POST_INTERVAL_SECONDS")
        .unwrap_or_else(|_| "300".to_string()) // 5 minutes default
        .parse::<u64>()?;

    let relay_url = std::env::var("HYPER_UPPERCUT_RELAY_URL")
        .map_err(|_| "HYPER_UPPERCUT_RELAY_URL environment variable not set")?;
    let relay_client = RelayClient::new(relay_url);

    // Optional environment variables with defaults
    let feed_check_seconds = std::env::var("HYPER_UPPERCUT_FEED_CHECK_SECONDS")
        .unwrap_or_else(|_| "10".to_string())
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

    println!("Starting RSS feeds monitoring...");
    
    let summarizer = if let Ok(ollama_url) = std::env::var("HYPER_UPPERCUT_OLLAMA_URL") {
        Some(Summarizer::new(ollama_url))
    } else {
        None
    };

    loop {
        let summarizer = summarizer.as_ref().ok_or("HYPER_UPPERCUT_OLLAMA_URL not set")?;
        let feeds = parse_opml_with_ollama(&opml_path, summarizer).await?;
        
        for feed in feeds {
            let feed_reader = rss::FeedReader::new(feed.url.clone());
            let items = feed_reader.fetch_latest().await?;
            
            // Take only the top 3 items
            let top_items: Vec<_> = items.into_iter().take(3).collect();
            
            println!("Fetched {} items from feed: {}", top_items.len(), feed.url);
            
            let content = match summarizer.summarize_feed(&top_items).await {
                Ok(summary) => {
                    let links = top_items.iter()
                        .filter_map(|item| item.link())
                        .collect::<Vec<_>>()
                        .join("\n");
                    
                    format!("{}\n\n{}", summary, links)
                },
                Err(e) => {
                    eprintln!("Failed to summarize content: {}", e);
                    format!(
                        "Latest items:\n\n{}",
                        top_items.iter()
                            .filter_map(|item| {
                                Some(format!("{}\n{}", 
                                    item.title().unwrap_or("No title"),
                                    item.link().unwrap_or("No link")))
                            })
                            .collect::<Vec<_>>()
                            .join("\n\n")
                    )
                }
            };

            // Create and publish the event
            let mut tags = vec![
                vec!["client".to_string(), "hyper-uppercut".to_string()],
                vec!["alt".to_string(), "RSS Feed Summary".to_string()]
            ];

            if let Ok(lightning_address) = std::env::var("HYPER_UPPERCUT_LIGHTNING_ADDRESS") {
                tags.push(vec!["lud06".to_string(), lightning_address.clone()]);
                tags.push(vec!["lud16".to_string(), lightning_address.clone()]);
                tags.push(vec!["zap".to_string(), lightning_address]);
            }

            let event = nostr::Event::new(
                &secret_key,
                content,
                1,
                tags
            );

            match relay_client.publish_event(event).await {
                Ok(_) => println!("Successfully published event for feed: {}", feed.url),
                Err(e) => eprintln!("Failed to publish event: {}", e),
            }

            tokio::time::sleep(Duration::from_secs(post_interval_seconds)).await;
        }

        println!("Sleeping for {} seconds before next feed check", feed_check_seconds);
        tokio::time::sleep(Duration::from_secs(feed_check_seconds)).await;
    }
} 