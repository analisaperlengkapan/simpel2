//! gRPC Client Module for Layanan Portal
//!
//! Provides gRPC clients for communicating with:
//! - Authenc (IAM service) on port 9088
//! - Secreton (Secret Management) on port 9090

pub mod authenc;
pub mod secreton;

// Include generated protobuf code using standard include_proto! macro
// This uses OUT_DIR which is set by tonic_prost_build
pub mod generated {
    pub mod common_v1 {
        tonic::include_proto!("common.v1");
    }

    pub mod authenc_v1 {
        tonic::include_proto!("authenc.v1");
    }

    pub mod secreton_v1 {
        tonic::include_proto!("secreton.v1");
    }
}

// Re-export clients and errors for convenience
pub use authenc::{AuthencClient, AuthencError};
pub use secreton::SecretonClient;

/// gRPC Configuration
#[derive(Debug, Clone)]
pub struct GrpcConfig {
    /// Authenc gRPC endpoint (default: http://authenc:9088)
    pub authenc_url: String,
    /// Secreton gRPC endpoint (default: http://secreton-server:9090)
    pub secreton_url: String,
}

impl Default for GrpcConfig {
    fn default() -> Self {
        Self {
            authenc_url: std::env::var("AUTHENC_GRPC_URL")
                .unwrap_or_else(|_| "http://authenc:9088".to_string()),
            secreton_url: std::env::var("SECRETON_GRPC_URL")
                .unwrap_or_else(|_| "http://secreton-server:9090".to_string()),
        }
    }
}

impl GrpcConfig {
    pub fn from_env() -> Self {
        Self::default()
    }
}
