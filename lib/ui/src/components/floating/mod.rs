//! Floating UI primitives backed by `floating-ui-leptos`.
//!
//! Project defaults: 8px offset, flip when clipped, shift to stay in
//! viewport, z-index from the `z-popover` Tailwind token.
//! Outside-click and Escape-key dismiss are wired automatically.
//!
//! Currently exports:
//! - [`Popover`] — controlled floating panel anchored to a trigger.
//!
//! Planned (will land alongside concrete usage demand):
//! - `Tooltip` — hover / focus styled wrapper around `Popover`.
//! - `Dropdown` — `Popover` + menu role + keyboard navigation.

pub mod popover;

pub use popover::Popover;

#[doc(hidden)]
pub use floating_ui_leptos;
