use std::path::PathBuf;

fn main() {
    // Get the proto directory
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let proto_dir = PathBuf::from(&manifest_dir).join("proto");

    // Compile common.proto first
    tonic_prost_build::configure()
        .build_server(false)
        .build_client(false)
        .compile_protos(
            &[proto_dir.join("common.proto")],
            std::slice::from_ref(&proto_dir),
        )
        .expect("Failed to compile common.proto");

    // Compile the main proto files using tonic_prost_build (for tonic 0.14)
    // Use extern_path to correctly reference common.v1 types
    tonic_prost_build::configure()
        .build_server(false)
        .build_client(true)
        .extern_path(".common.v1", "crate::grpc::generated::common_v1")
        .compile_protos(
            &[
                proto_dir.join("authenc.proto"),
                proto_dir.join("secreton.proto"),
            ],
            &[proto_dir],
        )
        .expect("Failed to compile proto files");

    // Rerun if proto files change
    println!("cargo:rerun-if-changed=proto/authenc.proto");
    println!("cargo:rerun-if-changed=proto/secreton.proto");
    println!("cargo:rerun-if-changed=proto/common.proto");
}
