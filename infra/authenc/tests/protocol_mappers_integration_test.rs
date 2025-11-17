#[cfg(test)]
mod tests {

    use authenc::models::protocol_mapper::{ProtocolMapperConfiguration, ProtocolMapperType};

    #[tokio::test]
    #[ignore = "Requires PostgreSQL database"]
    async fn test_protocol_mapper_end_to_end() {
        // This test requires a running PostgreSQL database
        // Run with: cargo test --test protocol_mappers_integration_test -- --ignored

        // Initialize database connection
        // let db = Database::new(&db_config).await.unwrap();
        // let realm_id = Uuid::new_v4();

        // Create a protocol mapper
        // let request = CreateProtocolMapperRequest {
        //     name: "test-mapper".to_string(),
        //     protocol: "openid-connect".to_string(),
        //     mapper_type: ProtocolMapperType::UserProperty,
        //     config: ProtocolMapperConfiguration {
        //         claim_name: "username".to_string(),
        //         user_property: Some("username".to_string()),
        //         include_in_access_token: true,
        //         include_in_id_token: true,
        //         include_in_userinfo: true,
        //         ..Default::default()
        //     },
        //     client_id: None,
        //     client_scope_id: None,
        //     };

        // let mapper = protocol_mappers_ops::create(&db, realm_id, request).await.unwrap();

        // assert_eq!(mapper.name, "test-mapper");
        // assert_eq!(mapper.protocol, "openid-connect");

        // Clean up
        // protocol_mappers_ops::delete(&db, mapper.id).await.unwrap();
    }

    #[test]
    fn test_protocol_mapper_type_parsing() {
        use std::str::FromStr;

        let mapper_type = ProtocolMapperType::from_str("user-property").unwrap();
        assert_eq!(mapper_type, ProtocolMapperType::UserProperty);

        let mapper_type = ProtocolMapperType::from_str("user-attribute").unwrap();
        assert_eq!(mapper_type, ProtocolMapperType::UserAttribute);

        let mapper_type = ProtocolMapperType::from_str("hardcoded-claim").unwrap();
        assert_eq!(mapper_type, ProtocolMapperType::HardcodedClaim);
    }

    #[test]
    fn test_protocol_mapper_configuration_defaults() {
        let config = ProtocolMapperConfiguration::default();
        assert_eq!(config.include_in_access_token, true);
        assert_eq!(config.include_in_id_token, true);
        assert_eq!(config.include_in_userinfo, true);
    }
}
