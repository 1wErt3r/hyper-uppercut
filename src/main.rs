mod nostr;
mod rss;
mod relay;
mod summarize;
mod opml;

use secp256k1::SecretKey;
use tokio;
use std::str::FromStr;
use relay::RelayPool;
use summarize::Summarizer;
use opml::OpmlParser;
use dotenv::dotenv;


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    // Required environment variables
    let secret_key_str = std::env::var("HYPER_UPPERCUT_SECRET_KEY")
        .map_err(|_| "HYPER_UPPERCUT_SECRET_KEY environment variable not set")?;
    let secret_key = SecretKey::from_str(&secret_key_str)?;
    
    let relay_urls = std::env::var("HYPER_UPPERCUT_RELAY_URLS")
        .map_err(|_| "HYPER_UPPERCUT_RELAY_URLS environment variable not set")?
        .split(',')
        .map(|s| s.trim().to_string())
        .collect::<Vec<_>>();

    println!("Configured relay URLs: {:?}", relay_urls);

    let relay_pool = RelayPool::new(relay_urls);

    let opml_source = std::env::var("HYPER_UPPERCUT_OPML_SOURCE")
        .map_err(|_| "HYPER_UPPERCUT_OPML_SOURCE environment variable not set")?;

    let ollama_url = std::env::var("HYPER_UPPERCUT_OLLAMA_URL")
        .map_err(|_| "HYPER_UPPERCUT_OLLAMA_URL environment variable not set")?;

    // Optional environment variables with defaults
    let feed_check_seconds = std::env::var("HYPER_UPPERCUT_FEED_CHECK_SECONDS")
        .unwrap_or_else(|_| "10000".to_string())
        .parse::<u64>()?;

    let note_delay_seconds = std::env::var("HYPER_UPPERCUT_NOTE_DELAY_SECONDS")
        .unwrap_or_else(|_| "60".to_string())
        .parse::<u64>()?;

    let process_interval = std::env::var("HYPER_UPPERCUT_PROCESS_INTERVAL_SECONDS")
        .unwrap_or_else(|_| "30".to_string())
        .parse::<u64>()?;

    // Initialize services
    let opml_parser = OpmlParser::new(ollama_url.clone());
    let summarizer = Summarizer::new(ollama_url);

    // Spawn the outbox processing task
    tokio::spawn({
        let relay_pool = relay_pool.clone();
        async move {
            loop {
                if let Err(e) = relay_pool.process_outbox().await {
                    eprintln!("Error processing outbox: {}", e);
                }
                tokio::time::sleep(tokio::time::Duration::from_secs(process_interval)).await;
            }
        }
    });

    println!("Starting OPML processing...");
    
    loop {
        if let Err(e) = process_feeds(
            &opml_parser,
            &summarizer,
            &relay_pool,
            &feed_check_seconds,
            &note_delay_seconds,
            &opml_source,
            &secret_key
        ).await {
            eprintln!("Error processing feeds: {}", e);
            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
        }
    }
}

async fn process_feeds(
    opml_parser: &OpmlParser,
    summarizer: &Summarizer,
    relay_pool: &RelayPool,
    feed_check_seconds: &u64,
    note_delay_seconds: &u64,
    opml_source: &str,
    secret_key: &SecretKey
) -> Result<(), Box<dyn std::error::Error>> {
    // Fetch and parse OPML
    let opml_content = opml_parser.fetch_opml(opml_source).await?;
    let feed_list = opml_parser.parse_opml(&opml_content).await?;
    
    println!("Found {} feeds in OPML", feed_list.feeds.len());

    // Process each feed
    for feed_url in feed_list.feeds {
        println!("Attempting to fetch feed: {}", feed_url);
        let feed_reader = rss::FeedReader::new(feed_url.clone());
        
        let items = match feed_reader.fetch_latest().await {
            Ok(mut items) => {
                println!("Successfully fetched {} items from feed", items.len());
                items.truncate(3);
                println!("Truncated to {} items", items.len());
                items
            },
            Err(e) => {
                eprintln!("Failed to fetch feed {}: {}", feed_url, e);
                continue;
            }
        };

        let content = match summarizer.summarize_feed(&items).await {
            Ok(summary) => {
                let links = items.iter()
                    .filter_map(|item| item.link())
                    .collect::<Vec<_>>()
                    .join("\n");
                
                format!(
                    "{}",
                    summary
                )
            },
            Err(e) => {
                eprintln!("Failed to summarize content: {}", e);
                continue;
            }
        };

        let mut tags = vec![
            vec!["client".to_string(), "hyper-uppercut".to_string()],
            vec!["alt".to_string(), "RSS Feed Summary".to_string()],
            vec!["r".to_string(), feed_url.clone()]
        ];

        if let Ok(lightning_address) = std::env::var("HYPER_UPPERCUT_LIGHTNING_ADDRESS") {
            tags.push(vec!["lud06".to_string(), lightning_address.clone()]);
            tags.push(vec!["lud16".to_string(), lightning_address.clone()]);
            tags.push(vec!["zap".to_string(), lightning_address]);
        }

        let event = nostr::Event::new(
            secret_key,
            content,
            1,
            tags
        );

        relay_pool.add_to_outbox(event).await;
        println!("Added event to outbox for feed: {}", feed_url);
        tokio::time::sleep(tokio::time::Duration::from_secs(*note_delay_seconds)).await;
    }

    println!("Sleeping for {} seconds before next OPML check", feed_check_seconds);
    tokio::time::sleep(tokio::time::Duration::from_secs(*feed_check_seconds)).await;
    Ok(())
} 