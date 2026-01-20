use std::process::Command;

fn main() {
    // Set build date
    let build_date = chrono::Utc::now()
        .format("%Y-%m-%d %H:%M:%S UTC")
        .to_string();
    println!("cargo:rustc-env=BUILD_DATE={}", build_date);

    // Get git commit hash
    let git_commit = Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .ok()
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "unknown".to_string());
    println!("cargo:rustc-env=GIT_COMMIT={}", git_commit);

    // Get Rust version
    let rust_version = rustc_version::version()
        .map(|v| v.to_string())
        .unwrap_or_else(|_| "unknown".to_string());
    println!("cargo:rustc-env=RUST_VERSION={}", rust_version);

    // Generate gRPC code from proto files
    let proto_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .and_then(|p| p.parent())
        .map(|p| p.join("proto"))
        .unwrap_or_else(|| std::path::PathBuf::from("proto"));

    let proto_files = vec![
        proto_dir.join("secreton.proto"),
        proto_dir.join("common.proto"),
    ];

    // Only compile if proto files exist
    if proto_files.iter().all(|p| p.exists()) {
        tonic_build::configure()
            .build_server(true)
            .build_client(false)
            .compile_protos(
                &proto_files
                    .iter()
                    .map(|p| p.to_string_lossy().to_string())
                    .collect::<Vec<_>>(),
                &[proto_dir.to_string_lossy().to_string()],
            )
            .expect("Failed to compile proto files");

        // Tell cargo to rerun if proto files change
        for proto_file in &proto_files {
            println!("cargo:rerun-if-changed={}", proto_file.display());
        }
    } else {
        println!("cargo:warning=Proto files not found, skipping gRPC compilation");
    }
}
