//! Configuration Migration Utilities
//!
//! Migrates legacy TOML configuration files to the new encrypted storage system.
//! This is a one-time operation that should be run when upgrading to the new config system.

use anyhow::Result;
use chrono::Utc;
use std::path::Path;

use crate::config::{ApplicationConfig, ConfigMetadata};
use crate::services::seal::SealService;
use secreton_storage::StorageBackend;

/// Migration from legacy TOML files to encrypted storage
pub struct ConfigMigration;

impl ConfigMigration {
    /// Migrate from legacy TOML configuration files
    ///
    /// # Arguments
    /// * `config_dir` - Directory containing legacy config files
    /// * `storage` - Storage backend for encrypted config
    /// * `seal_service` - Seal service (must be unsealed)
    ///
    /// # Returns
    /// The migrated ApplicationConfig
    pub async fn migrate_from_toml(
        config_dir: &Path,
        storage: &dyn StorageBackend,
        seal_service: &SealService,
    ) -> Result<ApplicationConfig> {
        tracing::info!("🔄 Starting configuration migration from TOML files");

        // Check if engine is unsealed
        let status = seal_service.status().await;
        if !matches!(status.state, crate::services::seal::SealState::Unsealed) {
            anyhow::bail!("Engine must be unsealed to perform migration");
        }

        // Read legacy TOML files
        let default_path = config_dir.join("default.toml");
        let production_path = config_dir.join("production.toml");

        // Check if files exist
        if !default_path.exists() {
            anyhow::bail!("default.toml not found in {:?}", config_dir);
        }

        let default_toml = std::fs::read_to_string(&default_path)
            .map_err(|e| anyhow::anyhow!("Failed to read default.toml: {}", e))?;

        let production_toml = if production_path.exists() {
            Some(
                std::fs::read_to_string(&production_path)
                    .map_err(|e| anyhow::anyhow!("Failed to read production.toml: {}", e))?,
            )
        } else {
            None
        };

        tracing::info!("📖 Read legacy TOML files");

        // Convert to ApplicationConfig
        let mut config =
            ApplicationConfig::from_legacy_toml(&default_toml, production_toml.as_deref())?;

        // Update metadata
        config.metadata = ConfigMetadata {
            version: 1,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            updated_by: Some("migration".to_string()),
        };

        tracing::info!("✅ Converted TOML to ApplicationConfig");

        // Save to encrypted storage
        config.save_to_storage(storage, seal_service).await?;

        tracing::info!("💾 Saved encrypted configuration to storage");

        Ok(config)
    }

    /// Backup legacy configuration files before migration
    pub fn backup_legacy_files(config_dir: &Path) -> Result<()> {
        let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
        let backup_dir = config_dir
            .parent()
            .ok_or_else(|| anyhow::anyhow!("Invalid config directory"))?
            .join(format!("config_backup_{}", timestamp));

        std::fs::create_dir_all(&backup_dir)?;

        tracing::info!("📦 Creating backup at {:?}", backup_dir);

        // Files to backup
        let files_to_backup = vec![
            "default.toml",
            "production.toml",
            "raft.toml",
            "engine.toml",
            ".env",
            ".env.example",
        ];

        for filename in files_to_backup {
            let src = config_dir.join(filename);
            if src.exists() {
                let dst = backup_dir.join(filename);
                std::fs::copy(&src, &dst)
                    .map_err(|e| anyhow::anyhow!("Failed to backup {}: {}", filename, e))?;
                tracing::info!("  ✓ Backed up {}", filename);
            }
        }

        tracing::info!("✅ Backup complete at {:?}", backup_dir);

        Ok(())
    }

    /// Delete legacy configuration files after successful migration
    ///
    /// **CAUTION:** Only call this after verifying migration was successful
    pub fn cleanup_legacy_files(config_dir: &Path) -> Result<()> {
        tracing::warn!("🗑️  Cleaning up legacy configuration files");

        let files_to_delete = vec![
            "default.toml",
            "production.toml",
            "raft.toml",
            "engine.toml",
            ".env.example",
        ];

        for filename in files_to_delete {
            let path = config_dir.join(filename);
            if path.exists() {
                std::fs::remove_file(&path)
                    .map_err(|e| anyhow::anyhow!("Failed to delete {}: {}", filename, e))?;
                tracing::info!("  ✓ Deleted {}", filename);
            }
        }

        tracing::info!("🎉 Cleanup complete");

        Ok(())
    }

