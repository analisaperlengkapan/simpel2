//! Enhanced Unit Tests for SIMKARI User Model
//!
//! This test suite covers the enhanced functionality for:
//! - Role-based access control with hierarchical structure
//! - Satker (organizational unit) hierarchy validation
//! - Secreton access policy validation
//! - Admin level management and permissions
//! - Security context validation

#[cfg(test)]
mod enhanced_user_model_tests {
    use authenc::models::user::*;
    use chrono::{Duration, NaiveDate, NaiveTime, TimeZone, Utc};
    use serde_json::json;
    use uuid::Uuid;

    #[test]
    fn test_admin_level_hierarchy() {
        let admin_pusat = AdminLevel::AdminPusat;
        let admin_eselon1 = AdminLevel::AdminEselonI;
        let admin_wilayah = AdminLevel::AdminWilayah("KEJATI_DKI".to_string());
        let admin_satker1 = AdminLevel::AdminSatker("KEJATI_DKI_JAKPUS".to_string());
        let admin_satker2 = AdminLevel::AdminSatker("KEJATI_JABAR_BANDUNG".to_string());

        // AdminPusat can manage everyone
        assert!(admin_pusat.can_manage(&admin_eselon1));
        assert!(admin_pusat.can_manage(&admin_wilayah));
        assert!(admin_pusat.can_manage(&admin_satker1));
        assert!(admin_pusat.can_manage(&admin_satker2));

        // AdminEselonI can manage wilayah and satker
        assert!(admin_eselon1.can_manage(&admin_wilayah));
        assert!(admin_eselon1.can_manage(&admin_satker1));
        assert!(admin_eselon1.can_manage(&admin_satker2));
        assert!(!admin_eselon1.can_manage(&admin_pusat));

        // AdminWilayah can only manage satker in their region
        assert!(admin_wilayah.can_manage(&admin_satker1));
        assert!(!admin_wilayah.can_manage(&admin_satker2));
        assert!(!admin_wilayah.can_manage(&admin_eselon1));
        assert!(!admin_wilayah.can_manage(&admin_pusat));

        // AdminSatker can only manage their own satker
        assert!(admin_satker1.can_manage(&admin_satker1));
        assert!(!admin_satker1.can_manage(&admin_satker2));
        assert!(!admin_satker1.can_manage(&admin_wilayah));
    }

    #[test]
    fn test_admin_level_scope_mapping() {
        let admin_satker = AdminLevel::AdminSatker("KEJATI_DKI_JAKPUS".to_string());
        let admin_wilayah = AdminLevel::AdminWilayah("KEJATI_DKI".to_string());
        let admin_eselon1 = AdminLevel::AdminEselonI;
        let admin_pusat = AdminLevel::AdminPusat;

        // Test scope mapping
        assert_eq!(
            admin_satker.get_scope(),
            RoleScope::Satker("KEJATI_DKI_JAKPUS".to_string())
        );
        assert_eq!(
            admin_wilayah.get_scope(),
            RoleScope::Wilayah("KEJATI_DKI".to_string())
        );
        assert_eq!(admin_eselon1.get_scope(), RoleScope::Pusat);
        assert_eq!(admin_pusat.get_scope(), RoleScope::Pusat);
    }

    #[test]
    fn test_role_scope_satker_inclusion() {
        let scope_satker = RoleScope::Satker("KEJATI_DKI_JAKPUS".to_string());
        let scope_wilayah = RoleScope::Wilayah("KEJATI_DKI".to_string());
        let scope_pusat = RoleScope::Pusat;

        // Test satker inclusion
        assert!(scope_satker.includes_satker("KEJATI_DKI_JAKPUS"));
        assert!(!scope_satker.includes_satker("KEJATI_DKI_JAKSEL"));
        assert!(!scope_satker.includes_satker("KEJATI_JABAR_BANDUNG"));

        // Test wilayah inclusion
        assert!(scope_wilayah.includes_satker("KEJATI_DKI_JAKPUS"));
        assert!(scope_wilayah.includes_satker("KEJATI_DKI_JAKSEL"));
        assert!(!scope_wilayah.includes_satker("KEJATI_JABAR_BANDUNG"));

        // Test pusat inclusion (includes all)
        assert!(scope_pusat.includes_satker("KEJATI_DKI_JAKPUS"));
        assert!(scope_pusat.includes_satker("KEJATI_JABAR_BANDUNG"));
        assert!(scope_pusat.includes_satker("KEJARI_SOLO"));
    }

