//! Build script for compiling Protocol Buffer definitions
//!
//! Compiles authenc.proto, secreton.proto, and integrasi.proto for gRPC client generation.

use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Navigate from layanan/perlengkapan to root
    let authenc_proto_dir = PathBuf::from("../../layanan/authenc/proto");
    let secreton_proto_dir = PathBuf::from("../../layanan/secreton/proto");
    let integrasi_proto_dir = PathBuf::from("../../layanan/integrasi/proto");
    // Internal protos kept temporarily while the workflow ↔ dokumen /
    // notifikasi gRPC clients have not yet been replaced with
    // `lib_perlengkapan::contracts::{DocumentGenerator, NotificationSender}`
    // trait calls. They will be deleted in the trait-wiring commit along
    // with `workflow/{dokumen,notifikasi}_client.rs`.
    let dokumen_proto_dir = PathBuf::from("proto");
    let notifikasi_proto_dir = PathBuf::from("proto");

    // Verify proto directories exist
    if !authenc_proto_dir.exists() {
        eprintln!(
            "Warning: Authenc proto directory not found: {:?}",
            authenc_proto_dir
        );
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
        println!(
            "cargo:warning=integrasi.proto not found at {:?}, skipping",
            integrasi_proto_dir
        );
    }

    // Add dokumen proto if exists
    let dokumen_proto = dokumen_proto_dir.join("dokumen.proto");
    if dokumen_proto.exists() {
        proto_files.push(dokumen_proto);
        println!("cargo:warning=Including dokumen.proto");
    } else {
        println!(
            "cargo:warning=dokumen.proto not found at {:?}, skipping",
            dokumen_proto_dir
        );
    }

    // Add notifikasi proto if exists
    let notifikasi_proto = notifikasi_proto_dir.join("notifikasi.proto");
    if notifikasi_proto.exists() {
        proto_files.push(notifikasi_proto);
        println!("cargo:warning=Including notifikasi.proto");
    } else {
        println!(
            "cargo:warning=notifikasi.proto not found at {:?}, skipping",
            notifikasi_proto_dir
        );
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
        .compile_protos(
            &proto_files,
            &[
                // integrasi_proto_dir must come before authenc to avoid
                // shadowing (both contain integrasi.proto)
                integrasi_proto_dir,
                dokumen_proto_dir,
                notifikasi_proto_dir,
                authenc_proto_dir.clone(),
                secreton_proto_dir,
            ],
        )?;

    // Tell Cargo to rerun this build script if proto files change
    for proto_file in &proto_files {
        if proto_file.exists() {
            println!("cargo:rerun-if-changed={}", proto_file.display());
        }
    }

    Ok(())
}
