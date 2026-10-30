//! Verifies DocxGenerator's output survives the docx-rs zip 9 patch:
//! it must still be a readable ZIP container holding the OOXML parts.
use std::io::Read;

#[tokio::test]
async fn docx_output_is_a_valid_ooxml_archive() {
    use layanan_perlengkapan::dokumen::{DocxGenerator, TemplateService};

    let generator = DocxGenerator::new(TemplateService::new());
    let template = layanan_perlengkapan::dokumen::DocumentTemplate {
        id: uuid::Uuid::new_v4(),
        name: "t".into(),
        description: None,
        template_type: "konsep_sk".into(),
        content: "<p>Nomor: {{nomor}}</p>\n<p>Kepada {{nama}}</p>".into(),
        format: "html".into(),
        output_format: "docx".into(),
        version: 1,
        is_active: true,
        parent_template_id: None,
        variables: serde_json::json!({}),
        sample_data: None,
        letterhead_config: None,
        created_by: uuid::Uuid::new_v4(),
        created_at: chrono::Utc::now(),
        updated_by: None,
        updated_at: chrono::Utc::now(),
    };
    let data = serde_json::json!({ "nomor": "SK-1/2026", "nama": "Budi" });

    let bytes = generator.generate_docx(&template, &data, "").await.unwrap();
    assert!(!bytes.is_empty(), "generator returned no bytes");

    let mut archive = zip::ZipArchive::new(std::io::Cursor::new(bytes)).expect("not a zip archive");
    let mut names: Vec<String> = (0..archive.len())
        .map(|i| archive.by_index(i).unwrap().name().unwrap().to_string())
        .collect();
    names.sort();
    assert!(
        names.iter().any(|n| n == "word/document.xml"),
        "missing word/document.xml, got {names:?}"
    );

    let mut body = String::new();
    archive
        .by_name("word/document.xml")
        .unwrap()
        .read_to_string(&mut body)
        .unwrap();
    assert!(body.contains("SK-1/2026"), "rendered data missing: {body}");
    assert!(body.contains("Budi"), "rendered data missing: {body}");
}
