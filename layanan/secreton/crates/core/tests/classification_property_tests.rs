//! Property-Based Tests for Data Classification Service
//!
//! These tests verify the correctness properties of the data classification system.

use proptest::prelude::*;
use secreton_core::services::classification::{
    ClassificationLevel, ClassificationService, InMemoryClassificationService,
};
use secreton_core::services::mfa::MfaService;
use std::sync::Arc;

// Helper to setup test service without MFA
async fn setup_test_service() -> Arc<InMemoryClassificationService> {
    Arc::new(InMemoryClassificationService::new())
}

// Strategy for generating classification levels
fn classification_level_strategy() -> impl Strategy<Value = ClassificationLevel> {
    prop_oneof![
        Just(ClassificationLevel::Biasa),
        Just(ClassificationLevel::Terbatas),
        Just(ClassificationLevel::Rahasia),
        Just(ClassificationLevel::SangatRahasia),
    ]
}

// Strategy for generating secret paths
fn secret_path_strategy() -> impl Strategy<Value = String> {
    prop::string::string_regex("secret/[a-z0-9_-]{5,30}").expect("Valid regex")
}

// Strategy for generating user IDs
fn user_id_strategy() -> impl Strategy<Value = String> {
    prop::string::string_regex("user[0-9]{1,5}").expect("Valid regex")
}

// **Feature: secreton-comprehensive-enhancement, Property 23: Classification Enforcement**
// **Validates: Requirements 9.2**
//
// Property: For any secret with classification RAHASIA or SANGAT_RAHASIA, access without
// MFA verification SHALL be denied.
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn property_classification_enforcement(
        path in secret_path_strategy(),
        user_id in user_id_strategy(),
        level in classification_level_strategy(),
    ) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let service = setup_test_service().await;

            // Classify the secret
            service.classify(&path, level, &user_id).await
                .expect("Classification should succeed");

            // Property 1: High-classification secrets require MFA
            let requires_mfa = service.require_mfa(&path).await
                .expect("MFA check should succeed");

            if level == ClassificationLevel::Rahasia || level == ClassificationLevel::SangatRahasia {
                prop_assert!(requires_mfa,
                    "RAHASIA and SANGAT_RAHASIA should require MFA");

                // Property 2: Access without MFA should be denied for high-classification
                let has_access = service.check_access(&path, level, false).await
                    .expect("Access check should succeed");

                prop_assert!(!has_access,
                    "Access without MFA should be denied for {} level", level);

                // Property 3: Access with MFA should be granted (if clearance sufficient)
                let has_access_with_mfa = service.check_access(&path, level, true).await
                    .expect("Access check should succeed");

                prop_assert!(has_access_with_mfa,
                    "Access with MFA should be granted for {} level with sufficient clearance", level);
             } else {
                // Property 4: Low-classification secrets don't require MFA
                prop_assert!(!requires_mfa,
                    "BIASA and TERBATAS should not require MFA");

                // Property 5: Access without MFA should be granted for low-classification
                let has_access = service.check_access(&path, level, false).await
                    .expect("Access check should succeed");

                prop_assert!(has_access,
                    "Access without MFA should be granted for {} level", level);
            }

            Ok(())
        })?;
    }
}

// **Feature: secreton-comprehensive-enhancement, Property 23: Classification Enforcement (MFA Integration)**
// **Validates: Requirements 9.2**
//
// Property: When MFA service is integrated, enforcement should check user MFA configuration
// for high-classification secrets.
proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    #[test]
    fn property_classification_mfa_integration(
        path in secret_path_strategy(),
        user_id in user_id_strategy(),
    ) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let mfa_service = Arc::new(MfaService::new());
            let service = Arc::new(InMemoryClassificationService::with_mfa(mfa_service.clone()));

            // Classify as RAHASIA (requires MFA)
            service.classify(&path, ClassificationLevel::Rahasia, &user_id).await
                .expect("Classification should succeed");

            // Property 1: Enforcement should fail when user has no MFA configured
            let result = service.enforce_mfa(&path, &user_id).await;
            prop_assert!(result.is_err(),
                "MFA enforcement should fail when user has no MFA configured");

            // Enable MFA for user
            mfa_service.enable_totp(
                &user_id,
                "Secreton".to_string(),
                format!("{}@test.com", user_id)
            ).await.expect("MFA setup should succeed");

            // Property 2: Enforcement should pass when user has MFA configured
            let result = service.enforce_mfa(&path, &user_id).await;
            prop_assert!(result.is_ok(),
                "MFA enforcement should pass when user has MFA configured");

            Ok(())
        })?;
    }
}

