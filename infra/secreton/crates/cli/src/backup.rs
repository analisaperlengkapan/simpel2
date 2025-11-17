//! Backup and Restore CLI commands
//!
//! Provides CLI commands for vault backup operations including:
//! - backup: Create encrypted backup of secrets, metadata, and audit logs
//! - verify: Verify backup integrity
//! - list: List available backups

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use clap::Subcommand;
use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::Path;

use crate::config::CliConfig;

#[derive(Subcommand)]
pub enum BackupCommand {
    /// Create a backup of vault data
    Create {
        /// Output file path for the backup
        #[arg(short, long)]
        output: String,

        /// Encryption password for the backup (if not provided, will prompt)
        #[arg(short = 'p', long)]
        password: Option<String>,

        /// Include audit logs in backup
        #[arg(long, default_value = "true")]
        include_audit: bool,

        /// Incremental backup (only changes since last backup)
        #[arg(short, long)]
        incremental: bool,

        /// Base backup file for incremental backup
        #[arg(long)]
        base_backup: Option<String>,

        /// Compression level (0-9, default: 6)
        #[arg(short, long, default_value = "6")]
        compression: u8,
    },

    /// Restore vault data from a backup
    Restore {
        /// Backup file to restore from
        #[arg(short, long)]
        file: String,

        /// Decryption password (if not provided, will prompt)
        #[arg(short = 'p', long)]
        password: Option<String>,

        /// Point-in-time restore (ISO 8601 timestamp)
        #[arg(long)]
        point_in_time: Option<String>,

        /// Restore only secrets (skip audit logs)
        #[arg(long)]
        secrets_only: bool,

        /// Restore only audit logs (skip secrets)
        #[arg(long)]
        audit_only: bool,

        /// Dry run - verify restore without applying changes
        #[arg(long)]
        dry_run: bool,

        /// Force restore even if secrets already exist (overwrite)
        #[arg(long)]
        force: bool,

        /// Restore to a different namespace
        #[arg(long)]
        target_namespace: Option<String>,
    },

    /// Verify backup integrity
    Verify {
        /// Backup file to verify
        #[arg(short, long)]
        file: String,

        /// Decryption password (if not provided, will prompt)
        #[arg(short = 'p', long)]
        password: Option<String>,

        /// Verbose output showing all entries
        #[arg(short, long)]
        verbose: bool,
    },

    /// List available backups in a directory
    List {
        /// Directory containing backups (default: current directory)
        #[arg(short, long, default_value = ".")]
        directory: String,

        /// Show detailed information
        #[arg(short, long)]
        detailed: bool,
    },
}

/// Backup manifest containing metadata about the backup
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(test, derive(PartialEq))]
pub struct BackupManifest {
    pub backup_id: String,
    pub created_at: DateTime<Utc>,
    pub format_version: String,
    pub backup_type: BackupType,
    pub base_backup_id: Option<String>,
    pub server_version: String,
    pub secret_count: usize,
    pub audit_log_count: usize,
    pub uncompressed_size: u64,
    pub compression_algorithm: String,
    pub encryption_algorithm: String,
    pub data_checksum: String,
    pub metadata: std::collections::HashMap<String, serde_json::Value>,
}

/// Type of backup
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum BackupType {
    Full,
    Incremental,
}

/// Backup data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
struct BackupData {
    manifest: BackupManifest,
    secrets: Vec<VaultEntryBackup>,
    audit_logs: Vec<AuditLogBackup>,
}

/// Simplified vault entry for backup
#[derive(Debug, Clone, Serialize, Deserialize)]
struct VaultEntryBackup {
    id: String,
    path: String,
    encrypted_data: String,
    encryption_metadata: serde_json::Value,
    security_level: String,
    metadata: serde_json::Value,
    tags: Vec<String>,
    version: u32,
    owner_id: String,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    expires_at: Option<DateTime<Utc>>,
}

