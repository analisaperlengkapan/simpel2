//! # Analisis Kebutuhan
//!
//! CRUD over `perlengkapan.analisis_kebutuhan` (the `/analitik/roadmap` UI).
//! Extracted from the former top-level catch-all into a self-contained feature
//! module (F0-A): `models` · `repository` (trait + `Database` impl) · `services`
//! · `handlers`.

pub mod handlers;
pub mod models;
pub mod repository;
pub mod services;

pub use handlers::{create_analisis, get_all_analisis, get_analisis_by_id};
pub use models::{AnalisisKebutuhan, CreateAnalisisRequest};
pub use repository::AnalisisRepository;
pub use services::AnalisisService;

#[cfg(test)]
mod tests;