// **Feature: secreton-comprehensive-enhancement, Property 24: Clearance Level Access Control**
// **Validates: Requirements 9.5**
//
// Property: For any user with clearance level L, access to secrets with classification
// higher than L SHALL be denied.
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn property_clearance_level_access_control(
        path in secret_path_strategy(),
        user_id in user_id_strategy(),
        secret_level in classification_level_strategy(),
        user_clearance in classification_level_strategy(),
    ) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let service = setup_test_service().await;

            // Classify the secret
            service.classify(&path, secret_level, &user_id).await
                .expect("Classification should succeed");

            // Check clearance
            let has_clearance = service.check_clearance(&path, user_clearance).await
                .expect("Clearance check should succeed");

            // Property 1: User with sufficient or higher clearance should have access
            if user_clearance >= secret_level {
                prop_assert!(has_clearance,
                    "User with clearance {} should have access to {} secret",
                    user_clearance, secret_level);
             } else {
                // Property 2: User with insufficient clearance should be denied
                prop_assert!(!has_clearance,
                    "User with clearance {} should NOT have access to {} secret",
                    user_clearance, secret_level);
            }

            // Property 3: Highest clearance (SANGAT_RAHASIA) should access all levels
            let highest_clearance = service.check_clearance(&path, ClassificationLevel::SangatRahasia).await
                .expect("Clearance check should succeed");
            prop_assert!(highest_clearance,
                "User with SANGAT_RAHASIA clearance should access all secrets");

            // Property 4: Lowest clearance (BIASA) should only access BIASA secrets
            let lowest_clearance = service.check_clearance(&path, ClassificationLevel::Biasa).await
                .expect("Clearance check should succeed");
            if secret_level == ClassificationLevel::Biasa {
                prop_assert!(lowest_clearance,
                    "User with BIASA clearance should access BIASA secrets");
             } else {
                prop_assert!(!lowest_clearance,
                    "User with BIASA clearance should NOT access {} secrets", secret_level);
            }

            Ok(())
        })?;
    }
}

// Edge case tests
#[cfg(test)]
mod classification_edge_cases {
    use super::*;

    #[tokio::test]
    async fn test_classification_enforcement_biasa_no_mfa() {
        let service = setup_test_service().await;

        service
            .classify("secret/public", ClassificationLevel::Biasa, "user1")
            .await
            .unwrap();

        // BIASA should not require MFA
        assert!(!service.require_mfa("secret/public").await.unwrap());

        // Access should be granted without MFA
        assert!(
            service
                .check_access("secret/public", ClassificationLevel::Biasa, false)
                .await
                .unwrap()
        );
    }

    #[tokio::test]
    async fn test_classification_enforcement_terbatas_no_mfa() {
        let service = setup_test_service().await;

        service
            .classify("secret/internal", ClassificationLevel::Terbatas, "user1")
            .await
            .unwrap();

        // TERBATAS should not require MFA
        assert!(!service.require_mfa("secret/internal").await.unwrap());

        // Access should be granted without MFA
        assert!(
            service
                .check_access("secret/internal", ClassificationLevel::Terbatas, false)
                .await
                .unwrap()
        );
    }

