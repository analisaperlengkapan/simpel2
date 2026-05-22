//! Pagination query parameters and result types.

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

pub const DEFAULT_PER_PAGE: u32 = 20;
pub const MAX_PER_PAGE: u32 = 200;

/// Pagination query parameters parsed from URL query strings.
///
/// Both fields are optional in the wire format; defaults are applied via
/// [`PageParams::resolved`].
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct PageParams {
    pub page: Option<u32>,
    pub per_page: Option<u32>,
}

impl PageParams {
    /// Returns `(page, per_page)` with defaults applied and bounds clamped.
    /// - `page` is clamped to minimum 1.
    /// - `per_page` is clamped to `[1, MAX_PER_PAGE]`.
    pub fn resolved(self) -> (u32, u32) {
        let page = self.page.unwrap_or(1).max(1);
        let per_page = self
            .per_page
            .unwrap_or(DEFAULT_PER_PAGE)
            .clamp(1, MAX_PER_PAGE);
        (page, per_page)
    }

    /// SQL OFFSET corresponding to `(page - 1) * per_page`.
    pub fn offset(self) -> u64 {
        let (page, per_page) = self.resolved();
        ((page - 1) as u64) * (per_page as u64)
    }

    /// SQL LIMIT.
    pub fn limit(self) -> u64 {
        self.resolved().1 as u64
    }
}

/// A page of results plus the total row count (for downstream `PageMeta`).
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PageResult<T> {
    pub items: Vec<T>,
    pub total: u64,
}

impl<T> PageResult<T> {
    pub fn new(items: Vec<T>, total: u64) -> Self {
        Self { items, total }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_applied_when_none() {
        let p = PageParams::default();
        assert_eq!(p.resolved(), (1, DEFAULT_PER_PAGE));
        assert_eq!(p.offset(), 0);
        assert_eq!(p.limit(), DEFAULT_PER_PAGE as u64);
    }

    #[test]
    fn page_clamped_to_min_one() {
        let p = PageParams {
            page: Some(0),
            per_page: Some(10),
        };
        assert_eq!(p.resolved(), (1, 10));
    }

    #[test]
    fn per_page_clamped_to_max() {
        let p = PageParams {
            page: Some(1),
            per_page: Some(MAX_PER_PAGE + 1000),
        };
        assert_eq!(p.resolved(), (1, MAX_PER_PAGE));
    }

    #[test]
    fn offset_calculation() {
        let p = PageParams {
            page: Some(3),
            per_page: Some(20),
        };
        assert_eq!(p.offset(), 40);
    }
}
