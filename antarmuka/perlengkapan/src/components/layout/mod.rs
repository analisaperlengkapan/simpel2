//! Shared layout primitives for Perlengkapan pages.
//!
//! Every page in the refactored codebase wraps its content in `PageLayout`
//! and composes `SectionCard`, `StatCard`, `FormLayout`, `EmptyState`,
//! `LoadingState`, `ErrorState`, and `DataTable` from this module instead of
//! hand-rolling markup with hex colors or inline styles. Contract tests in
//! `antarmuka/perlengkapan/tests/` enforce that page files only reference
//! these primitives plus `lib_ui::prelude::*`.

mod empty_state;
mod error_state;
mod form_layout;
mod loading_state;
mod page_layout;
mod section_card;
mod stat_card;

pub use empty_state::EmptyState;
pub use error_state::ErrorState;
pub use form_layout::FormField;
pub use loading_state::LoadingState;
pub use page_layout::{PageBreadcrumb, PageLayout};
pub use section_card::SectionCard;
pub use stat_card::{StatCard, StatTone};
