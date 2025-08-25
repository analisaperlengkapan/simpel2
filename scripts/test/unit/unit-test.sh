#!/bin/bash

# Unit Test Runner for SIMPelv2
# Runs unit tests for all Rust components

set -euo pipefail

WORKSPACE_ROOT="/var/www/simpelv2"

echo "🧪 Running Unit Tests"
echo "===================="

cd "$WORKSPACE_ROOT"

# Run Cargo tests
echo "🦀 Running Rust unit tests..."
cargo test --all --lib --bins

echo "✅ Unit tests completed"