    #[test]
    fn test_role_scope_access_control() {
        let scope_pusat = RoleScope::Pusat;
        let scope_wilayah_dki = RoleScope::Wilayah("KEJATI_DKI".to_string());
        let scope_wilayah_jabar = RoleScope::Wilayah("KEJATI_JABAR".to_string());
        let scope_satker_jakpus = RoleScope::Satker("KEJATI_DKI_JAKPUS".to_string());
        let scope_satker_bandung = RoleScope::Satker("KEJATI_JABAR_BANDUNG".to_string());

        // Pusat can access everything
        assert!(scope_pusat.can_access(&scope_wilayah_dki));
        assert!(scope_pusat.can_access(&scope_wilayah_jabar));
        assert!(scope_pusat.can_access(&scope_satker_jakpus));
        assert!(scope_pusat.can_access(&scope_satker_bandung));

        // Wilayah can access same wilayah and its satker
        assert!(scope_wilayah_dki.can_access(&scope_wilayah_dki));
        assert!(scope_wilayah_dki.can_access(&scope_satker_jakpus));
        assert!(!scope_wilayah_dki.can_access(&scope_wilayah_jabar));
        assert!(!scope_wilayah_dki.can_access(&scope_satker_bandung));

        // Satker can only access same satker
        assert!(scope_satker_jakpus.can_access(&scope_satker_jakpus));
        assert!(!scope_satker_jakpus.can_access(&scope_satker_bandung));
        assert!(!scope_satker_jakpus.can_access(&scope_wilayah_dki));
    }

    #[test]
    fn test_secreton_access_policy_path_validation() {
        let policy = SecretonAccessPolicy {
            allowed_satker_secrets: vec![
                "KEJATI_DKI_JAKPUS".to_string(),
                "KEJATI_DKI_JAKSEL".to_string(),
            ],
            access_level: AccessLevel::ReadWrite,
            time_restrictions: None,
            audit_required: true,
            rate_limit: Some(100),
            allowed_paths: Some(vec![
                "/app/config/".to_string(),
                "/user/credentials/".to_string(),
            ]),
            denied_paths: Some(vec!["/admin/".to_string(), "/system/root/".to_string()]),
        };

        // Test allowed paths with allowed satker
        assert!(policy.can_access_path("/app/config/database", "KEJATI_DKI_JAKPUS"));
        assert!(policy.can_access_path("/user/credentials/api_key", "KEJATI_DKI_JAKSEL"));

        // Test denied paths (should be denied even with allowed satker)
        assert!(!policy.can_access_path("/admin/users", "KEJATI_DKI_JAKPUS"));
        assert!(!policy.can_access_path("/system/root/password", "KEJATI_DKI_JAKSEL"));

        // Test disallowed satker
        assert!(!policy.can_access_path("/app/config/database", "KEJATI_JABAR_BANDUNG"));

        // Test path not in allowed list
        assert!(!policy.can_access_path("/other/path", "KEJATI_DKI_JAKPUS"));
    }

    #[test]
    fn test_secreton_access_policy_time_restrictions() {
        let time_restrictions = TimeRestrictions {
            start_hour: 8,                     // 8 AM
            end_hour: 17,                      // 5 PM
            allowed_days: vec![1, 2, 3, 4, 5], // Monday to Friday
            timezone: "Asia/Jakarta".to_string(),
        };

        let policy = SecretonAccessPolicy {
            allowed_satker_secrets: vec!["KEJATI_DKI_JAKPUS".to_string()],
            access_level: AccessLevel::ReadOnly,
            time_restrictions: Some(time_restrictions),
            audit_required: true,
            rate_limit: None,
            allowed_paths: None,
            denied_paths: None,
        };

        // Create test times using specific known dates
        // January 8, 2024 is a Monday
        let monday_date = NaiveDate::from_ymd_opt(2024, 1, 8).unwrap();
        let monday_10am = Utc
            .from_utc_datetime(&monday_date.and_time(NaiveTime::from_hms_opt(10, 0, 0).unwrap()));
        let monday_6pm = Utc
            .from_utc_datetime(&monday_date.and_time(NaiveTime::from_hms_opt(18, 0, 0).unwrap()));

        // January 7, 2024 is a Sunday
        let sunday_date = NaiveDate::from_ymd_opt(2024, 1, 7).unwrap();
        let sunday_10am = Utc
            .from_utc_datetime(&sunday_date.and_time(NaiveTime::from_hms_opt(10, 0, 0).unwrap()));

        // Test allowed time (Monday 10 AM)
        assert!(policy.is_time_allowed(&monday_10am));

        // Test disallowed time (Monday 6 PM - after hours)
        assert!(!policy.is_time_allowed(&monday_6pm));

        // Test disallowed day (Sunday)
        assert!(!policy.is_time_allowed(&sunday_10am));
    }

