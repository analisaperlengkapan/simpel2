// Many services and queue processors contain placeholder/stub fields
// with unused variables and dead code that will be cleaned up
// incrementally.  Suppress these crate-wide for now.
#![allow(dead_code)]
#![allow(async_fn_in_trait)]

pub mod audit;
pub mod config;
pub mod email;
pub mod error;
pub mod grpc_service;
pub mod handlers;
pub mod in_app;
pub mod models;
pub mod preferences;
pub mod push;
pub mod queue;
pub mod queue_processor;
pub mod scheduler;
pub mod security;
pub mod sms;
pub mod template;
pub mod websocket;
pub mod whatsapp;
