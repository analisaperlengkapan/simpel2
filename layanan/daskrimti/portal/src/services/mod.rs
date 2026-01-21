//! Service layer - gRPC clients and business logic

mod authenc_client;
mod secreton_client;

pub use authenc_client::AuthencClient;
pub use secreton_client::SecretonClient;
