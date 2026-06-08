//! gRPC subsystem: clients to external sibling services
//! (authenc, secreton, integrasi). Internal gRPC servers were dropped
//! during the trait-wiring commit — workflow now talks to dokumen /
//! notifikasi through the `crate::contracts::*` traits, not
//! gRPC.

pub mod clients;

pub use clients::{AuthencClient, IntegrasiClient, SecretonClient};
