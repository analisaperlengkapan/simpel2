//! # Upload validation (OWASP File Upload Cheat Sheet)
//!
//! What may be stored, decided from the **content**, not from what the client
//! claims about it.
//!
//! The handlers used to accept any bytes under any name and store them with the
//! client's own `Content-Type`. A `.html` or `.svg` uploaded as
//! `application/pdf` (or with no type at all) would later be served back to
//! another user — stored XSS from a file the application vouched for. So:
//!
//! * an **allowlist** of extensions (documents and images the workflow actually
//!   uses), no others;
//! * the file's **magic bytes** must match the extension it claims — a renamed
//!   executable or HTML page is refused;
//! * the stored `Content-Type` is the one **we** derive from the extension, never
//!   the client's;
//! * a hard **size** ceiling per file ([`MAX_FILE_BYTES`]) and a cap on the
//!   number of files in one request ([`MAX_FILES_PER_REQUEST`]);
//! * empty files are refused.
//!
//! Magic-byte sniffing is a floor, not a malware scan: it stops the trivially
//! mislabelled file, not a well-formed PDF with a hostile payload. Antivirus is an
//! infrastructure concern (ClamAV sidecar) and is out of scope here.

use crate::shared::error::AppError;

/// Largest single file accepted, in bytes (10 MiB — equal to the request-body
/// ceiling, so a single file is never accepted by this check and then refused by
/// the transport).
pub const MAX_FILE_BYTES: usize = 10 * 1024 * 1024;
/// Most files one request may carry.
pub const MAX_FILES_PER_REQUEST: usize = 20;

/// One accepted kind of file.
struct Kind {
    ext: &'static str,
    content_type: &'static str,
    /// Leading bytes every file of this kind starts with.
    magic: &'static [&'static [u8]],
}

const ZIP_MAGIC: &[&[u8]] = &[b"PK\x03\x04"];

const ALLOWED: &[Kind] = &[
    Kind {
        ext: "pdf",
        content_type: "application/pdf",
        magic: &[b"%PDF-"],
    },
    Kind {
        ext: "png",
        content_type: "image/png",
        magic: &[b"\x89PNG\r\n\x1a\n"],
    },
    Kind {
        ext: "jpg",
        content_type: "image/jpeg",
        magic: &[b"\xFF\xD8\xFF"],
    },
    Kind {
        ext: "jpeg",
        content_type: "image/jpeg",
        magic: &[b"\xFF\xD8\xFF"],
    },
    Kind {
        ext: "docx",
        content_type: "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
        magic: ZIP_MAGIC,
    },
    Kind {
        ext: "xlsx",
        content_type: "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
        magic: ZIP_MAGIC,
    },
];

/// A file that passed [`validate_upload`].
#[derive(Debug, PartialEq, Eq)]
pub struct ValidUpload {
    /// The lower-cased extension, from the allowlist.
    pub ext: &'static str,
    /// The `Content-Type` to store and serve — derived, never the client's.
    pub content_type: &'static str,
}

/// The extension of `filename`, lower-cased; `None` when there is none.
fn extension(filename: &str) -> Option<String> {
    let name = filename.rsplit(['/', '\\']).next().unwrap_or(filename);
    let (stem, ext) = name.rsplit_once('.')?;
    if stem.is_empty() || ext.is_empty() {
        return None;
    }
    Some(ext.to_ascii_lowercase())
}

