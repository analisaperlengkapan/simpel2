# Rust/Cargo automation for SIMPelv2
# This makefile fragment provides unified Rust/Cargo workflow targets for the monorepo.

.PHONY: rust-build rust-test rust-fmt rust-clippy rust-doc rust-chef rust-chef-prepare rust-chef-cook

## Build all Rust crates in the workspace
rust-build:
	cargo build --workspace --all-targets

## Run all tests in the workspace
rust-test:
	cargo test --workspace --all-targets

## Format all Rust code in the workspace
rust-fmt:
	cargo fmt --all

## Lint all Rust code in the workspace
rust-clippy:
	cargo clippy --workspace --all-targets -- -D warnings

## Build documentation for all Rust crates
rust-doc:
	cargo doc --workspace --no-deps --open

## Prepare Cargo Chef recipe for Docker/CI caching
rust-chef-prepare:
	cargo chef prepare --recipe-path recipe.json

## Cook dependencies using Cargo Chef (for CI/Docker cache)
rust-chef-cook:
	cargo chef cook --recipe-path recipe.json
