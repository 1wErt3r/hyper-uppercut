use ollama_rs::Ollama;
use ollama_rs::generation::completion::{request::GenerationRequest, GenerationContext};
use rss::Item;
use serde_json;

pub struct Summarizer {
    client: Ollama,
}

impl Summarizer {
    pub fn new(ollama_url: String) -> Self {
        Self {
            client: Ollama::new(ollama_url, 11434),
        }
    }

    pub async fn extract_feed_urls(&self, opml_content: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let prompt = format!(
            "Extract all RSS feed URLs (xmlUrl attributes) from this OPML file and return them as a JSON array of strings. Only include the URLs, nothing else:\n\n{}",
            opml_content
        );

        let request = GenerationRequest::new(
            "llama3.2".to_string(),
            prompt,
        );

        let response = self.client.generate(request).await?;
        
        // Extract JSON array from the response
        let response_text = response.response.trim();
        let json_start = response_text.find('[').ok_or("No JSON array found")?;
        let json_end = response_text.rfind(']').ok_or("No JSON array end found")?;
        let json_str = &response_text[json_start..=json_end];
        
        let urls: Vec<String> = serde_json::from_str(json_str)?;
        Ok(urls)
    }

    pub async fn summarize_feed(&self, items: &[Item]) -> Result<String, Box<dyn std::error::Error>> {
        let combined_text = items.iter()
            .map(|item| format!(
                "Title: {}\nDescription: {}\n",
                item.title().unwrap_or("No title"),
                item.description().unwrap_or("No description")
            ))
            .collect::<Vec<_>>()
            .join("\n---\n");

        let prompt = format!(
            "Please provide a concise summary of these RSS feed items in 2-3 sentences:\n\n{}",
            combined_text
        );

        let request = GenerationRequest::new(
            "llama3.2".to_string(),
            prompt,
        );

        let response = self.client.generate(request).await?;
        Ok(response.response)
    }
} 