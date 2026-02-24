//! Integration and unit tests for authenc-storage crate
//!
//! This module contains comprehensive tests for:
//! - Database connection pooling
//! - Prepared statement caching
//! - Transaction handling (commit/rollback)
//! - All store implementations (User, Session, Realm, Client)
//! - Error handling and edge cases
//!
//! ## Test Organization
//!
//! - `database_tests.rs` - Database infrastructure tests
//! - `user_store_tests.rs` - UserStore implementation tests
//! - `session_store_tests.rs` - SessionStore implementation tests
//! - `realm_store_tests.rs` - RealmStore implementation tests
//! - `client_store_tests.rs` - ClientStore implementation tests
//! - `transaction_tests.rs` - Transaction handling tests
//!
//! ## Running Tests
//!
//! ```bash
//! # Run all storage tests
//! cargo test -p authenc-storage
//!
//! # Run specific test module
//! cargo test -p authenc-storage database_tests
//!
//! # Run with output
//! cargo test -p authenc-storage -- --nocapture
//! ```

mod client_store_tests;
mod database_tests;
mod realm_store_tests;
mod session_store_tests;
mod transaction_tests;
mod user_store_tests;
