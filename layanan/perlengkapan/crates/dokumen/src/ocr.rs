use crate::config::AppConfig;
use crate::error::AppError;
use crate::models::OcrResult;
use reqwest::Client;
use serde_json::json;
use uuid::Uuid;

pub struct OcrService {
    pub config: AppConfig,
    client: Client,
}

impl OcrService {
    pub fn new(config: AppConfig) -> Self {
        Self {
            config,
            client: Client::new(),
        }
    }

    pub async fn request_ocr(
        &self,
        _document_id: Uuid,
        file_bytes: Vec<u8>,
    ) -> Result<OcrResult, AppError> {
        let url = format!("{}/ocr", self.config.ai_service_url);
        let mut req = self
            .client
            .post(&url)
            .header("Content-Type", "application/octet-stream");
        if let Some(ref key) = self.config.ai_service_api_key {
            req = req.header("x-api-key", key);
        }
        let resp = req
            .body(file_bytes)
            .send()
            .await
            .map_err(|e| AppError::Ai(e.to_string()))?;
        if !resp.status().is_success() {
            return Err(AppError::Ai(format!("AI OCR gagal: {}", resp.status())));
        }
        let ocr: OcrResult = resp.json().await.map_err(|e| AppError::Ai(e.to_string()))?;
        Ok(ocr)
    }

    pub async fn get_ocr_status(&self, document_id: Uuid) -> Result<String, AppError> {
        let url = format!("{}/ocr/{}/status", self.config.ai_service_url, document_id);
        let mut req = self.client.get(&url);
        if let Some(ref key) = self.config.ai_service_api_key {
            req = req.header("x-api-key", key);
        }
        let resp = req.send().await.map_err(|e| AppError::Ai(e.to_string()))?;
        if !resp.status().is_success() {
            return Err(AppError::Ai(format!(
                "AI OCR status gagal: {}",
                resp.status()
            )));
        }
        let status: serde_json::Value =
            resp.json().await.map_err(|e| AppError::Ai(e.to_string()))?;
        Ok(status["status"].as_str().unwrap_or("unknown").to_string())
    }

    pub async fn get_ocr_text(&self, document_id: Uuid) -> Result<String, AppError> {
        let url = format!("{}/ocr/{}/text", self.config.ai_service_url, document_id);
        let mut req = self.client.get(&url);
        if let Some(ref key) = self.config.ai_service_api_key {
            req = req.header("x-api-key", key);
        }
        let resp = req.send().await.map_err(|e| AppError::Ai(e.to_string()))?;
        if !resp.status().is_success() {
            return Err(AppError::Ai(format!(
                "AI OCR text gagal: {}",
                resp.status()
            )));
        }
        let text: serde_json::Value = resp.json().await.map_err(|e| AppError::Ai(e.to_string()))?;
        Ok(text["text"].as_str().unwrap_or("").to_string())
    }
}