/// Decide whether `bytes`, uploaded as `filename`, may be stored.
///
/// `declared_content_type` is what the client said. It is only used to refuse a
/// contradiction (a `.pdf` that announces itself as `text/html`); it is never
/// stored.
pub fn validate_upload(
    filename: &str,
    declared_content_type: Option<&str>,
    bytes: &[u8],
) -> Result<ValidUpload, AppError> {
    if bytes.is_empty() {
        return Err(AppError::BadRequest(format!("File '{filename}' kosong")));
    }
    if bytes.len() > MAX_FILE_BYTES {
        return Err(AppError::PayloadTooLarge(format!(
            "File '{filename}' melebihi batas {} MB",
            MAX_FILE_BYTES / (1024 * 1024)
        )));
    }

    let ext = extension(filename).ok_or_else(|| {
        AppError::BadRequest(format!(
            "File '{filename}' tidak memiliki ekstensi; jenis yang diizinkan: {}",
            allowed_list()
        ))
    })?;
    let kind = ALLOWED.iter().find(|k| k.ext == ext).ok_or_else(|| {
        AppError::BadRequest(format!(
            "Jenis file '.{ext}' tidak diizinkan; jenis yang diizinkan: {}",
            allowed_list()
        ))
    })?;

    if !kind.magic.iter().any(|m| bytes.starts_with(m)) {
        return Err(AppError::BadRequest(format!(
            "Isi file '{filename}' tidak sesuai dengan ekstensi '.{ext}'"
        )));
    }

    if let Some(declared) = declared_content_type {
        let declared = declared
            .split(';')
            .next()
            .unwrap_or("")
            .trim()
            .to_ascii_lowercase();
        // `application/octet-stream` and a missing type are what browsers send for
        // anything they cannot classify; they contradict nothing. Everything else
        // must agree with what the extension (and the bytes) say it is.
        let agrees = declared.is_empty()
            || declared == "application/octet-stream"
            || ALLOWED
                .iter()
                .filter(|k| k.ext == kind.ext || k.content_type == kind.content_type)
                .any(|k| k.content_type == declared);
        if !agrees {
            return Err(AppError::BadRequest(format!(
                "Tipe konten '{declared}' tidak sesuai dengan file '.{ext}'"
            )));
        }
    }

    Ok(ValidUpload {
        ext: kind.ext,
        content_type: kind.content_type,
    })
}

/// Longest document URL stored.
pub const MAX_URL_CHARS: usize = 2048;

/// Validate a document URL the client supplies (`signed_pdf_url`,
/// `lampiran_persyaratan`, …), which is stored and later rendered as a link.
///
/// Only `https://` and `http://` URLs, or an absolute path on this site
/// (`/…`), are accepted. `javascript:`, `data:`, `file:` and friends are refused:
/// a stored `javascript:` URL is a link that runs script when another user — the
/// validator reviewing the usulan — clicks it. No embedded credentials.
pub fn validate_document_url(field: &str, url: &str) -> Result<(), AppError> {
    let url = url.trim();
    if url.is_empty() {
        return Err(AppError::BadRequest(format!("{field} wajib diisi")));
    }
    if url.chars().count() > MAX_URL_CHARS {
        return Err(AppError::BadRequest(format!(
            "{field} terlalu panjang (maksimal {MAX_URL_CHARS} karakter)"
        )));
    }
    if url.chars().any(|c| c.is_control() || c.is_whitespace()) {
        return Err(AppError::BadRequest(format!(
            "{field} tidak boleh mengandung spasi atau karakter kontrol"
        )));
    }

    let lower = url.to_ascii_lowercase();
    let rest = if let Some(r) = lower.strip_prefix("https://") {
        r
    } else if let Some(r) = lower.strip_prefix("http://") {
        r
    } else if lower.starts_with('/') && !lower.starts_with("//") {
        // A path on this origin; "//host/x" is protocol-relative, i.e. off-site.
        return Ok(());
    } else {
        return Err(AppError::BadRequest(format!(
            "{field} harus berupa URL http(s) atau path situs ini"
        )));
    };

    let authority = rest.split(['/', '?', '#']).next().unwrap_or("");
    if authority.is_empty() || authority.contains('@') {
        return Err(AppError::BadRequest(format!(
            "{field} bukan URL yang valid (host kosong atau memuat kredensial)"
        )));
    }
    Ok(())
}

fn allowed_list() -> String {
    ALLOWED.iter().map(|k| k.ext).collect::<Vec<_>>().join(", ")
}

#[cfg(test)]
mod tests {
    use super::*;

    const PDF: &[u8] = b"%PDF-1.7\n1 0 obj\n<<>>\nendobj\n";
    const PNG: &[u8] = b"\x89PNG\r\n\x1a\n\0\0\0\rIHDR";
    const ZIP: &[u8] = b"PK\x03\x04\x14\0\0\0";

    #[test]
    fn genuine_documents_are_accepted_with_a_server_chosen_type() {
        let pdf = validate_upload("Surat Usulan.PDF", Some("application/pdf"), PDF).unwrap();
        assert_eq!(pdf.ext, "pdf");
        assert_eq!(pdf.content_type, "application/pdf");

        assert_eq!(
            validate_upload("foto.png", None, PNG).unwrap().content_type,
            "image/png"
        );
        assert_eq!(
            validate_upload("foto.JPG", Some("image/jpeg"), b"\xFF\xD8\xFF\xE0jfif")
                .unwrap()
                .ext,
            "jpg"
        );
        assert_eq!(validate_upload("dok.docx", None, ZIP).unwrap().ext, "docx");
        assert_eq!(validate_upload("dok.xlsx", None, ZIP).unwrap().ext, "xlsx");
    }

