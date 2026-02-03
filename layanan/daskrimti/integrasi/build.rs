//! Build script for compiling Protocol Buffer definitions
//!
//! This script uses tonic-prost-build to compile .proto files into Rust code
//! at build time. The generated code is included via tonic::include_proto!

use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Skip proto compilation if grpc feature is not enabled
    if std::env::var("CARGO_FEATURE_GRPC").is_err() {
        println!("cargo::warning=Skipping proto compilation: grpc feature not enabled");
        return Ok(());
    }

    // Get the proto directory path
    let proto_dir = PathBuf::from("proto");

    if !proto_dir.exists() {
        println!(
            "cargo::warning=Proto directory not found at {:?}, skipping proto compilation",
            proto_dir
        );
        return Ok(());
    }

    // Proto file to compile
    let proto_file = proto_dir.join("integrasi.proto");

    if !proto_file.exists() {
        println!(
            "cargo::warning=Proto file not found at {:?}, skipping proto compilation",
            proto_file
        );
        return Ok(());
    }

    // Tell cargo to rerun this build script if the proto file changes
    println!("cargo::rerun-if-changed=proto/integrasi.proto");
    println!("cargo::rerun-if-env-changed=CARGO_FEATURE_GRPC");

    // Compile the proto file using tonic_prost_build
    tonic_prost_build::configure()
        .build_server(true)
        .build_client(true)
        .compile_protos(&[proto_file], &[proto_dir])?;

    println!("cargo::warning=Successfully compiled integrasi.proto");

    Ok(())
}
