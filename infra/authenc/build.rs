//! Build script for compiling Protocol Buffer definitions
//!
//! This script uses tonic-build to compile .proto files into Rust code
//! at build time. The generated code is included in the binary.

use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Skip compilation if grpc feature is not enabled
    if std::env::var("CARGO_FEATURE_GRPC").is_err() {
        println!("Skipping protobuf compilation because grpc feature is disabled");
        return Ok(());
    }

    // Get the proto directory path
    // Try multiple locations for flexibility:
    // 1. proto/ (for Docker builds)
    // 2. ../proto/ (for local development from authenc/)
    let proto_dir = if PathBuf::from("proto").exists() {
        PathBuf::from("proto")
    } else {
        PathBuf::from("../proto")
    };

    // Proto files to compile
    let proto_files = vec![
        proto_dir.join("authenc.proto"),
        proto_dir.join("common.proto"),
        proto_dir.join("secreton.proto"), // For Secreton client
    ];

    // Verify proto files exist
    for proto_file in &proto_files {
        if !proto_file.exists() {
            eprintln!("Warning: Proto file not found: {:?}", proto_file);
        }
    }

    // Configure tonic-prost-build (tonic 0.14+)
    tonic_prost_build::configure()
        // Set the output directory for generated code
        .build_server(true)
        .build_client(true)
        // Add serde derives for serialization support
        .type_attribute(".", "#[derive(serde::Serialize, serde::Deserialize)]")
        // Compile the proto files
        .compile_protos(
            &proto_files,
            &[proto_dir.clone()], // Include directories
        )?;

    // Tell Cargo to rerun this build script if proto files change
    for proto_file in &proto_files {
        if proto_file.exists() {
            println!("cargo:rerun-if-changed={}", proto_file.display());
        }
    }

    Ok(())
}