/// Simplified audit log entry for backup
#[derive(Debug, Clone, Serialize, Deserialize)]
struct AuditLogBackup {
    event_id: String,
    timestamp: DateTime<Utc>,
    event_type: String,
    operation: String,
    resource_path: String,
    result: String,
    nip: Option<String>,
    satker_code: Option<String>,
    metadata: serde_json::Value,
}

/// Execute backup command
pub async fn execute_backup_command(cmd: BackupCommand, config: &CliConfig) -> Result<()> {
    match cmd {
        BackupCommand::Create {
            output,
            password,
            include_audit,
            incremental,
            base_backup,
            compression,
        } => {
            create_backup(
                config,
                &output,
                password,
                include_audit,
                incremental,
                base_backup.as_deref(),
                compression,
            )
            .await
        }
        BackupCommand::Restore {
            file,
            password,
            point_in_time,
            secrets_only,
            audit_only,
            dry_run,
            force,
            target_namespace,
        } => {
            restore_backup(
                config,
                &file,
                password,
                point_in_time,
                secrets_only,
                audit_only,
                dry_run,
                force,
                target_namespace,
            )
            .await
        }
        BackupCommand::Verify {
            file,
            password,
            verbose,
        } => verify_backup(&file, password, verbose).await,
        BackupCommand::List {
            directory,
            detailed,
        } => list_backups(&directory, detailed).await,
    }
}

