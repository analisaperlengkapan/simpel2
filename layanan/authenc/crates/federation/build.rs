use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let proto_dir = PathBuf::from("../../proto");

    let proto_file = proto_dir.join("integrasi.proto");

    if !proto_file.exists() {
        println!(
            "cargo::warning=Proto file not found at {:?}, skipping proto compilation",
            proto_file
        );
        return Ok(());
    }

    println!("cargo::rerun-if-changed=../../proto/integrasi.proto");

    tonic_prost_build::configure()
        .build_server(false)
        .build_client(true)
        .compile_protos(&[proto_file], &[proto_dir])?;

    println!("cargo::warning=Successfully compiled integrasi.proto for federation client");

    Ok(())
}
