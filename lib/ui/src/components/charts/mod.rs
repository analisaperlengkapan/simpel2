//! Future home of the [`leptos_chartistry`]-backed `<SimpelBarChart>`
//! and `<SimpelPieChart>`.
//!
//! `leptos-chartistry` is wired up as a `lib-ui` dependency so the
//! migration target is one `use` away. The actual chart wrappers are
//! intentionally deferred — see Phase 5b in
//! `.claude/plans/coba-kritisi-uraian-berikut-pure-rossum.md`.
//!
//! Once the workspace build is healthy enough to verify the
//! migration, this module will export:
//! - `<SimpelBarChart data=... />` — replaces the CSS-conic-gradient
//!   `BarChart` in [`crate::components::dashboard`].
//! - `<SimpelPieChart data=... />` — replaces the equivalent
//!   CSS-only `PieChart`.
//!
//! Both wrappers will apply the project's Navy/Gold theme tokens
//! (`gold-400`/`navy-800`/`success-500`/etc.) so charts render
//! consistently with the rest of the UI without the consumer
//! configuring colors per-call.

#[doc(hidden)]
pub use leptos_chartistry;