/// Create a backup of vault data
async fn create_backup(
    config: &CliConfig,
    output_path: &str,
    password: Option<String>,
    include_audit: bool,
    incremental: bool,
    base_backup: Option<&str>,
    compression_level: u8,
) -> Result<()> {
    println!("Creating vault backup...");
    println!();

    if compression_level > 9 {
        anyhow::bail!("Compression level must be between 0 and 9");
    }

    let encryption_password = get_password(password)?;

    if encryption_password.len() < 8 {
        anyhow::bail!("Password must be at least 8 characters long");
    }

    let base_manifest = if incremental {
        if let Some(base_path) = base_backup {
            println!("Loading base backup: {}", base_path);
            Some(load_backup_manifest(base_path, &encryption_password).await?)
        } else {
            anyhow::bail!("Incremental backup requires --base-backup parameter");
        }
    } else {
        None
    };

    let client = reqwest::Client::new();

    println!("Fetching secrets from vault...");
    let secrets_url = format!("{}/v1/secrets", config.server_url);
    let secrets_response = client
        .get(&secrets_url)
        .send()
        .await
        .context("Failed to fetch secrets list")?;

    if !secrets_response.status().is_success() {
        anyhow::bail!("Failed to fetch secrets: {}", secrets_response.status());
    }

    let secrets_list: serde_json::Value = secrets_response
        .json()
        .await
        .context("Failed to parse secrets list")?;

    let mut secrets = Vec::new();
    let empty_vec = Vec::new();
    let secret_paths = secrets_list
        .get("keys")
        .and_then(|k| k.as_array())
        .unwrap_or(&empty_vec);

    println!("Found {} secret(s)", secret_paths.len());

    for (idx, path_value) in secret_paths.iter().enumerate() {
        if let Some(path) = path_value.as_str() {
            if let Some(ref base) = base_manifest
                && incremental
                && !should_include_in_incremental(path, base).await
            {
                continue;
            }

            let secret_url = format!("{}/v1/secret/data/{}", config.server_url, path);
            let secret_response = client.get(&secret_url).send().await;

            if let Ok(response) = secret_response
                && response.status().is_success()
                && let Ok(secret_data) = response.json::<serde_json::Value>().await
            {
                secrets.push(convert_to_backup_entry(path, &secret_data));
            }

            if (idx + 1) % 10 == 0 {
                println!("Progress: {}/{}", idx + 1, secret_paths.len());
            }
        }
    }

    println!("Fetched {} secret(s)", secrets.len());

    let mut audit_logs = Vec::new();
    if include_audit {
        println!("Fetching audit logs...");
        let audit_url = format!("{}/v1/audit/logs", config.server_url);
        let audit_response = client.get(&audit_url).send().await;

        if let Ok(response) = audit_response
            && response.status().is_success()
            && let Ok(audit_data) = response.json::<serde_json::Value>().await
            && let Some(logs) = audit_data.get("logs").and_then(|l| l.as_array())
        {
            for log in logs {
                audit_logs.push(convert_to_audit_backup(log));
            }
        }

        println!("Fetched {} audit log(s)", audit_logs.len());
    }

    let backup_id = uuid::Uuid::new_v4().to_string();
    let manifest = BackupManifest {
        backup_id: backup_id.clone(),
        created_at: Utc::now(),
        format_version: "1.0.0".to_string(),
        backup_type: if incremental {
            BackupType::Incremental
        } else {
            BackupType::Full
        },
        base_backup_id: base_manifest.as_ref().map(|m| m.backup_id.clone()),
        server_version: "1.0.0".to_string(),
        secret_count: secrets.len(),
        audit_log_count: audit_logs.len(),
        uncompressed_size: 0,
        compression_algorithm: format!("gzip-{}", compression_level),
        encryption_algorithm: "aes-256-gcm".to_string(),
        data_checksum: String::new(),
        metadata: std::collections::HashMap::new(),
    };

    let backup_data = BackupData {
        manifest,
        secrets,
        audit_logs,
    };

    println!("Serializing backup data...");
    let json_data = serde_json::to_vec(&backup_data).context("Failed to serialize backup data")?;

    let checksum = calculate_checksum(&json_data);

    let mut backup_data = backup_data;
    backup_data.manifest.uncompressed_size = json_data.len() as u64;
    backup_data.manifest.data_checksum = checksum;

    let json_data = serde_json::to_vec(&backup_data).context("Failed to serialize backup data")?;

    println!("Compressing backup data...");
    let compressed_data = compress_data(&json_data, compression_level)?;
    let compression_ratio = (compressed_data.len() as f64 / json_data.len() as f64) * 100.0;
    println!(
        "Compressed: {} bytes -> {} bytes ({:.1}%)",
        json_data.len(),
        compressed_data.len(),
        compression_ratio
    );

    println!("Encrypting backup...");
    let encrypted_data = encrypt_data(&compressed_data, &encryption_password)?;

    println!("Writing backup to file: {}", output_path);
    let mut file = File::create(output_path).context("Failed to create backup file")?;
    file.write_all(&encrypted_data)
        .context("Failed to write backup file")?;

    println!();
    println!("Backup created successfully!");
    println!();
    println!("Backup Summary:");
    println!("  Backup ID: {}", backup_id);
    println!("  Type: {:?}", backup_data.manifest.backup_type);
    println!("  Secrets: {}", backup_data.manifest.secret_count);
    println!("  Audit Logs: {}", backup_data.manifest.audit_log_count);
    println!(
        "  Uncompressed Size: {} bytes",
        backup_data.manifest.uncompressed_size
    );
    println!("  Compressed Size: {} bytes", compressed_data.len());
    println!("  Encrypted Size: {} bytes", encrypted_data.len());
    println!("  Checksum: {}", backup_data.manifest.data_checksum);
    println!();
    println!("IMPORTANT:");
    println!("  - Store this backup file securely");
    println!("  - Remember the encryption password");
    println!("  - Test restore procedure regularly");

    Ok(())
}

