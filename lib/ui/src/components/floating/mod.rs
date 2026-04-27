//! Floating UI primitives — placeholder module.
//!
//! `floating-ui-leptos` (`use_floating`, `Offset`, `Flip`, `Shift`,
//! `Arrow`) is wired up as a `lib-ui` dependency so positioning logic
//! is one `use` away when we need it. Pre-built `Tooltip`, `Popover`,
//! and `Dropdown` components are intentionally **deferred** until the
//! workspace build is healthy enough to verify them — see the 62
//! pre-existing errors in `lib/ui/src/hooks/use_form.rs`,
//! `lib/ui/src/components/error_boundary.rs`, and
//! `lib/crypto/src/shamir.rs` (Send + Sync bound regression).
//!
//! Until then, simple dropdowns can use Tailwind absolute positioning
//! with the centralized z-index tokens (`z-dropdown`, `z-popover`,
//! `z-tooltip`, `z-modal`, `z-toast`) defined in
//! `tailwind.config.js`. Combine with `leptos_use::on_click_outside`
//! (re-exported from `lib_ui::prelude`) for outside-click dismiss.
//!
//! When primitives land, this module will export at minimum:
//! - `Tooltip` — hover / focus, positioned via `use_floating`.
//! - `Popover` — controlled panel with click-outside + Escape dismiss.
//! - `Dropdown` — `Popover` + menu role + keyboard navigation.
//!
//! Tracked: phase 4 in
//! `.claude/plans/coba-kritisi-uraian-berikut-pure-rossum.md`.

#[doc(hidden)]
pub use floating_ui_leptos;
