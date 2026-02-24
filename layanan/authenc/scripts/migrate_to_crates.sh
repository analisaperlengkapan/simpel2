#!/bin/bash
# Migration Script: Authenc Monolith → Multi-Crate Architecture
# Usage: ./scripts/migrate_to_crates.sh [--dry-run]

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
DRY_RUN=false
if [[ "$1" == "--dry-run" ]]; then
    DRY_RUN=true
    echo -e "${YELLOW}Running in DRY RUN mode - no changes will be made${NC}"
fi

# Helper functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Function to migrate a file
migrate_file() {
    local src_file="$1"
    local dest_file="$2"

    if [[ ! -f "$src_file" ]]; then
        log_warning "Source file not found: $src_file"
        return 1
    fi

    if [[ "$DRY_RUN" == true ]]; then
        log_info "Would migrate: $src_file → $dest_file"
        return 0
    fi

    # Create destination directory
    mkdir -p "$(dirname "$dest_file")"

    # Copy file
    cp "$src_file" "$dest_file"
    log_success "Migrated: $src_file → $dest_file"
}

# Function to migrate a directory
migrate_directory() {
    local src_dir="$1"
    local dest_dir="$2"

    if [[ ! -d "$src_dir" ]]; then
        log_warning "Source directory not found: $src_dir"
        return 1
    fi

    if [[ "$DRY_RUN" == true ]]; then
        log_info "Would migrate directory: $src_dir → $dest_dir"
        return 0
    fi

    # Create destination directory
    mkdir -p "$dest_dir"

    # Copy directory contents
    cp -r "$src_dir"/* "$dest_dir"/
    log_success "Migrated directory: $src_dir → $dest_dir"
}

echo "========================================="
echo "  Authenc Multi-Crate Migration Script"
echo "========================================="
echo ""

# Check if we're in the right directory
if [[ ! -f "Cargo.toml" ]] || [[ ! -d "src" ]]; then
    log_error "Please run this script from the authenc root directory"
    exit 1
fi

log_info "Starting migration process..."
echo ""

# Phase 1: Migrate authenc-core services
log_info "Phase 1: Migrating authenc-core services..."

# Migrate CAPTCHA service (if not already migrated)
if [[ -d "src/services/captcha" ]]; then
    migrate_directory "src/services/captcha" "crates/core/src/services/captcha"
fi

# Migrate audit services
if [[ -f "src/services/audit_events.rs" ]]; then
    migrate_file "src/services/audit_events.rs" "crates/core/src/services/audit_events.rs"
fi

if [[ -f "src/services/audit_integrity.rs" ]]; then
    migrate_file "src/services/audit_integrity.rs" "crates/core/src/services/audit_integrity.rs"
fi

if [[ -f "src/services/audit_signature.rs" ]]; then
    migrate_file "src/services/audit_signature.rs" "crates/core/src/services/audit_signature.rs"
fi

if [[ -f "src/services/enhanced_audit.rs" ]]; then
    migrate_file "src/services/enhanced_audit.rs" "crates/core/src/services/enhanced_audit.rs"
fi

# Migrate event services
if [[ -f "src/services/event_publisher.rs" ]]; then
    migrate_file "src/services/event_publisher.rs" "crates/core/src/services/event_publisher.rs"
fi

if [[ -f "src/services/event_retention.rs" ]]; then
    migrate_file "src/services/event_retention.rs" "crates/core/src/services/event_retention.rs"
fi

if [[ -f "src/services/event_listeners.rs" ]]; then
    migrate_file "src/services/event_listeners.rs" "crates/core/src/services/event_listeners.rs"
fi

# Migrate cache services
if [[ -d "src/services/cache" ]]; then
    migrate_directory "src/services/cache" "crates/core/src/services/cache"
fi

if [[ -f "src/services/cache_invalidation_listener.rs" ]]; then
    migrate_file "src/services/cache_invalidation_listener.rs" "crates/core/src/services/cache_invalidation_listener.rs"
fi

log_success "Phase 1 completed"
echo ""

# Phase 2: Migrate authenc-api handlers
log_info "Phase 2: Migrating authenc-api handlers..."

# Create handlers directory in api crate
if [[ "$DRY_RUN" == false ]]; then
    mkdir -p "crates/api/src/handlers"
fi

# Migrate OAuth2 handlers
if [[ -f "src/handlers/oauth2.rs" ]]; then
    migrate_file "src/handlers/oauth2.rs" "crates/api/src/handlers/oauth2.rs"
fi

if [[ -f "src/handlers/oauth2_authz_code.rs" ]]; then
    migrate_file "src/handlers/oauth2_authz_code.rs" "crates/api/src/handlers/oauth2_authz_code.rs"
fi

# Migrate OIDC handlers
for file in src/handlers/oidc_*.rs; do
    if [[ -f "$file" ]]; then
        filename=$(basename "$file")
        migrate_file "$file" "crates/api/src/handlers/$filename"
    fi
done

# Migrate authentication handlers
if [[ -f "src/handlers/auth_helpers.rs" ]]; then
    migrate_file "src/handlers/auth_helpers.rs" "crates/api/src/handlers/auth_helpers.rs"
fi

# Migrate token handlers
if [[ -f "src/handlers/token_exchange.rs" ]]; then
    migrate_file "src/handlers/token_exchange.rs" "crates/api/src/handlers/token_exchange.rs"
fi

# Migrate WebAuthn handlers
if [[ -f "src/handlers/webauthn.rs" ]]; then
    migrate_file "src/handlers/webauthn.rs" "crates/api/src/handlers/webauthn.rs"
fi

# Migrate TOTP handlers
if [[ -f "src/handlers/totp.rs" ]]; then
    migrate_file "src/handlers/totp.rs" "crates/api/src/handlers/totp.rs"
fi

if [[ -f "src/handlers/totp_verify.rs" ]]; then
    migrate_file "src/handlers/totp_verify.rs" "crates/api/src/handlers/totp_verify.rs"
fi

# Migrate session handlers
if [[ -f "src/handlers/session.rs" ]]; then
    migrate_file "src/handlers/session.rs" "crates/api/src/handlers/session.rs"
fi

# Migrate health handlers
if [[ -f "src/handlers/health.rs" ]]; then
    migrate_file "src/handlers/health.rs" "crates/api/src/handlers/health.rs"
fi

# Migrate metrics handlers
if [[ -f "src/handlers/metrics.rs" ]]; then
    migrate_file "src/handlers/metrics.rs" "crates/api/src/handlers/metrics.rs"
fi

# Migrate JWKS handlers
if [[ -f "src/handlers/jwks.rs" ]]; then
    migrate_file "src/handlers/jwks.rs" "crates/api/src/handlers/jwks.rs"
fi

log_success "Phase 2 completed"
echo ""

# Phase 3: Migrate middleware
log_info "Phase 3: Migrating middleware..."

if [[ -d "src/middleware" ]]; then
    migrate_directory "src/middleware" "crates/api/src/middleware"
fi

log_success "Phase 3 completed"
echo ""

# Phase 4: Migrate authenc-iam-api handlers
log_info "Phase 4: Migrating authenc-iam-api handlers..."

# Create handlers directory in iam-api crate
if [[ "$DRY_RUN" == false ]]; then
    mkdir -p "crates/iam-api/src/handlers"
fi

# Migrate admin handlers
if [[ -f "src/handlers/admin.rs" ]]; then
    migrate_file "src/handlers/admin.rs" "crates/iam-api/src/handlers/admin.rs"
fi

# Migrate client management handlers
if [[ -f "src/handlers/client_registration.rs" ]]; then
    migrate_file "src/handlers/client_registration.rs" "crates/iam-api/src/handlers/client_registration.rs"
fi

if [[ -f "src/handlers/dcr_admin.rs" ]]; then
    migrate_file "src/handlers/dcr_admin.rs" "crates/iam-api/src/handlers/dcr_admin.rs"
fi

if [[ -f "src/handlers/client_policy.rs" ]]; then
    migrate_file "src/handlers/client_policy.rs" "crates/iam-api/src/handlers/client_policy.rs"
fi

# Migrate federation admin handlers
if [[ -f "src/handlers/federation_admin.rs" ]]; then
    migrate_file "src/handlers/federation_admin.rs" "crates/iam-api/src/handlers/federation_admin.rs"
fi

if [[ -f "src/handlers/jit_admin_service.rs" ]]; then
    migrate_file "src/handlers/jit_admin_service.rs" "crates/iam-api/src/handlers/jit_admin_service.rs"
fi

# Migrate group handlers
if [[ -f "src/handlers/group.rs" ]]; then
    migrate_file "src/handlers/group.rs" "crates/iam-api/src/handlers/group.rs"
fi

# Migrate organization handlers
if [[ -f "src/handlers/organization.rs" ]]; then
    migrate_file "src/handlers/organization.rs" "crates/iam-api/src/handlers/organization.rs"
fi

# Migrate satker handlers
if [[ -f "src/handlers/satker.rs" ]]; then
    migrate_file "src/handlers/satker.rs" "crates/iam-api/src/handlers/satker.rs"
fi

# Migrate audit handlers
if [[ -f "src/handlers/audit.rs" ]]; then
    migrate_file "src/handlers/audit.rs" "crates/iam-api/src/handlers/audit.rs"
fi

# Migrate SPI handlers
if [[ -f "src/handlers/spi_management.rs" ]]; then
    migrate_file "src/handlers/spi_management.rs" "crates/iam-api/src/handlers/spi_management.rs"
fi

if [[ -f "src/handlers/spi_federation.rs" ]]; then
    migrate_file "src/handlers/spi_federation.rs" "crates/iam-api/src/handlers/spi_federation.rs"
fi

log_success "Phase 4 completed"
echo ""

# Phase 5: Migrate authenc-mfa services
log_info "Phase 5: Migrating authenc-mfa services..."

# Migrate MFA services
if [[ -f "src/services/mfa_fallback_client.rs" ]]; then
    migrate_file "src/services/mfa_fallback_client.rs" "crates/mfa/src/fallback_client.rs"
fi

if [[ -f "src/services/mfa_local_storage.rs" ]]; then
    migrate_file "src/services/mfa_local_storage.rs" "crates/mfa/src/local_storage.rs"
fi

if [[ -f "src/services/mfa_security_monitor.rs" ]]; then
    migrate_file "src/services/mfa_security_monitor.rs" "crates/mfa/src/security_monitor.rs"
fi

if [[ -f "src/services/mfa_performance_monitor.rs" ]]; then
    migrate_file "src/services/mfa_performance_monitor.rs" "crates/mfa/src/performance_monitor.rs"
fi

if [[ -f "src/services/mfa_audit_logger.rs" ]]; then
    migrate_file "src/services/mfa_audit_logger.rs" "crates/mfa/src/audit_logger.rs"
fi

if [[ -f "src/services/totp_store.rs" ]]; then
    migrate_file "src/services/totp_store.rs" "crates/mfa/src/totp_store.rs"
fi

log_success "Phase 5 completed"
echo ""

# Phase 6: Migrate authenc-federation services
log_info "Phase 6: Migrating authenc-federation services..."

# Migrate federation services
if [[ -f "src/services/federation_manager.rs" ]]; then
    migrate_file "src/services/federation_manager.rs" "crates/federation/src/manager.rs"
fi

if [[ -f "src/services/federation_provider.rs" ]]; then
    migrate_file "src/services/federation_provider.rs" "crates/federation/src/provider.rs"
fi

if [[ -f "src/services/advanced_federation.rs" ]]; then
    migrate_file "src/services/advanced_federation.rs" "crates/federation/src/advanced.rs"
fi

# Migrate SSO services
if [[ -d "src/services/sso" ]]; then
    migrate_directory "src/services/sso" "crates/federation/src/sso"
fi

# Migrate broker services
if [[ -d "src/services/broker" ]]; then
    migrate_directory "src/services/broker" "crates/federation/src/broker"
fi

# Migrate SAML services
if [[ -f "src/services/saml.rs" ]]; then
    migrate_file "src/services/saml.rs" "crates/federation/src/saml/mod.rs"
fi

if [[ -f "src/services/saml_signature.rs" ]]; then
    migrate_file "src/services/saml_signature.rs" "crates/federation/src/saml/signature.rs"
fi

# Migrate social services
if [[ -d "src/services/social" ]]; then
    migrate_directory "src/services/social" "crates/federation/src/social"
fi

log_success "Phase 6 completed"
echo ""

# Summary
echo "========================================="
echo "  Migration Summary"
echo "========================================="
echo ""

if [[ "$DRY_RUN" == true ]]; then
    log_warning "DRY RUN completed - no actual changes were made"
    echo ""
    log_info "To perform the actual migration, run:"
    echo "  ./scripts/migrate_to_crates.sh"
else
    log_success "Migration completed successfully!"
    echo ""
    log_info "Next steps:"
    echo "  1. Update imports in migrated files"
    echo "  2. Update mod.rs files in each crate"
    echo "  3. Run: cargo build --workspace"
    echo "  4. Fix any compilation errors"
    echo "  5. Run: cargo test --workspace"
    echo "  6. Update documentation"
fi

echo ""
echo "========================================="