    /// The classic: markup uploaded with a document's name and type. If it were
    /// stored and later served, it would run in the application's origin.
    #[test]
    fn html_and_script_dressed_as_a_pdf_are_refused() {
        for body in [
            &b"<html><script>alert(1)</script></html>"[..],
            b"<svg xmlns='http://www.w3.org/2000/svg' onload='alert(1)'/>",
            b"MZ\x90\x00\x03\x00\x00\x00",
            b"#!/bin/sh\nrm -rf /\n",
        ] {
            assert!(
                validate_upload("laporan.pdf", Some("application/pdf"), body).is_err(),
                "{:?} must not pass as a PDF",
                String::from_utf8_lossy(body)
            );
        }
    }

    #[test]
    fn only_allowlisted_extensions_are_accepted_whatever_the_bytes() {
        for name in [
            "a.html", "a.svg", "a.exe", "a.js", "a.php", "a.phtml", "a.sh", "a.zip", "a.txt", "a",
        ] {
            assert!(
                validate_upload(name, None, PDF).is_err(),
                "{name} must be refused"
            );
        }
        // A double extension is judged by the LAST one, and must still match.
        assert!(validate_upload("evil.php.pdf", None, PDF).is_ok());
        assert!(validate_upload("evil.pdf.php", None, PDF).is_err());
        // Path components in the client's filename do not matter (and do not confuse).
        assert!(validate_upload("../../etc/passwd", None, PDF).is_err());
        assert!(validate_upload("C:\\temp\\a.pdf", None, PDF).is_ok());
        // A dotfile has no extension.
        assert!(validate_upload(".pdf", None, PDF).is_err());
    }

    #[test]
    fn a_contradicting_declared_type_is_refused_but_absence_is_not() {
        assert!(validate_upload("a.pdf", Some("text/html"), PDF).is_err());
        assert!(validate_upload("a.pdf", Some("image/png"), PDF).is_err());
        assert!(validate_upload("a.pdf", Some("application/pdf; charset=binary"), PDF).is_ok());
        assert!(validate_upload("a.pdf", Some("application/octet-stream"), PDF).is_ok());
        assert!(validate_upload("a.pdf", None, PDF).is_ok());
        assert!(validate_upload("a.pdf", Some(""), PDF).is_ok());
    }

    #[test]
    fn only_web_urls_and_site_paths_are_stored_as_document_links() {
        for ok in [
            "https://storage.example.com/l.pdf",
            "http://localhost:9000/a/b.pdf?x=1#p",
            "/api/v1/perlengkapan/files/abc.pdf",
            "HTTPS://Example.COM/x",
        ] {
            assert!(validate_document_url("url", ok).is_ok(), "{ok}");
        }
        for bad in [
            "",
            "   ",
            "javascript:alert(1)",
            "JaVaScRiPt:alert(1)",
            "data:text/html;base64,PHNjcmlwdD4=",
            "file:///etc/passwd",
            "ftp://x/y",
            "//evil.example/x.pdf",
            "https://",
            "https:///path",
            "https://user:pass@evil.example/x",
            "https://a.example/x y",
            "https://a.example/x\nSet-Cookie: a=b",
            "vbscript:msgbox(1)",
        ] {
            assert!(
                validate_document_url("url", bad).is_err(),
                "{bad:?} must be refused"
            );
        }
        let long = format!("https://a.example/{}", "x".repeat(MAX_URL_CHARS));
        assert!(validate_document_url("url", &long).is_err());
    }

    #[test]
    fn empty_and_oversized_files_are_refused() {
        assert!(validate_upload("a.pdf", None, b"").is_err());

        let mut big = PDF.to_vec();
        big.resize(MAX_FILE_BYTES + 1, b'x');
        assert!(matches!(
            validate_upload("a.pdf", None, &big),
            Err(AppError::PayloadTooLarge(_))
        ));
        big.truncate(MAX_FILE_BYTES);
        assert!(validate_upload("a.pdf", None, &big).is_ok());
    }

    #[test]
    fn a_zip_container_is_not_a_pdf_and_a_pdf_is_not_an_office_file() {
        assert!(validate_upload("a.pdf", None, ZIP).is_err());
        assert!(validate_upload("a.docx", None, PDF).is_err());
        assert!(validate_upload("a.png", None, PDF).is_err());
    }
}
