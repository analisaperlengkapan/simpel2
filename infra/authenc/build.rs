//! Build script for compiling Protocol Buffer definitions
//!
//! This script uses tonic-build to compile .proto files into Rust code
//! at build time. The generated code is included in the binary.

use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Get the proto directory path (relative to workspace root)
    let proto_dir = PathBuf::from("../proto");

    // Proto files to compile
    let proto_files = vec![
        proto_dir.join("authenc.proto"),
        proto_dir.join("common.proto"),
    ];

    // Configure tonic-build
    tonic_build::configure()
        // Set the output directory for generated code
        .build_server(true)
        .build_client(true)
        // Compile the proto files
        .compile_protos(
            &proto_files,
            &[proto_dir], // Include directories
        )?;

    // Tell Cargo to rerun this build script if proto files change
    println!("cargo:rerun-if-changed=../proto/authenc.proto");
    println!("cargo:rerun-if-changed=../proto/common.proto");

    Ok(())
}