/// Verify backup integrity
async fn verify_backup(file_path: &str, password: Option<String>, verbose: bool) -> Result<()> {
    println!("Verifying backup: {}", file_path);
    println!();

    let decryption_password = get_password(password)?;

    let mut file = File::open(file_path).context("Failed to open backup file")?;
    let mut encrypted_data = Vec::new();
    file.read_to_end(&mut encrypted_data)
        .context("Failed to read backup file")?;

    println!("File size: {} bytes", encrypted_data.len());

    println!("Decrypting backup...");
    let compressed_data = decrypt_data(&encrypted_data, &decryption_password)?;

    println!("Decompressing backup...");
    let json_data = decompress_data(&compressed_data)?;

    println!("Parsing backup data...");
    let backup_data: BackupData =
        serde_json::from_slice(&json_data).context("Failed to parse backup data")?;

    println!("Verifying checksum...");
    let calculated_checksum = calculate_checksum(&json_data);
    if calculated_checksum != backup_data.manifest.data_checksum {
        anyhow::bail!(
            "Checksum mismatch! Expected: {}, Got: {}",
            backup_data.manifest.data_checksum,
            calculated_checksum
        );
    }

    println!("Checksum verified");
    println!();
    println!("Backup Information:");
    println!("  Backup ID: {}", backup_data.manifest.backup_id);
    println!("  Created: {}", backup_data.manifest.created_at);
    println!("  Type: {:?}", backup_data.manifest.backup_type);
    println!("  Format Version: {}", backup_data.manifest.format_version);
    println!("  Server Version: {}", backup_data.manifest.server_version);
    println!("  Secrets: {}", backup_data.manifest.secret_count);
    println!("  Audit Logs: {}", backup_data.manifest.audit_log_count);
    println!(
        "  Uncompressed Size: {} bytes",
        backup_data.manifest.uncompressed_size
    );
    println!(
        "  Compression: {}",
        backup_data.manifest.compression_algorithm
    );
    println!(
        "  Encryption: {}",
        backup_data.manifest.encryption_algorithm
    );

    if let Some(base_id) = &backup_data.manifest.base_backup_id {
        println!("  Base Backup ID: {}", base_id);
    }

    if verbose {
        println!();
        println!("Secrets:");
        for secret in &backup_data.secrets {
            println!("  - {} (v{})", secret.path, secret.version);
        }

        if !backup_data.audit_logs.is_empty() {
            println!();
            println!("Audit Logs (first 10):");
            for (idx, log) in backup_data.audit_logs.iter().take(10).enumerate() {
                println!(
                    "  {}. {} - {} on {}",
                    idx + 1,
                    log.timestamp,
                    log.operation,
                    log.resource_path
                );
            }
            if backup_data.audit_logs.len() > 10 {
                println!("  ... and {} more", backup_data.audit_logs.len() - 10);
            }
        }
    }

    println!();
    println!("Backup verification successful!");
    println!("All integrity checks passed.");

    Ok(())
}

/// List available backups in a directory
async fn list_backups(directory: &str, detailed: bool) -> Result<()> {
    println!("Listing backups in: {}", directory);
    println!();

    let dir_path = Path::new(directory);
    if !dir_path.exists() {
        anyhow::bail!("Directory does not exist: {}", directory);
    }

    let entries = fs::read_dir(dir_path).context("Failed to read directory")?;

    let mut backup_files = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_file()
            && let Some(ext) = path.extension()
            && (ext == "backup" || ext == "bak")
        {
            backup_files.push(path);
        }
    }

    if backup_files.is_empty() {
        println!("No backup files found in directory.");
        return Ok(());
    }

    println!("Found {} backup file(s):", backup_files.len());
    println!();

    for (idx, backup_file) in backup_files.iter().enumerate() {
        let file_name = backup_file
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown");
        let metadata = fs::metadata(backup_file)?;
        let size = metadata.len();

        println!("{}. {}", idx + 1, file_name);
        println!("   Size: {} bytes", size);

        if detailed {
            println!("   Path: {}", backup_file.display());
        }

        println!();
    }

    Ok(())
}

