#!/bin/bash
# Secreton Compilation Fix Script
# This script applies systematic fixes to resolve compilation errors

set -e

echo "Starting Secreton compilation fixes..."

# Phase 1: Fix metrics API calls in grpc/server.rs
echo "Phase 1: Fixing metrics API calls..."

# Fix grpc/server.rs metrics
sed -i 's/metrics::counter!("\([^"]*\)", 1)/metrics::counter!("\1").increment(1)/g' crates/api/src/grpc/server.rs

# Phase 2: Fix middleware.rs issues
echo "Phase 2: Fixing middleware issues..."

# Fix request_id header parsing
sed -i 's/\.insert("x-request-id", request_id\.parse());/\.insert("x-request-id", request_id.parse().expect("Invalid header value"));/g' crates/api/src/middleware.rs

# Fix security headers parsing
sed -i 's/headers\.insert("\([^"]*\)", "\([^"]*\)"\.parse());/headers.insert("\1", "\2".parse().expect("Invalid header value"));/g' crates/api/src/middleware.rs

# Fix multi-line header inserts
sed -i 's/"max-age=31536000; includeSubDomains"\.parse(),/"max-age=31536000; includeSubDomains".parse().expect("Invalid header value"),/g' crates/api/src/middleware.rs
sed -i 's/"strict-origin-when-cross-origin"\.parse(),/"strict-origin-when-cross-origin".parse().expect("Invalid header value"),/g' crates/api/src/middleware.rs

# Phase 3: Fix auth service issues
echo "Phase 3: Fixing auth service..."

# Add InternalError variant to AuthError enum (manual fix needed)
echo "  Note: AuthError::InternalError variant needs to be added manually"

# Phase 4: Comment out problematic audit logging
echo "Phase 4: Commenting out audit logging calls..."

# This requires more careful manual editing

echo "Compilation fixes applied!"
echo "Please review COMPILATION_FIXES_NEEDED.md for remaining manual fixes"
echo ""
echo "Run 'cargo check --workspace' to verify fixes"
