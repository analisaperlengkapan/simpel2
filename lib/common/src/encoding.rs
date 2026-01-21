//! Encoding utilities for standardizing data formats
//!
//! This module provides standardized base64 and hex encoding/decoding functions
//! using the modern Engine-based API for base64.

use base64::{engine::general_purpose, Engine as _};

/// Encode bytes to base64 string using standard encoding
#[inline]
pub fn base64_encode<T: AsRef<[u8]>>(data: T) -> String {
    general_purpose::STANDARD.encode(data)
}

/// Decode base64 string to bytes using standard encoding
#[inline]
pub fn base64_decode<T: AsRef<[u8]>>(data: T) -> Result<Vec<u8>, base64::DecodeError> {
    general_purpose::STANDARD.decode(data)
}

/// Encode bytes to URL-safe base64 string (no padding)
#[inline]
pub fn base64_encode_url<T: AsRef<[u8]>>(data: T) -> String {
    general_purpose::URL_SAFE_NO_PAD.encode(data)
}

/// Decode URL-safe base64 string to bytes
#[inline]
pub fn base64_decode_url<T: AsRef<[u8]>>(data: T) -> Result<Vec<u8>, base64::DecodeError> {
    general_purpose::URL_SAFE_NO_PAD.decode(data)
}

/// Encode bytes to hex string
#[inline]
pub fn hex_encode<T: AsRef<[u8]>>(data: T) -> String {
    hex::encode(data)
}

/// Decode hex string to bytes
#[inline]
pub fn hex_decode<T: AsRef<[u8]>>(data: T) -> Result<Vec<u8>, hex::FromHexError> {
    hex::decode(data)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_base64_roundtrip() {
        let original = b"hello world";
        let encoded = base64_encode(original);
        let decoded = base64_decode(&encoded).unwrap();
        assert_eq!(original, decoded.as_slice());
    }

    #[test]
    fn test_base64_url_roundtrip() {
        let original = b"hello world!@#$%";
        let encoded = base64_encode_url(original);
        let decoded = base64_decode_url(&encoded).unwrap();
        assert_eq!(original, decoded.as_slice());
    }

    #[test]
    fn test_hex_roundtrip() {
        let original = b"hello world";
        let encoded = hex_encode(original);
        let decoded = hex_decode(&encoded).unwrap();
        assert_eq!(original, decoded.as_slice());
    }
}