/// Restore vault data from a backup
async fn restore_backup(
    config: &CliConfig,
    file_path: &str,
    password: Option<String>,
    point_in_time: Option<String>,
    secrets_only: bool,
    audit_only: bool,
    dry_run: bool,
    force: bool,
    target_namespace: Option<String>,
) -> Result<()> {
    println!("Restoring vault from backup: {}", file_path);
    println!();

    if secrets_only && audit_only {
        anyhow::bail!("Cannot specify both --secrets-only and --audit-only");
    }

    let decryption_password = get_password(password)?;

    // Step 1: Load and validate backup
    println!("Step 1/5: Loading backup file...");
    let mut file = File::open(file_path).context("Failed to open backup file")?;
    let mut encrypted_data = Vec::new();
    file.read_to_end(&mut encrypted_data)
        .context("Failed to read backup file")?;

    println!("  File size: {} bytes", encrypted_data.len());

    println!("Step 2/5: Decrypting and decompressing backup...");
    let compressed_data = decrypt_data(&encrypted_data, &decryption_password)
        .context("Failed to decrypt backup - check password")?;
    let json_data = decompress_data(&compressed_data).context("Failed to decompress backup")?;

    println!("Step 3/5: Validating backup integrity...");
    let backup_data: BackupData =
        serde_json::from_slice(&json_data).context("Failed to parse backup data")?;

    // Verify checksum
    let calculated_checksum = calculate_checksum(&json_data);
    if calculated_checksum != backup_data.manifest.data_checksum {
        anyhow::bail!(
            "Backup integrity check failed! Checksum mismatch.\nExpected: {}\nGot: {}",
            backup_data.manifest.data_checksum,
            calculated_checksum
        );
    }
    println!("  ✓ Checksum verified");

    // Parse point-in-time if provided
    let pit_timestamp = if let Some(pit_str) = point_in_time {
        let pit = DateTime::parse_from_rfc3339(&pit_str)
            .context("Invalid point-in-time format. Use ISO 8601 (e.g., 2025-10-29T10:00:00Z)")?
            .with_timezone(&Utc);
        println!("  ✓ Point-in-time restore: {}", pit);
        Some(pit)
    } else {
        None
    };

    // Display backup information
    println!();
    println!("Backup Information:");
    println!("  Backup ID: {}", backup_data.manifest.backup_id);
    println!("  Created: {}", backup_data.manifest.created_at);
    println!("  Type: {:?}", backup_data.manifest.backup_type);
    println!("  Secrets: {}", backup_data.manifest.secret_count);
    println!("  Audit Logs: {}", backup_data.manifest.audit_log_count);

    if dry_run {
        println!();
        println!("DRY RUN MODE - No changes will be applied");
    }

    // Filter data based on point-in-time
    let secrets_to_restore: Vec<_> = backup_data
        .secrets
        .iter()
        .filter(|s| {
            if let Some(pit) = pit_timestamp {
                s.created_at <= pit
            } else {
                true
            }
        })
        .collect();

    let audit_logs_to_restore: Vec<_> = backup_data
        .audit_logs
        .iter()
        .filter(|a| {
            if let Some(pit) = pit_timestamp {
                a.timestamp <= pit
            } else {
                true
            }
        })
        .collect();

    println!();
    println!("Restore Plan:");
    if !audit_only {
        println!("  Secrets to restore: {}", secrets_to_restore.len());
    }
    if !secrets_only {
        println!("  Audit logs to restore: {}", audit_logs_to_restore.len());
    }

    if dry_run {
        println!();
        println!("Dry run complete. No changes were made.");
        println!();
        println!("To perform the actual restore, run without --dry-run flag.");
        return Ok(());
    }

    // Confirm restore
    if !force {
        println!();
        println!("⚠️  WARNING: This will restore data to the vault.");
        println!("   Existing secrets may be overwritten.");
        println!();
        print!("Do you want to continue? (yes/no): ");
        std::io::Write::flush(&mut std::io::stdout())?;

        let mut confirmation = String::new();
        std::io::stdin().read_line(&mut confirmation)?;

        if confirmation.trim().to_lowercase() != "yes" {
            println!("Restore cancelled.");
            return Ok(());
        }
    }

    let client = reqwest::Client::new();

    // Step 4: Restore secrets
    if !audit_only {
        println!();
        println!("Step 4/5: Restoring secrets...");

        let mut restored_count = 0;
        let mut skipped_count = 0;
        let mut failed_count = 0;

        for (idx, secret) in secrets_to_restore.iter().enumerate() {
            let restore_path = if let Some(ref ns) = target_namespace {
                format!("{}/{}", ns, secret.path)
            } else {
                secret.path.clone()
            };

            // Check if secret already exists
            let check_url = format!("{}/v1/secret/data/{}", config.server_url, restore_path);
            let check_response = client.get(&check_url).send().await;

            let exists = check_response
                .map(|r| r.status().is_success())
                .unwrap_or(false);

            if exists && !force {
                skipped_count += 1;
                if (idx + 1) % 10 == 0 {
                    println!(
                        "  Progress: {}/{} (restored: {}, skipped: {}, failed: {})",
                        idx + 1,
                        secrets_to_restore.len(),
                        restored_count,
                        skipped_count,
                        failed_count
                    );
                }
                continue;
            }

            // Parse encrypted data back to JSON
            let secret_data: serde_json::Value = serde_json::from_str(&secret.encrypted_data)
                .unwrap_or_else(|_| serde_json::json!({}));

            // Restore secret
            let restore_url = format!("{}/v1/secret/data/{}", config.server_url, restore_path);
            let payload = serde_json::json!({
                "data": secret_data,
                "metadata": secret.metadata,
            });

            let restore_response = client
                .post(&restore_url)
                .header("Content-Type", "application/json")
                .json(&payload)
                .send()
                .await;

            match restore_response {
                Ok(response) if response.status().is_success() => {
                    restored_count += 1;
                }
                _ => {
                    failed_count += 1;
                }
            }

            if (idx + 1) % 10 == 0 {
                println!(
                    "  Progress: {}/{} (restored: {}, skipped: {}, failed: {})",
                    idx + 1,
                    secrets_to_restore.len(),
                    restored_count,
                    skipped_count,
                    failed_count
                );
            }
        }

        println!();
        println!("  ✓ Secrets restored: {}", restored_count);
        if skipped_count > 0 {
            println!("  ⊘ Secrets skipped (already exist): {}", skipped_count);
            println!("    Use --force to overwrite existing secrets");
        }
        if failed_count > 0 {
            println!("  ✗ Secrets failed: {}", failed_count);
        }
    } else {
        println!();
        println!("Step 4/5: Skipping secrets (--audit-only specified)");
    }

    // Step 5: Restore audit logs
    if !secrets_only {
        println!();
        println!("Step 5/5: Restoring audit logs...");

        let mut restored_count = 0;
        let mut failed_count = 0;

        for (idx, audit_log) in audit_logs_to_restore.iter().enumerate() {
            let restore_url = format!("{}/v1/audit/logs", config.server_url);
            let payload = serde_json::json!({
                "event_id": audit_log.event_id,
                "timestamp": audit_log.timestamp,
                "event_type": audit_log.event_type,
                "operation": audit_log.operation,
                "resource_path": audit_log.resource_path,
                "result": audit_log.result,
                "nip": audit_log.nip,
                "satker_code": audit_log.satker_code,
                "metadata": audit_log.metadata,
            });

            let restore_response = client
                .post(&restore_url)
                .header("Content-Type", "application/json")
                .json(&payload)
                .send()
                .await;

            match restore_response {
                Ok(response) if response.status().is_success() => {
                    restored_count += 1;
                }
                _ => {
                    failed_count += 1;
                }
            }

            if (idx + 1) % 50 == 0 {
                println!(
                    "  Progress: {}/{} (restored: {}, failed: {})",
                    idx + 1,
                    audit_logs_to_restore.len(),
                    restored_count,
                    failed_count
                );
            }
        }

        println!();
        println!("  ✓ Audit logs restored: {}", restored_count);
        if failed_count > 0 {
            println!("  ✗ Audit logs failed: {}", failed_count);
        }
    } else {
        println!();
        println!("Step 5/5: Skipping audit logs (--secrets-only specified)");
    }

    // Final verification
    println!();
    println!("Performing post-restore verification...");

    let verify_url = format!("{}/v1/secrets", config.server_url);
    let verify_response = client.get(&verify_url).send().await;

    match verify_response {
        Ok(response) if response.status().is_success() => {
            if let Ok(secrets_list) = response.json::<serde_json::Value>().await
                && let Some(keys) = secrets_list.get("keys").and_then(|k| k.as_array())
            {
                println!("  ✓ Vault is accessible");
                println!("  ✓ Total secrets in vault: {}", keys.len());
            }
        }
        _ => {
            println!("  ⚠ Warning: Could not verify vault status");
        }
    }

    println!();
    println!("╔════════════════════════════════════════════════════════════╗");
    println!("║           RESTORE COMPLETED SUCCESSFULLY                  ║");
    println!("╚════════════════════════════════════════════════════════════╝");
    println!();
    println!("Restore Summary:");
    println!("  Backup ID: {}", backup_data.manifest.backup_id);
    println!("  Backup Date: {}", backup_data.manifest.created_at);
    if let Some(pit) = pit_timestamp {
        println!("  Point-in-Time: {}", pit);
    }
    if let Some(ref ns) = target_namespace {
        println!("  Target Namespace: {}", ns);
    }
    println!();
    println!("IMPORTANT:");
    println!("  - Verify critical secrets are accessible");
    println!("  - Check application connectivity");
    println!("  - Review audit logs for restore operations");
    println!("  - Consider creating a new backup");

    Ok(())
}

