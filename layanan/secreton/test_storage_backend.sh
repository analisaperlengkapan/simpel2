#!/bin/bash

# Test script for Secreton persistent storage backend implementation
# This script verifies that the storage backend configuration is working correctly

set -e

echo "=========================================="
echo "Secreton Storage Backend Test"
echo "=========================================="
echo ""

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Test counters
TESTS_PASSED=0
TESTS_FAILED=0

# Helper functions
test_passed() {
    echo -e "${GREEN}✓ PASSED${NC}: $1"
    ((TESTS_PASSED++))
}

test_failed() {
    echo -e "${RED}✗ FAILED${NC}: $1"
    ((TESTS_FAILED++))
}

test_info() {
    echo -e "${YELLOW}ℹ INFO${NC}: $1"
}

# Test 1: Check if Cargo build succeeds
echo "Test 1: Cargo build"
if cargo build --release -p secreton-api --bin api_server 2>&1 | grep -q "Finished"; then
    test_passed "Cargo build completed successfully"
else
    test_failed "Cargo build failed"
fi
echo ""

# Test 2: Check if Docker image builds
echo "Test 2: Docker image build"
if docker build -t secreton:latest . 2>&1 | grep -q "Successfully tagged"; then
    test_passed "Docker image built successfully"
else
    test_failed "Docker image build failed"
fi
echo ""

# Test 3: Check if config files exist
echo "Test 3: Configuration files"
if [ -f "config/default.toml" ]; then
    test_passed "default.toml exists"
else
    test_failed "default.toml not found"
fi

if [ -f "config/production.toml" ]; then
    test_passed "production.toml exists"
else
    test_failed "production.toml not found"
fi
echo ""

# Test 4: Check if storage configuration is in config files
echo "Test 4: Storage configuration in config files"
if grep -q "^\[storage\]" config/default.toml; then
    test_passed "Storage section found in default.toml"
else
    test_failed "Storage section not found in default.toml"
fi

if grep -q "backend = \"raft\"" config/default.toml; then
    test_passed "Raft backend set as default in default.toml"
else
    test_failed "Raft backend not set as default in default.toml"
fi

if grep -q "^\[storage\]" config/production.toml; then
    test_passed "Storage section found in production.toml"
else
    test_failed "Storage section not found in production.toml"
fi

if grep -q "backend = \"raft\"" config/production.toml; then
    test_passed "Raft backend set as default in production.toml"
else
    test_failed "Raft backend not set as default in production.toml"
fi
echo ""

# Test 5: Check if Raft configuration exists
echo "Test 5: Raft configuration"
if grep -q "^\[storage.raft\]" config/default.toml; then
    test_passed "Raft configuration section found in default.toml"
else
    test_failed "Raft configuration section not found in default.toml"
fi

if grep -q "node_id = 1" config/default.toml; then
    test_passed "Raft node_id configured in default.toml"
else
    test_failed "Raft node_id not configured in default.toml"
fi

if grep -q "data_dir = " config/default.toml; then
    test_passed "Raft data_dir configured in default.toml"
else
    test_failed "Raft data_dir not configured in default.toml"
fi
echo ""

# Test 6: Check if Docker image contains secreton.toml
echo "Test 6: Docker image configuration"
if docker run --rm secreton:latest ls -la /app/secreton.toml 2>&1 | grep -q "secreton.toml"; then
    test_passed "secreton.toml exists in Docker image"
else
    test_failed "secreton.toml not found in Docker image"
fi
echo ""

# Test 7: Check if Docker image contains api_server binary
echo "Test 7: Docker image binary"
if docker run --rm secreton:latest ls -la /app/api_server 2>&1 | grep -q "api_server"; then
    test_passed "api_server binary exists in Docker image"
else
    test_failed "api_server binary not found in Docker image"
fi
echo ""

# Test 8: Check if documentation files exist
echo "Test 8: Documentation files"
if [ -f "STORAGE_BACKEND_CONFIG.md" ]; then
    test_passed "STORAGE_BACKEND_CONFIG.md exists"
else
    test_failed "STORAGE_BACKEND_CONFIG.md not found"
fi

if [ -f "PERSISTENT_STORAGE_IMPLEMENTATION.md" ]; then
    test_passed "PERSISTENT_STORAGE_IMPLEMENTATION.md exists"
else
    test_failed "PERSISTENT_STORAGE_IMPLEMENTATION.md not found"
fi

if [ -f "IMPLEMENTATION_SUMMARY.md" ]; then
    test_passed "IMPLEMENTATION_SUMMARY.md exists"
else
    test_failed "IMPLEMENTATION_SUMMARY.md not found"
fi
echo ""

# Test 9: Check if services/mod.rs has storage backend configuration
echo "Test 9: Services layer configuration"
if grep -q "create_storage_backend" crates/api/src/services/mod.rs; then
    test_passed "create_storage_backend function exists"
else
    test_failed "create_storage_backend function not found"
fi

if grep -q "SECRETON_STORAGE_BACKEND" crates/api/src/services/mod.rs; then
    test_passed "SECRETON_STORAGE_BACKEND environment variable support exists"
else
    test_failed "SECRETON_STORAGE_BACKEND environment variable support not found"
fi

if grep -q "config.storage.backend" crates/api/src/services/mod.rs; then
    test_passed "Config file storage backend support exists"
else
    test_failed "Config file storage backend support not found"
fi
echo ""

# Test 10: Check if config.rs has StorageConfig
echo "Test 10: Configuration structures"
if grep -q "pub struct StorageConfig" crates/api/src/config.rs; then
    test_passed "StorageConfig struct exists"
else
    test_failed "StorageConfig struct not found"
fi

if grep -q "pub struct RaftConfig" crates/api/src/config.rs; then
    test_passed "RaftConfig struct exists"
else
    test_failed "RaftConfig struct not found"
fi

if grep -q "impl Default for StorageConfig" crates/api/src/config.rs; then
    test_passed "StorageConfig Default implementation exists"
else
    test_failed "StorageConfig Default implementation not found"
fi

if grep -q "impl Default for RaftConfig" crates/api/src/config.rs; then
    test_passed "RaftConfig Default implementation exists"
else
    test_failed "RaftConfig Default implementation not found"
fi
echo ""

# Summary
echo "=========================================="
echo "Test Summary"
echo "=========================================="
echo -e "Passed: ${GREEN}${TESTS_PASSED}${NC}"
echo -e "Failed: ${RED}${TESTS_FAILED}${NC}"
echo "=========================================="

if [ $TESTS_FAILED -eq 0 ]; then
    echo -e "${GREEN}All tests passed!${NC}"
    exit 0
else
    echo -e "${RED}Some tests failed!${NC}"
    exit 1
fi
