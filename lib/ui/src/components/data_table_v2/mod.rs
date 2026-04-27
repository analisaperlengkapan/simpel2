//! Future home of the [`leptos_struct_table`]-backed `<DataTable>`.
//!
//! `leptos-struct-table` is wired up as a `lib-ui` dependency (with
//! `chrono` and `uuid` features) so derive macros for our domain
//! types resolve. The actual `<DataTable>` wrapper is intentionally
//! deferred — see Phase 5a in
//! `.claude/plans/coba-kritisi-uraian-berikut-pure-rossum.md`.
//!
//! Once the workspace build is healthy enough to verify the
//! migration, this module will export:
//! - `<DataTable rows=... class_provider=SimpelTableClasses />` — a
//!   thin theme wrapper over `leptos_struct_table::TableContent`.
//! - A `class_providers::SimpelTableClasses` struct that emits the
//!   project's Navy/Gold Tailwind classes for header/row/cell cells.
//!
//! At that point:
//! - `lib/ui/src/components/display.rs::Table` collapses into a
//!   `pub use` re-export.
//! - `antarmuka/perlengkapan/src/components/layout/data_table.rs` is
//!   deleted in favor of this typed primitive.
//! - `pagination_controls.rs` either moves here or is replaced by
//!   `leptos_struct_table`'s built-in pagination.

#[doc(hidden)]
pub use leptos_struct_table;
