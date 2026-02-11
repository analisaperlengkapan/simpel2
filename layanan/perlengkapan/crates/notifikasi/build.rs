use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let proto_dir = PathBuf::from("proto");
    let proto_file = proto_dir.join("notifikasi.proto");

    if !proto_file.exists() {
        println!("cargo::warning=Proto file not found at {:?}, skipping proto compilation", proto_file);
        return Ok(());
    }

    println!("cargo::rerun-if-changed=proto/notifikasi.proto");

    tonic_prost_build::configure()
        .build_server(true)
        .build_client(true)
        .compile_protos(&[proto_file], &[proto_dir])?;

    Ok(())
}
