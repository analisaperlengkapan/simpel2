//! Shared pagination query DTO.
//!
//! `PaginationQuery` is cross-cutting: it is consumed by several feature
//! modules (`analisis`, `pakaian_dinas`, …) as the `?page=&per_page=&category=`
//! extractor, so it lives in `shared` rather than inside any one feature.
//! (Some features — `kebutuhan_bmn`, `penghapusan_bmn` — define their own
//! richer pagination types; those are intentionally separate.)

use serde::Deserialize;

use crate::shared::error::{AppError, bad_request};

/// Standard pagination query parameters: `?page=&per_page=&category=`.
/// `page` / `per_page` go through [`lib_core::serde_query::de_i32`] because
/// this type is `#[serde(flatten)]`ed into the pakaian-dinas query DTOs, and
/// flatten strips the type hint serde_urlencoded needs — every `?page=` request
/// answered 400 until 2026-08-24. See that module for the measured boundary.
#[derive(Debug, Deserialize)]
pub struct PaginationQuery {
    #[serde(
        default = "default_page",
        deserialize_with = "lib_core::serde_query::de_i32"
    )]
    pub page: i32,
    #[serde(
        default = "default_per_page",
        deserialize_with = "lib_core::serde_query::de_i32"
    )]
    pub per_page: i32,
    pub category: Option<String>,
}

impl PaginationQuery {
    pub fn validate(&self) -> Result<(), AppError> {
        if self.page < 1 {
            return Err(bad_request("Page must be greater than 0"));
        }
        if self.page > 100_000 {
            return Err(bad_request("Page must be less than or equal to 100,000"));
        }
        if self.per_page < 1 {
            return Err(bad_request("Per page must be greater than 0"));
        }
        if self.per_page > 1000 {
            return Err(bad_request("Per page must be less than or equal to 1000"));
        }
        Ok(())
    }
}

fn default_page() -> i32 {
    1
}

fn default_per_page() -> i32 {
    20
}
