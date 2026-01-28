fn main() -> Result<(), Box<dyn std::error::Error>> {
    let proto_root = "../../../infra";
    let proto_files = &[
        format!("{}/secreton/proto/secreton.proto", proto_root),
        format!("{}/authenc/proto/authenc.proto", proto_root),
        format!("{}/secreton/proto/common.proto", proto_root), // Ensure common is included if needed explicitly, though usually implicitly via includes
    ];

    let includes = &[
        format!("{}/secreton/proto", proto_root),
        format!("{}/authenc/proto", proto_root),
    ];

    tonic_build::configure()
        .build_server(false)
        .build_client(true)
        .compile_protos(
            proto_files,
            includes,
        )?;

    Ok(())
}
