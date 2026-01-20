pub struct ActiveLearningService;

impl ActiveLearningService {
    pub async fn query(&self, _data: &str) -> Result<String, Box<dyn std::error::Error>> {
        Ok("dummy active learning result".to_string())
    }
} 