    #[tokio::test]
    async fn test_classification_enforcement_rahasia_requires_mfa() {
        let service = setup_test_service().await;

        service
            .classify("secret/confidential", ClassificationLevel::Rahasia, "user1")
            .await
            .unwrap();

        // RAHASIA should require MFA
        assert!(service.require_mfa("secret/confidential").await.unwrap());

        // Access without MFA should be denied
        assert!(
            !service
                .check_access("secret/confidential", ClassificationLevel::Rahasia, false)
                .await
                .unwrap()
        );

        // Access with MFA should be granted
        assert!(
            service
                .check_access("secret/confidential", ClassificationLevel::Rahasia, true)
                .await
                .unwrap()
        );
    }

    #[tokio::test]
    async fn test_classification_enforcement_sangat_rahasia_requires_mfa() {
        let service = setup_test_service().await;

        service
            .classify(
                "secret/topsecret",
                ClassificationLevel::SangatRahasia,
                "user1",
            )
            .await
            .unwrap();

        // SANGAT_RAHASIA should require MFA
        assert!(service.require_mfa("secret/topsecret").await.unwrap());

        // Access without MFA should be denied
        assert!(
            !service
                .check_access(
                    "secret/topsecret",
                    ClassificationLevel::SangatRahasia,
                    false
                )
                .await
                .unwrap()
        );

        // Access with MFA should be granted
        assert!(
            service
                .check_access("secret/topsecret", ClassificationLevel::SangatRahasia, true)
                .await
                .unwrap()
        );
    }

    #[tokio::test]
    async fn test_classification_enforcement_multiple_secrets() {
        let service = setup_test_service().await;

        // Create secrets at different levels
        service
            .classify("secret/public", ClassificationLevel::Biasa, "user1")
            .await
            .unwrap();
        service
            .classify("secret/internal", ClassificationLevel::Terbatas, "user1")
            .await
            .unwrap();
        service
            .classify("secret/confidential", ClassificationLevel::Rahasia, "user1")
            .await
            .unwrap();
        service
            .classify(
                "secret/topsecret",
                ClassificationLevel::SangatRahasia,
                "user1",
            )
            .await
            .unwrap();

        // Verify MFA requirements
        assert!(!service.require_mfa("secret/public").await.unwrap());
        assert!(!service.require_mfa("secret/internal").await.unwrap());
        assert!(service.require_mfa("secret/confidential").await.unwrap());
        assert!(service.require_mfa("secret/topsecret").await.unwrap());

        // Verify access without MFA
        assert!(
            service
                .check_access("secret/public", ClassificationLevel::Biasa, false)
                .await
                .unwrap()
        );
        assert!(
            service
                .check_access("secret/internal", ClassificationLevel::Terbatas, false)
                .await
                .unwrap()
        );
        assert!(
            !service
                .check_access("secret/confidential", ClassificationLevel::Rahasia, false)
                .await
                .unwrap()
        );
        assert!(
            !service
                .check_access(
                    "secret/topsecret",
                    ClassificationLevel::SangatRahasia,
                    false
                )
                .await
                .unwrap()
        );
    }

    #[tokio::test]
    async fn test_classification_enforcement_with_mfa_service() {
        let mfa_service = Arc::new(MfaService::new());
        let service = Arc::new(InMemoryClassificationService::with_mfa(mfa_service.clone()));

        service
            .classify("secret/confidential", ClassificationLevel::Rahasia, "user1")
            .await
            .unwrap();

        // User without MFA should fail enforcement
        let result = service.enforce_mfa("secret/confidential", "user1").await;
        assert!(result.is_err());

        // Enable MFA for user
        mfa_service
            .enable_totp(
                "user1",
                "Secreton".to_string(),
                "user1@test.com".to_string(),
            )
            .await
            .unwrap();

        // Now enforcement should pass
        let result = service.enforce_mfa("secret/confidential", "user1").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_classification_enforcement_nonexistent_secret() {
        let service = setup_test_service().await;

        // Non-existent secret should not require MFA (defaults to BIASA)
        assert!(!service.require_mfa("secret/nonexistent").await.unwrap());

        // Access should be granted
        assert!(
            service
                .check_access("secret/nonexistent", ClassificationLevel::Biasa, false)
                .await
                .unwrap()
        );
    }
}
