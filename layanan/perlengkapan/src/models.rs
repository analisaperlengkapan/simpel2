//! # Data Models for Perlengkapan Service

// ============ Response Models ============
//
// `ApiResponse<T>` + `PaginatedResponse<T>` live in `lib_perlengkapan` so
// the frontend (Leptos WASM) and the backend share one definition + one
// wire format. The constructors are inherent impls on the lib types, so
// callers keep using `ApiResponse::success(...)` / `PaginatedResponse::new(...)`
// unchanged after the re-export.

pub use lib_perlengkapan::response::{ApiResponse, PaginatedResponse};
