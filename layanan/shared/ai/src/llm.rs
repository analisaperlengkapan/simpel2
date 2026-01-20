#[derive(Clone, Debug)]
pub struct LlmService;

impl LlmService {
    pub async fn new(_config: &crate::config::Config) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self)
    }
    pub async fn generate_text(&self, _prompt: &str, _max_tokens: usize) -> Result<String, Box<dyn std::error::Error>> {
        Ok("dummy text".to_string())
    }
    pub async fn summarize_text(&self, _text: &str) -> Result<String, Box<dyn std::error::Error>> {
        Ok("dummy summary".to_string())
    }
    pub async fn classify_text(&self, _text: &str, _categories: &[String]) -> Result<String, Box<dyn std::error::Error>> {
        Ok("dummy class".to_string())
    }
} 