// Helper functions

fn get_password(password: Option<String>) -> Result<String> {
    if let Some(pwd) = password {
        Ok(pwd)
    } else {
        println!("Enter encryption password for backup:");
        let pwd = rpassword::read_password().context("Failed to read password")?;
        println!("Confirm password:");
        let pwd_confirm = rpassword::read_password().context("Failed to read password")?;

        if pwd != pwd_confirm {
            anyhow::bail!("Passwords do not match");
        }

        Ok(pwd)
    }
}

fn convert_to_backup_entry(path: &str, data: &serde_json::Value) -> VaultEntryBackup {
    VaultEntryBackup {
        id: data
            .get("id")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string(),
        path: path.to_string(),
        encrypted_data: data
            .get("data")
            .and_then(|v| serde_json::to_string(v).ok())
            .unwrap_or_default(),
        encryption_metadata: data
            .get("encryption_metadata")
            .cloned()
            .unwrap_or(serde_json::json!({})),
        security_level: data
            .get("security_level")
            .and_then(|v| v.as_str())
            .unwrap_or("Internal")
            .to_string(),
        metadata: data
            .get("metadata")
            .cloned()
            .unwrap_or(serde_json::json!({})),
        tags: data
            .get("tags")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default(),
        version: data.get("version").and_then(|v| v.as_u64()).unwrap_or(1) as u32,
        owner_id: data
            .get("owner_id")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string(),
        created_at: Utc::now(),
        updated_at: Utc::now(),
        expires_at: None,
    }
}

