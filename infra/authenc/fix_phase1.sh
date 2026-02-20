#!/bin/bash
# Phase 1 Quick Fixes Script

echo "=== Phase 1: Quick Fixes ==="
echo ""

# Step 2: Comment out unused imports (safer than deleting)
echo "Step 2: Commenting out unused imports..."

# Fix token_exchange.rs
sed -i 's/^use crate::utils::crypto::jwt::verify_jwt_with_validation;/\/\/ use crate::utils::crypto::jwt::verify_jwt_with_validation; \/\/ UNUSED/' crates/core/src/services/token_exchange.rs
sed -i 's/^use crate::stores::audit_log_store::AuditLogStore;/\/\/ use crates::stores::audit_log_store::AuditLogStore; \/\/ UNUSED/' crates/core/src/services/token_exchange.rs
sed -i 's/^use crate::services::jwt_validator::JwtValidator;/\/\/ use crate::services::jwt_validator::JwtValidator; \/\/ UNUSED/' crates/core/src/services/token_exchange.rs

# Fix enhanced_audit.rs
sed -i 's/^use crate::utils::/\/\/ use crate::utils:: \/\/ UNUSED - /' crates/core/src/services/enhanced_audit.rs

# Fix pg_audit_log_store.rs
sed -i 's/^use crate::stores::audit_log_store::AuditLogStore;/\/\/ use crate::stores::audit_log_store::AuditLogStore; \/\/ UNUSED/' crates/core/src/services/pg_audit_log_store.rs

# Fix elasticsearch_audit_log_sink.rs
sed -i 's/^use reqwest::Client;/\/\/ use reqwest::Client; \/\/ UNUSED/' crates/core/src/services/elasticsearch_audit_log_sink.rs

# Fix cache_invalidation_listener.rs
sed -i 's/use crate::events::{EventError, EventListener}/\/\/ use crate::events::{EventError, EventListener} \/\/ UNUSED/' crates/core/src/services/cache_invalidation_listener.rs

# Fix cache_utils.rs
sed -i 's/^use lib_common::cache::/\/\/ use lib_common::cache:: \/\/ UNUSED - /' crates/core/src/services/cache_utils.rs
sed -i 's/^pub use lib_common::cache::/\/\/ pub use lib_common::cache:: \/\/ UNUSED - /' crates/core/src/services/cache_utils.rs

# Fix key_rotation.rs
sed -i 's/^use crate::secreton_client::secreton_client::SecretonClient;/\/\/ use crate::secreton_client::secreton_client::SecretonClient; \/\/ UNUSED/' crates/core/src/services/key_rotation.rs

# Fix spi/credential/mod.rs
sed -i 's/use crate::utils::crypto::password::/\/\/ use crate::utils::crypto::password:: \/\/ UNUSED - /' crates/core/src/spi/credential/mod.rs

# Fix config/mfa_fallback.rs
sed -i 's/use crate::crypto::aes_gcm::AesGcmService;/\/\/ use crate::crypto::aes_gcm::AesGcmService; \/\/ UNUSED/' crates/core/src/config/mfa_fallback.rs

# Fix config/mod.rs
sed -i 's/^use crate::middleware::/\/\/ use crate::middleware:: \/\/ UNUSED - /' crates/core/src/config/mod.rs

echo "✓ Unused imports commented out"
echo ""

# Step 3: Fix feature-gated module exports
echo "Step 3: Fixing feature-gated module exports..."

# This will be done manually in services/mod.rs and cache/mod.rs

echo "✓ Feature gates need manual update"
echo ""

echo "=== Phase 1 Complete ==="
echo "Run: cargo check -p authenc-core to verify"
