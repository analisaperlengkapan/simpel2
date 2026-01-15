use std::process::Command;
use std::path::PathBuf;

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

    // Output directory for generated code
    let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());

    // Generate gRPC code from proto files
    tonic_prost_build::configure()
        .build_server(true)
        .build_client(false)
        .out_dir(&out_dir)
        .compile_protos(
            &["../../proto/secreton.proto"],
            &["../../proto"],
        )
        .expect("Failed to compile proto files");

    // Tell cargo to rerun if proto files change
    println!("cargo:rerun-if-changed=../../proto/secreton.proto");
    println!("cargo:rerun-if-changed=../../proto/common.proto");
}
