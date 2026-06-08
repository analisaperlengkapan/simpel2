//! [`leptos_struct_table`]-backed typed table primitives.
//!
//! `leptos-struct-table` (with `chrono` + `uuid` features) provides the
//! `#[derive(TableRow)]` macro and the [`TableContent`] component; this module
//! adds the project theme and a curated re-export surface so feature code can
//! build typed, sortable tables without depending on `leptos_struct_table`
//! directly.
//!
//! ## Usage
//! ```ignore
//! use lib_ui::components::data_table_v2::{SimpelTableClasses, TableContent, TableRow};
//!
//! #[derive(TableRow, Clone)]
//! #[table(classes_provider = "SimpelTableClasses")]
//! struct AssetRow {
//!     kode: String,
//!     nama: String,
//! }
//!
//! view! { <TableContent rows scroll_container="html" /> }
//! ```
//!
//! ## Follow-up (needs a concrete consumer to validate generics)
//! - A thin opinionated `<DataTable rows=.../>` wrapper that injects
//!   [`SimpelTableClasses`] and sensible defaults — deferred until the first
//!   feature migrates so the generic `TableRow` bounds can be verified against
//!   a real row type.
//! - Collapse `components::display::Table` into a re-export and delete
//!   `antarmuka/perlengkapan/src/components/layout/data_table.rs`.

mod class_providers;

pub use class_providers::SimpelTableClasses;

// Curated re-exports: the building blocks consumers need to declare and render
// a typed table, so feature code imports from `lib_ui`, not the upstream crate.
pub use leptos_struct_table::{
    ColumnSort, DefaultTableCellRenderer, DefaultTableRowRenderer, TableClassesProvider,
    TableContent, TableRow,
};

#[doc(hidden)]
pub use leptos_struct_table;
