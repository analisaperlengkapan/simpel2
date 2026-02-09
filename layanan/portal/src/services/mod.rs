//! Service layer - gRPC clients and business logic

pub mod authenc_client;
pub mod secreton_client;

pub use authenc_client::{AuthencClient, AuthencError};
pub use secreton_client::SecretonClient;