    #[test]
    fn test_enhanced_user_creation_with_simkari_fields() {
        let satker_code = "KEJATI_DKI_JAKPUS".to_string();
        let nip = "198501012010011001".to_string();
        let nama = "Budi Santoso".to_string();
        let jabatan = "Jaksa Muda".to_string();

        let secreton_policy = SecretonAccessPolicy {
            allowed_satker_secrets: vec![satker_code.clone()],
            access_level: AccessLevel::ReadWrite,
            time_restrictions: None,
            audit_required: true,
            rate_limit: Some(50),
            allowed_paths: None,
            denied_paths: None,
        };

        let security_context = SecurityContext {
            ip_address: Some("192.168.1.100".to_string()),
            user_agent: Some("Mozilla/5.0".to_string()),
            session_id: Some("session_123".to_string()),
            timestamp: Utc::now(),
            risk_score: Some(0.2),
            metadata: Some(json!({"login_method": "password"})),
        };

        let user = User {
            id: Uuid::new_v4(),
            username: "budi.santoso".to_string(),
            email: "budi.santoso@kejaksaan.go.id".to_string(),
            email_verified: true,
            first_name: Some("Budi".to_string()),
            last_name: Some("Santoso".to_string()),
            nip: Some(nip.clone()),
            nama: Some(nama.clone()),
            jabatan: Some(jabatan.clone()),
            satker_code: satker_code.clone(),
            phone_number: Some("+628123456789".to_string()),
            phone_verified: true,
            password_hash: Some("hashed_password".to_string()),
            totp_secret: None,
            totp_backup_codes: None,
            mfa_enabled: false,
            mfa_setup_at: None,
            mfa_last_used: None,
            webauthn_enabled: false,
            account_locked: false,
            account_locked_until: None,
            failed_login_attempts: 0,
            last_login_at: None,
            last_failed_login_at: None,
            password_changed_at: Some(Utc::now()),
            password_expires_at: Some(Utc::now() + Duration::days(90)),
            require_password_change: false,
            realm_id: Some(Uuid::new_v4()),
            organization_id: Some(Uuid::new_v4()),
            roles: vec![],
            permissions: vec![],
            session_data: Some(json!({"last_activity": "document_review"})),
            secreton_access_policy: secreton_policy,
            security_context: security_context,
            attributes: Some(json!({"department": "pidana_umum"})),
            enabled: true,
            federated: false,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            deleted_at: None,
            login_count: 0,
        };

        // Validate SIMKARI-specific fields
        assert_eq!(user.nip, Some(nip));
        assert_eq!(user.nama, Some(nama));
        assert_eq!(user.jabatan, Some(jabatan));
        assert_eq!(user.satker_code, satker_code);
        assert!(user.secreton_access_policy.audit_required);
        assert_eq!(
            user.secreton_access_policy.access_level,
            AccessLevel::ReadWrite
        );
        assert!(user.security_context.risk_score.is_some());
    }

