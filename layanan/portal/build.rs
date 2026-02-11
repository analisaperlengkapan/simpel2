//! Build script for compiling Protocol Buffer definitions
//!
//! Compiles authenc.proto and secreton.proto for gRPC client generation.

use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Get the proto directory path - relative to workspace root
    let manifest_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    let proto_dir = manifest_dir.join("../../infra/authenc/proto");

    // Verify proto directory exists
    if !proto_dir.exists() {
        eprintln!("Warning: Proto directory not found: {:?}", proto_dir);
        println!("cargo:warning=Proto directory not found, skipping proto compilation");
        return Ok(());
    }

    // Proto files to compile (only client-side needed)
    let proto_files = vec![
        proto_dir.join("authenc.proto"),
        proto_dir.join("secreton.proto"),
        proto_dir.join("common.proto"),
    ];

    // Verify proto files exist
    for proto_file in &proto_files {
        if !proto_file.exists() {
            eprintln!("Warning: Proto file not found: {:?}", proto_file);
        }
    }

    // Configure tonic-prost-build - client only (no server)
    tonic_prost_build::configure()
        .build_server(false) // Only need client
        .build_client(true)
        // Add serde derives for serialization support
        .type_attribute(".", "#[derive(serde::Serialize, serde::Deserialize)]")
        .compile_protos(&proto_files, &[proto_dir.clone()])?;

    // Tell Cargo to rerun this build script if proto files change
    for proto_file in &proto_files {
        if proto_file.exists() {
            println!("cargo:rerun-if-changed={}", proto_file.display());
        }
    }

    Ok(())
}
