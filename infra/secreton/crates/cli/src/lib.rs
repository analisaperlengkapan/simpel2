//! Secreton CLI Library
//!
//! Command-line interface for Secreton secret management system.
//!
//! This library provides a comprehensive CLI for interacting with Secreton engine,
//! including authentication, policy management, token management, seal/unseal operations,
//! backup/restore, and audit log access.
//!
//! # Modules
//!
//! - [`audit`] - Audit log viewing and filtering
//! - [`auth`] - Authentication (login/logout)
//! - [`backup`] - Backup and restore operations
//! - [`config`] - Configuration management
//! - [`middleware`] - Pre-operation checks (seal status, initialization)
//! - [`operator`] - Operator diagnostic commands
//! - [`policy`] - Policy management (CRUD, format, validate, test)
//! - [`policy_parser`] - Policy file parsing (TOML format)
//! - [`seal`] - Seal/unseal operations
//! - [`token`] - Token lifecycle management
//! - [`token_store`] - Persistent token storage
//!
//! # Examples
//!
//! ## Basic Usage
//!
//! ```no_run
//! use secreton_cli::config::CliConfig;
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     // Load configuration
//!     let config = CliConfig::load_default().await?;
//!
//!     // Use the CLI modules
//!     // ...
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Authentication
//!
//! ```no_run
//! use secreton_cli::auth::login_command;
//! use secreton_cli::config::CliConfig;
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     let config = CliConfig::load_default().await?;
//!
//!     // Login with username/password
//!     login_command(&config, "userpass", Some("admin".to_string()), None).await?;
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Policy Management
//!
//! ```no_run
//! use secreton_cli::policy::{PolicyCommand, execute_policy_command};
//! use secreton_cli::config::CliConfig;
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     let config = CliConfig::load_default().await?;
//!
//!     // List policies
//!     let cmd = PolicyCommand::List {
//!         namespace: None,
//!         format: "table".to_string(),
//!     };
//!     execute_policy_command(cmd, &config, None).await?;
//!
//!     Ok(())
//! }
//! ```

/// Audit log viewing and filtering commands
pub mod audit;

/// Authentication commands (login/logout)
pub mod auth;

/// Backup and restore operations
pub mod backup;

/// Configuration management
pub mod config;

/// Pre-operation middleware (seal status checks)
pub mod middleware;

/// Operator diagnostic commands
pub mod operator;

/// Policy management commands (CRUD, format, validate, test)
pub mod policy;

/// Policy file parsing (TOML format support)
pub mod policy_parser;

/// Seal/unseal operations
pub mod seal;

/// Token lifecycle management commands
pub mod token;

/// Persistent token storage
pub mod token_store;