    #[test]
    fn test_role_with_hierarchical_structure() {
        let role_id = Uuid::new_v4();
        let realm_id = Uuid::new_v4();

        let permission = Permission {
            id: Uuid::new_v4(),
            name: "secret.read".to_string(),
            description: Some("Read secrets".to_string()),
            resource_type: "secret".to_string(),
            resource_pattern: Some("/app/config/*".to_string()),
            action: "read".to_string(),
            scope: RoleScope::Satker("KEJATI_DKI_JAKPUS".to_string()),
            conditions: Some(json!({"time_restricted": true})),
            realm_id: Some(realm_id),
            active: true,
            attributes: Some(json!({"priority": "high"})),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let role = Role {
            id: role_id,
            name: "jaksa_satker".to_string(),
            description: Some("Jaksa di tingkat satker".to_string()),
            scope: RoleScope::Satker("KEJATI_DKI_JAKPUS".to_string()),
            permissions: vec![permission],
            managed_by: AdminLevel::AdminSatker("KEJATI_DKI_JAKPUS".to_string()),
            realm_id: Some(realm_id),
            composite: false,
            client_role: false,
            client_id: None,
            priority: 10,
            active: true,
            attributes: Some(json!({"level": "operational"})),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        // Test role assignment validation
        assert!(role.can_assign_to_satker("KEJATI_DKI_JAKPUS"));
        assert!(!role.can_assign_to_satker("KEJATI_JABAR_BANDUNG"));

        // Test access grants
        let target_scope = RoleScope::Satker("KEJATI_DKI_JAKPUS".to_string());
        assert!(role.grants_access_to(&target_scope));

        let other_scope = RoleScope::Satker("KEJATI_JABAR_BANDUNG".to_string());
        assert!(!role.grants_access_to(&other_scope));

        // Test permission filtering
        let secret_permissions = role.get_permissions_for_resource("secret");
        assert_eq!(secret_permissions.len(), 1);
        assert_eq!(secret_permissions[0].action, "read");

        let user_permissions = role.get_permissions_for_resource("user");
        assert_eq!(user_permissions.len(), 0);
    }

    #[test]
    fn test_access_level_ordering() {
        let read_only = AccessLevel::ReadOnly;
        let read_write = AccessLevel::ReadWrite;
        let admin = AccessLevel::Admin;
        let super_admin = AccessLevel::SuperAdmin;

        // Test ordering (higher access levels should be greater)
        assert!(read_only < read_write);
        assert!(read_write < admin);
        assert!(admin < super_admin);

        // Test equality
        assert_eq!(read_only, AccessLevel::ReadOnly);
        assert_ne!(read_only, read_write);
    }

    #[test]
    fn test_security_context_risk_assessment() {
        let low_risk_context = SecurityContext {
            ip_address: Some("192.168.1.100".to_string()), // Internal IP
            user_agent: Some("Mozilla/5.0 (Windows NT 10.0; Win64; x64)".to_string()),
            session_id: Some("session_123".to_string()),
            timestamp: Utc::now(),
            risk_score: Some(0.1), // Low risk
            metadata: Some(json!({
                "login_method": "password_mfa",
                "device_trusted": true,
                "location_verified": true
            })),
        };

        let high_risk_context = SecurityContext {
            ip_address: Some("203.0.113.1".to_string()), // External IP
            user_agent: Some("curl/7.68.0".to_string()), // Suspicious user agent
            session_id: Some("session_456".to_string()),
            timestamp: Utc::now(),
            risk_score: Some(0.8), // High risk
            metadata: Some(json!({
                "login_method": "password_only",
                "device_trusted": false,
                "location_verified": false,
                "suspicious_activity": true
            })),
        };

        // Validate risk scores
        assert!(low_risk_context.risk_score.unwrap() < 0.5);
        assert!(high_risk_context.risk_score.unwrap() > 0.5);

        // Validate metadata structure
        assert!(low_risk_context.metadata.is_some());
        assert!(high_risk_context.metadata.is_some());
    }

    #[test]
    fn test_time_restrictions_validation() {
        let valid_restrictions = TimeRestrictions {
            start_hour: 8,
            end_hour: 17,
            allowed_days: vec![1, 2, 3, 4, 5], // Monday to Friday
            timezone: "Asia/Jakarta".to_string(),
        };

        // Test valid configuration
        assert!(valid_restrictions.start_hour < valid_restrictions.end_hour);
        assert!(valid_restrictions.allowed_days.iter().all(|&day| day <= 6));
        assert!(!valid_restrictions.timezone.is_empty());

        // Test weekend restriction
        let weekend_only = TimeRestrictions {
            start_hour: 0,
            end_hour: 23,
            allowed_days: vec![0, 6], // Sunday and Saturday
            timezone: "Asia/Jakarta".to_string(),
        };

        assert_eq!(weekend_only.allowed_days.len(), 2);
        assert!(weekend_only.allowed_days.contains(&0)); // Sunday
        assert!(weekend_only.allowed_days.contains(&6)); // Saturday
    }

    #[test]
    fn test_user_claims_with_simkari_context() {
        let claims = UserClaims {
            sub: "user_123".to_string(),
            username: "budi.santoso".to_string(),
            email: "budi.santoso@kejaksaan.go.id".to_string(),
            realm_id: "kejaksaan_realm".to_string(),
            roles: vec!["jaksa_satker".to_string(), "document_reviewer".to_string()],
            exp: (Utc::now() + Duration::hours(1)).timestamp() as usize,
            iat: Utc::now().timestamp() as usize,
            iss: "authenc_simkari".to_string(),
        };

        // Validate claims structure
        assert!(claims.sub.starts_with("user_"));
        assert!(claims.email.ends_with("@kejaksaan.go.id"));
        assert_eq!(claims.roles.len(), 2);
        assert!(claims.exp > claims.iat);
        assert_eq!(claims.iss, "authenc_simkari");

        // Test serialization/deserialization
        let json = serde_json::to_string(&claims).unwrap();
        let deserialized: UserClaims = serde_json::from_str(&json).unwrap();
        assert_eq!(claims.sub, deserialized.sub);
        assert_eq!(claims.roles, deserialized.roles);
    }
}
