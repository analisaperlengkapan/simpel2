#[derive(Clone)]
pub struct RagService;

impl RagService {
    pub async fn new(_config: &crate::config::Config) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self)
    }
    pub async fn query(&self, _query: &str) -> Result<String, Box<dyn std::error::Error>> {
        Ok("dummy rag answer".to_string())
    }
    pub async fn ingest(&self, _data: &str) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }
}
