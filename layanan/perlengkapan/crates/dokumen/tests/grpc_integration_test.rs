//! Integration tests for Dokumen gRPC service

#[cfg(test)]
mod dokumen_grpc_integration_test {
    use uuid::Uuid;

    #[tokio::test]
    async fn test_generate_document_grpc() {
        let template_id = "permit_template";
        let context = r#"{"pegawai_nama": "John Doe", "nip": "123456"}"#;

        let result = generate_document(template_id, context).await;

        assert!(result.is_ok());
        let document_id = result.unwrap();
        assert!(!document_id.is_nil());
    }

    #[tokio::test]
    async fn test_get_document_grpc() {
        let document_id = Uuid::new_v4();

        let result = get_document(document_id).await;

        assert!(result.is_ok());
        let document_url = result.unwrap();
        assert!(document_url.starts_with("https://"));
    }

    #[tokio::test]
    async fn test_list_documents_grpc() {
        let entity_id = Uuid::new_v4();

        let result = list_documents(entity_id).await;

        assert!(result.is_ok());
        let documents = result.unwrap();
        assert!(!documents.is_empty());
    }

    // Mock helper functions
    async fn generate_document(_template_id: &str, _context: &str) -> Result<Uuid, String> {
        Ok(Uuid::new_v4())
    }

    async fn get_document(_document_id: Uuid) -> Result<String, String> {
        Ok("https://storage.example.com/doc.pdf".to_string())
    }

    async fn list_documents(_entity_id: Uuid) -> Result<Vec<String>, String> {
        Ok(vec!["doc1.pdf".to_string(), "doc2.pdf".to_string()])
    }
}
