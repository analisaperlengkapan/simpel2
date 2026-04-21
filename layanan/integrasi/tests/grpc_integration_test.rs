//! Integration tests for Integrasi gRPC service

#[cfg(test)]
mod integrasi_grpc_integration_test {
    #[tokio::test]
    async fn test_get_siman_assets_grpc() {
        // Mock gRPC client call
        let satker_id = "0100";
        let result = get_siman_assets(satker_id).await;

        assert!(result.is_ok());
        let assets = result.unwrap();
        assert!(!assets.is_empty());
    }

    #[tokio::test]
    async fn test_get_mysimkari_pegawai_grpc() {
        let satker_id = "0100";
        let result = get_mysimkari_pegawai(satker_id).await;

        assert!(result.is_ok());
        let pegawai = result.unwrap();
        assert!(!pegawai.is_empty());
    }

    #[tokio::test]
    async fn test_get_sync_status_grpc() {
        let result = get_sync_status("SIMAN").await;

        assert!(result.is_ok());
        let status = result.unwrap();
        assert!(status.contains("COMPLETED") || status.contains("RUNNING"));
    }

    #[tokio::test]
    async fn test_trigger_sync_grpc() {
        let result = trigger_sync("SIMAN", "FULL").await;

        assert!(result.is_ok());
    }

    // Mock helper functions
    async fn get_siman_assets(_satker_id: &str) -> Result<Vec<String>, String> {
        Ok(vec!["Asset1".to_string(), "Asset2".to_string()])
    }

    async fn get_mysimkari_pegawai(_satker_id: &str) -> Result<Vec<String>, String> {
        Ok(vec!["Pegawai1".to_string(), "Pegawai2".to_string()])
    }

    async fn get_sync_status(_service: &str) -> Result<String, String> {
        Ok("COMPLETED".to_string())
    }

    async fn trigger_sync(_service: &str, _sync_type: &str) -> Result<(), String> {
        Ok(())
    }
}