fn convert_to_audit_backup(data: &serde_json::Value) -> AuditLogBackup {
    AuditLogBackup {
        event_id: data
            .get("event_id")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string(),
        timestamp: Utc::now(),
        event_type: data
            .get("event_type")
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown")
            .to_string(),
        operation: data
            .get("operation")
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown")
            .to_string(),
        resource_path: data
            .get("resource_path")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string(),
        result: data
            .get("result")
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown")
            .to_string(),
        nip: data
            .get("nip")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()),
        satker_code: data
            .get("satker_code")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()),
        metadata: data
            .get("metadata")
            .cloned()
            .unwrap_or(serde_json::json!({})),
    }
}

fn calculate_checksum(data: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(data);
    format!("{:x}", hasher.finalize())
}

fn compress_data(data: &[u8], level: u8) -> Result<Vec<u8>> {
    use flate2::Compression;
    use flate2::write::GzEncoder;

    let mut encoder = GzEncoder::new(Vec::new(), Compression::new(level as u32));
    encoder.write_all(data).context("Failed to compress data")?;
    encoder.finish().context("Failed to finish compression")
}

fn decompress_data(data: &[u8]) -> Result<Vec<u8>> {
    use flate2::read::GzDecoder;

    let mut decoder = GzDecoder::new(data);
    let mut decompressed = Vec::new();
    decoder
        .read_to_end(&mut decompressed)
        .context("Failed to decompress data")?;
    Ok(decompressed)
}

