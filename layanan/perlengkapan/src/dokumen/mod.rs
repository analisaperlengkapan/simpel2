//! Dokumen module — document generation, templates, storage, OCR, archival.
//!
//! Previously a separate crate (`layanan-perlengkapan-dokumen`); folded into
//! the unified service. The internal gRPC server (`grpc_service`) was dropped
//! because workflow no longer calls dokumen via gRPC — it will use the
//! [`lib_perlengkapan::contracts::DocumentGenerator`] trait once wired up.

pub mod archive;
pub mod audit;
pub mod classify;
pub mod config;
pub mod docx_generator;
pub mod error;
pub mod excel_generator;
pub mod handlers;
pub mod models;
pub mod ocr;
pub mod pdf_generator;
pub mod scheduler;
pub mod security;
pub mod storage;
pub mod service;
pub mod template_models;
pub mod template_service;

pub use docx_generator::DocxGenerator;
pub use excel_generator::ExcelGenerator;
pub use pdf_generator::PdfGenerator;
pub use scheduler::DocumentScheduler;
pub use template_models::*;
pub use template_service::TemplateService;
