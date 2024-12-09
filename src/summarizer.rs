use ollama_rs::Ollama;
use ollama_rs::generation::completion::{request::GenerationRequest, GenerationContext};

pub struct Summarizer {
    client: Ollama,
}

impl Summarizer {
    pub fn new(ollama_url: String) -> Self {
        Self {
            client: Ollama::new(ollama_url, None),
        }
    }

    pub async fn summarize(&self, text: &str) -> Result<String, Box<dyn std::error::Error>> {
        let prompt = format!(
            "Please provide a concise summary of the following text in 2-3 sentences:\n\n{}",
            text
        );

        let request = GenerationRequest::new(
            "llama2".to_string(),
            prompt,
        );

        let response = self.client.generate(request).await?;
        Ok(response.response)
    }
}