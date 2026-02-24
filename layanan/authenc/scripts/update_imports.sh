#!/bin/bash
# Script to update imports after migration to multi-crate structure
# Usage: ./scripts/update_imports.sh [--dry-run]

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

DRY_RUN=false
if [[ "$1" == "--dry-run" ]]; then
    DRY_RUN=true
    echo -e "${YELLOW}Running in DRY RUN mode${NC}"
fi

log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

echo "========================================="
echo "  Import Update Script"
echo "========================================="
echo ""

# Function to update imports in a file
update_imports_in_file() {
    local file="$1"

    if [[ ! -f "$file" ]]; then
        return
    fi

    if [[ "$DRY_RUN" == true ]]; then
        log_info "Would update imports in: $file"
        return
    fi

    # Backup original file
    cp "$file" "$file.bak"

    # Update imports
    sed -i 's|use crate::services::captcha|use authenc_core::services::captcha|g' "$file"
    sed -i 's|use crate::services::audit|use authenc_core::services::audit|g' "$file"
    sed -i 's|use crate::services::event|use authenc_core::services::event|g' "$file"
    sed -i 's|use crate::services::cache|use authenc_core::services::cache|g' "$file"
    sed -i 's|use crate::handlers|use authenc_api::handlers|g' "$file"
    sed -i 's|use crate::middleware|use authenc_api::middleware|g' "$file"
    sed -i 's|use crate::models|use authenc_types::models|g' "$file"
    sed -i 's|use crate::crypto|use authenc_crypto|g' "$file"
    sed -i 's|use crate::database|use authenc_storage|g' "$file"

    log_success "Updated imports in: $file"
}

# Update imports in all Rust files
log_info "Updating imports in crates/..."

find crates -name "*.rs" -type f | while read -r file; do
    update_imports_in_file "$file"
done

log_success "Import update completed!"

if [[ "$DRY_RUN" == false ]]; then
    echo ""
    log_info "Backup files created with .bak extension"
    log_info "To remove backups: find crates -name '*.bak' -delete"
fi

echo ""
echo "========================================="
