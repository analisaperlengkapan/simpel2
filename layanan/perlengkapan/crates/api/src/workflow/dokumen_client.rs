// ============================================================================
// Dokumen Service gRPC Client
// Description: Client for calling dokumen service from workflow engine
// Requirements: REQ-D005, REQ-W011
// ============================================================================

use tonic::transport::Channel;
use uuid::Uuid;

// Re-export generated proto types
pub use crate::workflow::dokumen_proto::document_service_client::DocumentServiceClient;
pub use crate::workflow::dokumen_proto::{
    GenerateDocumentRequest, GenerateDocumentResponse,
};

/// Dokumen client error types
#[derive(Debug, thiserror::Error)]
pub enum DokumenClientError {
    #[error("gRPC transport error: {0}")]
    TransportError(#[from] tonic::transport::Error),

    #[error("gRPC status error: {0}")]
    StatusError(#[from] tonic::Status),

    #[error("Invalid response: {0}")]
    InvalidResponse(String),
}

pub type Result<T> = std::result::Result<T, DokumenClientError>;

/// Dokumen service gRPC client wrapper
pub struct DokumenClient {
    client: DocumentServiceClient<Channel>,
}

impl DokumenClient {
    /// Create a new dokumen client
    pub async fn new(endpoint: &str) -> Result<Self> {
        let channel = Channel::from_shared(endpoint.to_string())
            .map_err(|e| DokumenClientError::InvalidResponse(format!("Invalid endpoint URI: {}", e)))?
            .connect()
            .await?;

        let client = DocumentServiceClient::new(channel);

        Ok(Self { client })
    }

    /// Generate a document from template
    pub async fn generate_document(
        &mut self,
        template_id: Uuid,
        data: serde_json::Value,
        output_format: Option<String>,
        metadata: Option<serde_json::Value>,
    ) -> Result<GenerateDocumentResponse> {
        let request = GenerateDocumentRequest {
            template_id: template_id.to_string(),
            data_json: data.to_string(),
            output_format,
            metadata_json: metadata.map(|m| m.to_string()),
        };

        let response = self.client.generate_document(request).await?;

        Ok(response.into_inner())
    }
}

/// Document generation result
#[derive(Debug, Clone)]
pub struct DocumentGenerationResult {
    pub document_id: Uuid,
    pub document_number: String,
    pub filename: String,
    pub download_url: String,
    pub checksum: String,
}

impl TryFrom<GenerateDocumentResponse> for DocumentGenerationResult {
    type Error = DokumenClientError;

    fn try_from(response: GenerateDocumentResponse) -> Result<Self> {
        let document_id = Uuid::parse_str(&response.document_id)
            .map_err(|e| DokumenClientError::InvalidResponse(format!("Invalid document_id: {}", e)))?;

        Ok(Self {
            document_id,
            document_number: response.document_number,
            filename: response.filename,
            download_url: response.download_url,
            checksum: response.checksum,
        })
    }
}
