use rss::Channel;
use reqwest;

pub struct FeedReader {
    url: String,
}

impl FeedReader {
    pub fn new(url: String) -> Self {
        Self { url }
    }

    pub async fn fetch_latest(&self) -> Result<Vec<rss::Item>, Box<dyn std::error::Error>> {
        let content = reqwest::get(&self.url)
            .await?
            .bytes()
            .await?;

        let channel = Channel::read_from(&content[..])?;
        Ok(channel.items().to_vec())
    }
}