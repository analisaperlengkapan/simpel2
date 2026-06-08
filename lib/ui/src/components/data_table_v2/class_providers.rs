//! [`SimpelTableClasses`] — the project's Navy/Gold Tailwind theme for
//! `leptos_struct_table`'s [`TableContent`](leptos_struct_table::TableContent).
//!
//! Pass it as the class provider type parameter:
//!
//! ```ignore
//! use leptos_struct_table::TableContent;
//! use lib_ui::components::data_table_v2::SimpelTableClasses;
//!
//! view! {
//!     <TableContent rows scroll_container="html" />
//!     // ...with `#[table(classes_provider = SimpelTableClasses)]` on the
//!     // `#[derive(TableRow)]` struct, or the prop on TableContent.
//! }
//! ```

use leptos_struct_table::{ColumnSort, TableClassesProvider};

/// Tailwind class provider emitting the SIMPEL table look: light header,
/// zebra-free hover rows, gold (`amber`) selection highlight, comfortable
/// padding. Field-level `class`/`head_class` macro attributes are appended,
/// so per-column overrides still win.
#[derive(Clone, Copy, Debug, Default)]
pub struct SimpelTableClasses;

impl TableClassesProvider for SimpelTableClasses {
    fn new() -> Self {
        Self
    }

    fn thead(&self, prop_class: &str) -> String {
        format!("bg-gray-50 border-b border-gray-200 {prop_class}")
    }

    fn thead_cell(&self, sort: ColumnSort, macro_class: &str) -> String {
        format!(
            "px-4 py-2.5 text-left text-xs font-semibold uppercase tracking-wider \
             text-gray-600 select-none {} {macro_class}",
            sort.as_class()
        )
    }

    fn tbody(&self, prop_class: &str) -> String {
        format!("divide-y divide-gray-200 {prop_class}")
    }

    fn row(&self, _row_index: usize, selected: bool, prop_class: &str) -> String {
        let state = if selected {
            "bg-amber-50"
        } else {
            "hover:bg-gray-50"
        };
        format!("transition-colors {state} {prop_class}")
    }

    fn cell(&self, macro_class: &str) -> String {
        format!("px-4 py-2.5 text-sm text-gray-800 {macro_class}")
    }
}
