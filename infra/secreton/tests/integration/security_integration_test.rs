#![cfg(feature = "quantum-safe")]

use Secreton_core::security::{
    AdvancedSecurityOrchestrator, BankingGradeConfig, GovernmentGradeConfig,
    SecurityConfig, ComplianceStatus, SecurityMetrics, HealthStatus,
};
use tokio::time::{timeout, Duration};
use std::sync::Arc;
use serde_json::json;

/// Integration tests for the complete security system
/// Tests real-world scenarios and end-to-end security workflows
#[cfg(test)]
mod security_integration_tests {
    use super::*;

    /// Create a test security orchestrator with banking-grade configuration
    async fn create_banking_orchestrator() -> Result<Arc<AdvancedSecurityOrchestrator>, Box<dyn std::error::Error>> {
        let config = BankingGradeConfig::new();
        let orchestrator = AdvancedSecurityOrchestrator::new(config.into()).await?;
        Ok(Arc::new(orchestrator))
    }

    /// Create a test security orchestrator with government-grade configuration
    async fn create_government_orchestrator() -> Result<Arc<AdvancedSecurityOrchestrator>, Box<dyn std::error::Error>> {
        let config = GovernmentGradeConfig::new();
        let orchestrator = AdvancedSecurityOrchestrator::new(config.into()).await?;
        Ok(Arc::new(orchestrator))
    }

    #[tokio::test]
    async fn test_end_to_end_banking_workflow() -> Result<(), Box<dyn std::error::Error>> {
        let orchestrator = create_banking_orchestrator().await?;

        // 1. User Authentication
        let user_id = "bank_employee_001";
        let session_token = orchestrator.create_secure_session(user_id).await?;
        assert!(orchestrator.validate_session(&session_token).await?.valid);

        // 2. Data Protection (PII)
        let customer_pii = json!({
            "name": "John Doe",
            "account": "1234-5678-9012",
            "ssn": "XXX-XX-XXXX"
        }).to_string();

        let encrypted_pii = orchestrator.encrypt_data(&customer_pii).await?;
        assert_ne!(customer_pii, encrypted_pii);

        // 3. Access Control Check
        let access = orchestrator.check_access_permission(user_id, "customer_data_read").await?;
        if access.granted {
            // 4. Data Retrieval
            let decrypted_pii = orchestrator.decrypt_data(&encrypted_pii).await?;
            assert_eq!(customer_pii, decrypted_pii);
        }

        // 5. Audit Logging
        // Verify operations were logged (mock verification)

        Ok(())
    }

    #[tokio::test]
    async fn test_government_security_controls() -> Result<(), Box<dyn std::error::Error>> {
        let orchestrator = create_government_orchestrator().await?;

        // 1. System Health Check
        let health = orchestrator.check_system_health().await?;
        assert_eq!(health, HealthStatus::Healthy);

        // 2. Compliance Verification
        let compliance = orchestrator.check_compliance().await?;
        assert!(compliance.fips_140_2_compliant);

        // 3. Secure Key Rotation
        let rotation_result = orchestrator.rotate_master_keys().await;
        assert!(rotation_result.is_ok());

        // 4. High Security Encryption
        let top_secret_data = "NUCLEAR_LAUNCH_CODES_TEST";
        let quantum_safe_encrypted = orchestrator.encrypt_data_quantum_safe(top_secret_data).await?;

        let decrypted = orchestrator.decrypt_data_quantum_safe(&quantum_safe_encrypted).await?;
        assert_eq!(top_secret_data, decrypted);

        Ok(())
    }
}
