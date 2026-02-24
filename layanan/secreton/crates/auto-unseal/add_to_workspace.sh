#!/bin/bash
# Script to add auto-unseal crate to workspace members

CARGO_TOML="/home/anbud02/simpel2/Cargo.toml"

# Check if auto-unseal is already in workspace members
if grep -q '"infra/secreton/crates/auto-unseal"' "$CARGO_TOML"; then
    echo "auto-unseal is already in workspace members"
else
    echo "Adding auto-unseal to workspace members..."
    # Add after k8s-operator line
    sed -i '/"infra\/secreton\/crates\/k8s-operator",/a\    "infra/secreton/crates/auto-unseal",' "$CARGO_TOML"
    echo "Done!"
fi

# Run the tests
echo "Running property-based tests..."
cargo test --manifest-path /home/anbud02/simpel2/infra/secreton/crates/auto-unseal/Cargo.toml --test property_tests
