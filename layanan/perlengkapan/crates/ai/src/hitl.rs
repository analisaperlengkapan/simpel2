pub struct HitlService;

impl HitlService {
    pub async fn annotate(&self, _data: &str) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }
}
