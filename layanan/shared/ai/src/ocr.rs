#[derive(Clone, Debug)]
pub struct OcrService;

impl OcrService {
    pub async fn new(_config: &crate::config::Config) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self)
    }
    pub async fn extract_text(&self, _image_data: &[u8]) -> Result<String, Box<dyn std::error::Error>> {
        Ok("dummy ocr text".to_string())
    }
    pub async fn process_batch(&self, _documents: Vec<Vec<u8>>) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        Ok(vec!["dummy batch result".to_string()])
    }
} 