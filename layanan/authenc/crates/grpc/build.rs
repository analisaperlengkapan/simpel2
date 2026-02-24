//! Build script for proto code generation

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Get proto directory
    let proto_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .map(|p| p.join("proto"))
        .unwrap_or_else(|| std::path::PathBuf::from("proto"));

    let proto_files = vec![
        proto_dir.join("authenc.proto"),
        proto_dir.join("common.proto"),
    ];

    // Only compile if proto files exist
    if proto_files.iter().all(|p| p.exists()) {
        tonic_prost_build::configure()
            .build_server(true)
            .build_client(false)
            .compile_protos(
                &proto_files
                    .iter()
                    .map(|p| p.to_string_lossy().to_string())
                    .collect::<Vec<_>>(),
                &[proto_dir.to_string_lossy().to_string()],
            )?;

        // Tell cargo to rerun if proto files change
        for proto_file in &proto_files {
            println!("cargo:rerun-if-changed={}", proto_file.display());
        }
    } else {
        println!("cargo:warning=Proto files not found, skipping gRPC compilation");
    }

    Ok(())
}
