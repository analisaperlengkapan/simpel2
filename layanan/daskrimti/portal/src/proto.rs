//! Generated gRPC proto types
//!
//! This module includes all generated protobuf types in a central location
//! to ensure proper module resolution between authenc, secreton, and common types.

// Common types (shared between authenc and secreton)
pub mod common {
    pub mod v1 {
        tonic::include_proto!("common.v1");
    }
}

// Authenc authentication service types
pub mod authenc {
    pub mod v1 {
        tonic::include_proto!("authenc.v1");
    }
}

// Secreton secret management service types
pub mod secreton {
    pub mod v1 {
        tonic::include_proto!("secreton.v1");
    }
}
