//! Build script for compiling Protocol Buffer definitions
//!
//! Compiles authenc.proto, secreton.proto, and integrasi.proto for gRPC client generation.

use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let authenc_proto_dir = PathBuf::from("../../../infra/authenc/proto");
    let secreton_proto_dir = PathBuf::from("../../../infra/secreton/proto");
    let integrasi_proto_dir = PathBuf::from("../../daskrimti/integrasi/proto");

    // Verify proto directories exist
    if !authenc_proto_dir.exists() {
        eprintln!("Warning: Authenc proto directory not found: {:?}", authenc_proto_dir);
        println!("cargo:warning=Authenc proto directory not found, skipping proto compilation");
        return Ok(());
    }

    let mut proto_files = vec![
        authenc_proto_dir.join("authenc.proto"),
        authenc_proto_dir.join("secreton.proto"),
        authenc_proto_dir.join("common.proto"),
    ];

    // Add integrasi proto if exists
    let integrasi_proto = integrasi_proto_dir.join("integrasi.proto");
    if integrasi_proto.exists() {
        proto_files.push(integrasi_proto);
        println!("cargo:warning=Including integrasi.proto");
    } else {
        println!("cargo:warning=integrasi.proto not found at {:?}, skipping", integrasi_proto_dir);
    }

    // Verify proto files exist
    for proto_file in &proto_files {
        if !proto_file.exists() {
            eprintln!("Warning: Proto file not found: {:?}", proto_file);
        }
    }

    // Configure tonic-prost-build - client only (no server)
    tonic_prost_build::configure()
        .build_server(false)
        .build_client(true)
        .type_attribute(".", "#[derive(serde::Serialize, serde::Deserialize)]")
        .compile_protos(&proto_files, &[
            authenc_proto_dir.clone(),
            secreton_proto_dir,
            integrasi_proto_dir,
        ])?;

    // Tell Cargo to rerun this build script if proto files change
    for proto_file in &proto_files {
        if proto_file.exists() {
            println!("cargo:rerun-if-changed={}", proto_file.display());
        }
    }

    Ok(())
}
