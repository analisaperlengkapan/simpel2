//! Dokumen module — document generation, templates, storage, OCR, archival.
//! Cross-module callers (workflow, pemakaian/penghapusan, /admin/templates)
//! reach this module through the
//! [`crate::contracts::DocumentGenerator`] +
//! [`crate::contracts::DocumentStorage`] traits wired into
//! [`AppState`](crate::state::AppState).

pub mod archive;
pub mod audit;
pub mod classify;
pub mod config;
pub mod docx_generator;
pub mod error;
pub mod excel_generator;
pub mod filesystem_storage;
pub mod handlers;
pub mod models;
pub mod ocr;
pub mod pdf_generator;
pub mod scheduler;
pub mod security;
pub mod service;
pub mod storage;
pub mod template_models;
pub mod template_service;

pub use docx_generator::DocxGenerator;
pub use excel_generator::ExcelGenerator;
pub use filesystem_storage::FilesystemStorage;
pub use pdf_generator::PdfGenerator;
pub use scheduler::DocumentScheduler;
pub use template_models::*;
pub use template_service::TemplateService;
