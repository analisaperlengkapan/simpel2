//! Encoding utilities for Secreton
//!
//! This module provides standardized base64 encoding/decoding functions
//! using the modern Engine-based API.

use base64::{engine::general_purpose, Engine as _};

/// Encode bytes to base64 string using standard encoding
///
/// # Arguments
/// * `data` - The bytes to encode
///
/// # Returns
/// Base64 encoded string
#[inline]
pub fn base64_encode<T: AsRef<[u8]>>(data: T) -> String {
    general_purpose::STANDARD.encode(data)
}

/// Decode base64 string to bytes using standard encoding
///
/// # Arguments
/// * `data` - The base64 string to decode
///
/// # Returns
/// Result containing decoded bytes or an error
#[inline]
pub fn base64_decode<T: AsRef<[u8]>>(data: T) -> Result<Vec<u8>, base64::DecodeError> {
    general_purpose::STANDARD.decode(data)
}

/// Encode bytes to URL-safe base64 string (no padding)
///
/// # Arguments
/// * `data` - The bytes to encode
///
/// # Returns
/// URL-safe base64 encoded string without padding
#[inline]
pub fn base64_encode_url<T: AsRef<[u8]>>(data: T) -> String {
    general_purpose::URL_SAFE_NO_PAD.encode(data)
}

/// Decode URL-safe base64 string to bytes
///
/// # Arguments
/// * `data` - The URL-safe base64 string to decode
///
/// # Returns
/// Result containing decoded bytes or an error
#[inline]
pub fn base64_decode_url<T: AsRef<[u8]>>(data: T) -> Result<Vec<u8>, base64::DecodeError> {
    general_purpose::URL_SAFE_NO_PAD.decode(data)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_base64_roundtrip() {
        let original = b"hello world";
        let encoded = base64_encode(original);
        let decoded = base64_decode(&encoded).unwrap();
        assert_eq!(original.as_slice(), decoded.as_slice());
    }

    #[test]
    fn test_base64_url_roundtrip() {
        let original = b"hello world!@#$%";
        let encoded = base64_encode_url(original);
        let decoded = base64_decode_url(&encoded).unwrap();
        assert_eq!(original.as_slice(), decoded.as_slice());
    }

    #[test]
    fn test_base64_standard_encoding() {
        let data = b"test";
        let encoded = base64_encode(data);
        assert_eq!(encoded, "dGVzdA==");
    }

    #[test]
    fn test_base64_decode_error() {
        let result = base64_decode("!!!invalid!!!");
        assert!(result.is_err());
    }
}