    /// Verify migrated configuration
    pub async fn verify_migration(
        storage: &dyn StorageBackend,
        seal_service: &SealService,
    ) -> Result<ApplicationConfig> {
        tracing::info!("🔍 Verifying migrated configuration");

        // Try to load from storage
        let config = ApplicationConfig::load_from_storage(storage, seal_service).await?;

        // Basic validation
        if config.metadata.version == 0 {
            anyhow::bail!("Invalid config version");
        }

        tracing::info!("✅ Configuration verified successfully");
        tracing::info!("   Version: {}", config.metadata.version);
        tracing::info!("   Created: {}", config.metadata.created_at);
        tracing::info!("   Auth enabled: {}", config.auth.require_auth);
        tracing::info!("   MFA enabled: {}", config.mfa.enabled);

        Ok(config)
    }
}

/// CLI Migration Command Handler
pub async fn run_migration(
    config_dir: &Path,
    storage: &dyn StorageBackend,
    seal_service: &SealService,
    skip_backup: bool,
) -> Result<()> {
    println!("╔══════════════════════════════════════════════════════╗");
    println!("║   Secreton Configuration Migration Utility          ║");
    println!("╚══════════════════════════════════════════════════════╝");
    println!();

    // Step 1: Backup (unless skipped)
    if !skip_backup {
        println!("📦 Step 1/4: Backing up legacy configuration...");
        ConfigMigration::backup_legacy_files(config_dir)?;
        println!();
    } else {
        println!("⚠️  Skipping backup (--skip-backup flag set)");
        println!();
    }

    // Step 2: Migrate
    println!("🔄 Step 2/4: Migrating configuration to encrypted storage...");
    let config = ConfigMigration::migrate_from_toml(config_dir, storage, seal_service).await?;
    println!("   ✓ Migration complete");
    println!();

    // Step 3: Verify
    println!("🔍 Step 3/4: Verifying migrated configuration...");
    ConfigMigration::verify_migration(storage, seal_service).await?;
    println!("   ✓ Verification successful");
    println!();

    // Step 4: Summary
    println!("✅ Step 4/4: Migration Summary");
    println!("   • Configuration version: {}", config.metadata.version);
    println!("   • Auth required: {}", config.auth.require_auth);
    println!("   • MFA enabled: {}", config.mfa.enabled);
    println!("   • Rate limiting: {}", config.rate_limit.enabled);
    println!("   • Audit logging: {}", config.audit.enabled);
    println!();

    println!("🎉 Migration completed successfully!");
    println!();
    println!("⚠️  IMPORTANT NEXT STEPS:");
    println!("   1. Test the new configuration thoroughly");
    println!("   2. Verify all services work correctly");
    println!("   3. Only then, run cleanup to delete old files:");
    println!("      secreton migrate --cleanup");
    println!();

    Ok(())
}

/// Cleanup command handler
pub async fn run_cleanup(config_dir: &Path) -> Result<()> {
    println!("╔══════════════════════════════════════════════════════╗");
    println!("║   Secreton Configuration Cleanup Utility            ║");
    println!("╚══════════════════════════════════════════════════════╝");
    println!();

    println!("⚠️  WARNING: This will permanently delete legacy config files!");
    println!();
    println!("Press Enter to continue or Ctrl+C to cancel...");

    use std::io::{BufRead, stdin};
    let stdin = stdin();
    let mut lines = stdin.lock().lines();
    let _ = lines.next();

    println!();
    println!("🗑️  Deleting legacy configuration files...");
    ConfigMigration::cleanup_legacy_files(config_dir)?;
    println!();
    println!("✅ Cleanup complete!");

    Ok(())
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_migration_utility_exists() {
        // Basic test to ensure module compiles
        assert!(true);
    }
}
