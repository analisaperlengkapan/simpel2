//! Charts via [`leptos_chartistry`] — dependency wired, project
//! wrappers land alongside concrete usage demand.
//!
//! `leptos-chartistry` is a `lib-ui` dependency (`Cargo.toml`), and
//! the workspace builds cleanly against it. Until a page actually
//! needs a richer chart than the in-house CSS `BarChart`/`PieChart`
//! in [`crate::components::dashboard`], call `chartistry::Chart`
//! directly with project styling rather than maintaining a
//! placeholder wrapper.
//!
//! Minimal example using `chartistry` from a consumer crate:
//!
//! ```ignore
//! use lib_ui::components::charts::chartistry;
//!
//! view! {
//!     <chartistry::Chart
//!         aspect_ratio=chartistry::AspectRatio::from_outer_ratio(600.0, 300.0)
//!         debug=false
//!         data=data_signal
//!         /* series, axes, colors … */
//!     />
//! }
//! ```
//!
//! When the second consumer appears, factor the shared theme out
//! into `<SimpelBarChart>` / `<SimpelLineChart>` here and migrate
//! the existing CSS-based charts in `dashboard.rs` to match.

#[doc(hidden)]
pub use leptos_chartistry as chartistry;
