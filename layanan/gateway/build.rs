//! Build script: compile the upstream protos this gateway proxies.
//!
//! The gateway is a gRPC **client** to authenc, secreton and integrasi; it
//! exposes a thin REST surface that simpelv1's PHP clients consume over
//! localhost (see `monolith/simpelv1/AGENTS.md`). We reuse the canonical proto
//! files in-repo rather than vendoring copies, mirroring
//! `layanan/perlengkapan/build.rs` (same proto set ⇒ identical generated
//! clients, so the gateway and perlengkapan stay in lockstep).

use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let authenc_proto_dir = PathBuf::from("../../layanan/authenc/proto");
    let secreton_proto_dir = PathBuf::from("../../layanan/secreton/proto");
    let integrasi_proto_dir = PathBuf::from("../../layanan/integrasi/proto");

    if !authenc_proto_dir.exists() {
        println!("cargo:warning=authenc proto dir not found, skipping proto compilation");
        return Ok(());
    }

    let mut proto_files = vec![
        authenc_proto_dir.join("authenc.proto"),
        authenc_proto_dir.join("secreton.proto"),
        authenc_proto_dir.join("common.proto"),
    ];

    let integrasi_proto = integrasi_proto_dir.join("integrasi.proto");
    if integrasi_proto.exists() {
        proto_files.push(integrasi_proto);
    } else {
        println!(
            "cargo:warning=integrasi.proto not found, gateway integrasi routes will be unavailable"
        );
    }

    tonic_prost_build::configure()
        .build_server(false)
        .build_client(true)
        .type_attribute(".", "#[derive(serde::Serialize, serde::Deserialize)]")
        .compile_protos(
            &proto_files,
            &[
                // integrasi dir FIRST: both it and authenc/proto contain an
                // integrasi.proto; the integrasi service's copy is canonical.
                integrasi_proto_dir,
                authenc_proto_dir.clone(),
                secreton_proto_dir,
            ],
        )?;

    for proto_file in &proto_files {
        if proto_file.exists() {
            println!("cargo:rerun-if-changed={}", proto_file.display());
        }
    }

    Ok(())
}
