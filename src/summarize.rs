use ollama_rs::Ollama;
use ollama_rs::generation::completion::{request::GenerationRequest, GenerationContext};
use rss::Item;

pub struct Summarizer {
    client: Ollama,
}

impl Summarizer {
    pub fn new(ollama_url: String) -> Self {
        Self {
            client: Ollama::new(ollama_url, 11434),
        }
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
            "Please provide a concise summary of these RSS feed items in 2-3 sentences. Don't include links or other external references. This should only be text.\n\n{}",
            combined_text
        );

        let request = GenerationRequest::new(
            "qwen2.5".to_string(),
            prompt,
        );

        let response = self.client.generate(request).await?;
        Ok(response.response)
    }
} 