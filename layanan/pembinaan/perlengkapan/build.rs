fn main() {
    // Stub build script to avoid tonic compilation errors
    // TODO: Restore this when tonic/axum http::Request type mismatch is resolved
    /*
    use std::env;
    use std::path::PathBuf;

    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let proto_root = "../../../infra/secreton/proto";
    let proto_files = &[format!("{}/secreton.proto", proto_root)];

    tonic_build::configure()
        .build_server(false)
        .build_client(true)
        .compile(proto_files, &[proto_root]).unwrap();
    */
}
