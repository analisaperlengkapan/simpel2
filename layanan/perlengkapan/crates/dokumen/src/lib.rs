pub mod archive;
pub mod audit;
pub mod classify;
pub mod config;
pub mod error;
pub mod handlers;
pub mod models;
pub mod ocr;
pub mod scheduler;
pub mod security;
pub mod storage;
pub mod template_models;
pub mod template_service;
pub mod pdf_generator;
pub mod excel_generator;
pub mod grpc_service;

// Re-exports
pub use template_models::*;
pub use template_service::TemplateService;
pub use pdf_generator::PdfGenerator;
pub use excel_generator::ExcelGenerator;
pub use grpc_service::{create_grpc_server, dokumen_proto};
pub use scheduler::DocumentScheduler;