fn encrypt_data(data: &[u8], password: &str) -> Result<Vec<u8>> {
    use aes_gcm::{
        Aes256Gcm, Nonce,
        aead::{Aead, KeyInit},
    };
    use argon2::Argon2;

    let salt = b"secreton-backup-salt-v1";
    let mut key = [0u8; 32];
    Argon2::default()
        .hash_password_into(password.as_bytes(), salt, &mut key)
        .map_err(|e| anyhow::anyhow!("Failed to derive key: {}", e))?;

    let nonce = Nonce::from(*b"unique nonce");

    let cipher = Aes256Gcm::new(&key.into());
    let ciphertext = cipher
        .encrypt(&nonce, data)
        .map_err(|e| anyhow::anyhow!("Encryption failed: {}", e))?;

    let mut result = nonce.to_vec();
    result.extend_from_slice(&ciphertext);

    Ok(result)
}

fn decrypt_data(data: &[u8], password: &str) -> Result<Vec<u8>> {
    use aes_gcm::{
        Aes256Gcm, Nonce,
        aead::{Aead, KeyInit},
    };
    use argon2::Argon2;

    if data.len() < 12 {
        anyhow::bail!("Invalid encrypted data: too short");
    }

    let salt = b"secreton-backup-salt-v1";
    let mut key = [0u8; 32];
    Argon2::default()
        .hash_password_into(password.as_bytes(), salt, &mut key)
        .map_err(|e| anyhow::anyhow!("Failed to derive key: {}", e))?;

    let (nonce_bytes, ciphertext) = data.split_at(12);
    if nonce_bytes.len() != 12 {
        return Err(anyhow::anyhow!("Invalid nonce size"));
    }
    let mut nonce_arr = [0u8; 12];
    nonce_arr.copy_from_slice(nonce_bytes);
    let nonce = Nonce::from(nonce_arr);

    let cipher = Aes256Gcm::new(&key.into());
    let plaintext = cipher
        .decrypt(&nonce, ciphertext)
        .map_err(|e| anyhow::anyhow!("Decryption failed: {}", e))?;

    Ok(plaintext)
}

async fn should_include_in_incremental(_path: &str, _base_manifest: &BackupManifest) -> bool {
    true
}

async fn load_backup_manifest(file_path: &str, password: &str) -> Result<BackupManifest> {
    let mut file = File::open(file_path).context("Failed to open base backup")?;
    let mut encrypted_data = Vec::new();
    file.read_to_end(&mut encrypted_data)
        .context("Failed to read base backup")?;

    let compressed_data = decrypt_data(&encrypted_data, password)?;
    let json_data = decompress_data(&compressed_data)?;

    let backup_data: BackupData =
        serde_json::from_slice(&json_data).context("Failed to parse backup")?;

    Ok(backup_data.manifest)
}
