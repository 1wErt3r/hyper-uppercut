use reqwest;
use serde::{Deserialize, Serialize};
use ollama_rs::Ollama;
use ollama_rs::generation::completion::{request::GenerationRequest};
use ollama_rs::generation::parameters::FormatType;

#[derive(Debug, Serialize, Deserialize)]
pub struct FeedList {
    pub feeds: Vec<String>
}

pub struct OpmlParser {
    client: Ollama,
}

impl OpmlParser {
    pub fn new(ollama_url: String) -> Self {
        Self {
            client: Ollama::new(ollama_url, 11434),
        }
    }

    pub async fn parse_opml(&self, opml_content: &str) -> Result<FeedList, Box<dyn std::error::Error>> {
        let prompt = format!(
            "Extract all RSS/Atom feed URLs from this OPML file. Respond in JSON format with a single 'feeds' array containing the URLs. OPML content:\n\n{}",
            opml_content
        );

        let request = GenerationRequest::new(
            "llama3.2".to_string(),
            prompt,
        ).format(FormatType::Json);

        let response = self.client.generate(request).await?;
        let feed_list: FeedList = serde_json::from_str(&response.response)?;
        
        Ok(feed_list)
    }

    pub async fn fetch_opml(&self, url: &str) -> Result<String, Box<dyn std::error::Error>> {
        let content = if url.starts_with("http") {
            reqwest::get(url).await?.text().await?
        } else {
            std::fs::read_to_string(url)?
        };
        Ok(content)
    }
} 