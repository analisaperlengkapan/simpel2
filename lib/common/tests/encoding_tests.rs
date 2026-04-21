//! Unit tests for encoding module

#[cfg(test)]
mod encoding_tests {
    use base64::{Engine as _, engine::general_purpose};

    #[test]
    fn test_base64_encode_decode() {
        let original = "Hello, World!";
        let encoded = general_purpose::STANDARD.encode(original.as_bytes());
        let decoded = general_purpose::STANDARD.decode(&encoded).unwrap();
        let decoded_str = String::from_utf8(decoded).unwrap();

        assert_eq!(original, decoded_str);
    }

    #[test]
    fn test_base64_url_safe() {
        let data = "test+data/with=special";
        let encoded = general_purpose::URL_SAFE.encode(data.as_bytes());

        assert!(!encoded.contains('+'));
        assert!(!encoded.contains('/'));
        assert!(!encoded.contains('='));
    }

    #[test]
    fn test_hex_encoding() {
        let data = b"test data";
        let hex = hex::encode(data);
        let decoded = hex::decode(&hex).unwrap();

        assert_eq!(data.to_vec(), decoded);
    }

    #[test]
    fn test_json_serialization() {
        use serde::{Serialize, Deserialize};

        #[derive(Serialize, Deserialize, PartialEq, Debug)]
        struct TestData {
            id: u32,
            name: String,
        }

        let data = TestData {
            id: 1,
            name: "test".to_string(),
        };

        let json = serde_json::to_string(&data).unwrap();
        let deserialized: TestData = serde_json::from_str(&json).unwrap();

        assert_eq!(data, deserialized);
    }
}
