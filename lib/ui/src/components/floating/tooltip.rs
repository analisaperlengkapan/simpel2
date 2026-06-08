//! `<Tooltip>` — hover / focus text label, built on [`Popover`].
//!
//! The trigger element(s) are passed as `children`; `Tooltip` wraps them
//! in a focusable inline span that opens the bubble on `mouseenter` /
//! `focusin` and closes it on `mouseleave` / `focusout`. The bubble is a
//! [`Popover`] with `role="tooltip"`, and the trigger is wired to it via
//! `aria-describedby` for screen-reader users. Escape-dismiss and viewport
//! flip/shift come for free from `Popover`.
//!
//! ```ignore
//! use lib_ui::components::floating::Tooltip;
//! use leptos::prelude::*;
//!
//! #[component]
//! fn DeleteButton() -> impl IntoView {
//!     view! {
//!         <Tooltip text="Hapus permanen">
//!             <button class="icon-btn">"🗑"</button>
//!         </Tooltip>
//!     }
//! }
//! ```

use floating_ui_leptos::Placement;
use leptos::prelude::*;
use leptos_node_ref::AnyNodeRef;
use std::sync::atomic::{AtomicUsize, Ordering};

use super::popover::Popover;

/// Monotonic counter for unique `aria-describedby` ids — one per mounted
/// tooltip instance, so the trigger and its bubble are linked unambiguously.
static TOOLTIP_SEQ: AtomicUsize = AtomicUsize::new(0);

#[component]
pub fn Tooltip(
    /// Tooltip label. Accepts a `&str`/`String` or a reactive signal.
    #[prop(into)]
    text: Signal<String>,
    /// Placement relative to the trigger. Default `Top`.
    #[prop(default = Placement::Top)]
    placement: Placement,
    /// Distance in px between trigger and bubble. Default 6.
    #[prop(default = 6.0)]
    offset: f64,
    /// Override the bubble's Tailwind classes (sans positioning, which
    /// `Popover` owns). Default: dark pill with white text.
    #[prop(
        default = "rounded-md bg-gray-900 px-2 py-1 text-xs font-medium text-white shadow-lg".into(),
        into
    )]
    class: String,
    /// The trigger element(s) the tooltip describes.
    children: Children,
) -> impl IntoView {
    let open = RwSignal::new(false);
    let trigger_ref = AnyNodeRef::new();
    // `StoredValue` is `Copy`, so the bubble's `ChildrenFn` closure (which
    // `Popover` may re-run on each open/close) stays `Fn` rather than `FnOnce`.
    let id = StoredValue::new(format!(
        "tooltip-{}",
        TOOLTIP_SEQ.fetch_add(1, Ordering::Relaxed)
    ));

    view! {
        <span
            node_ref=trigger_ref
            class="inline-flex"
            aria-describedby=move || id.get_value()
            on:mouseenter=move |_| open.set(true)
            on:mouseleave=move |_| open.set(false)
            on:focusin=move |_| open.set(true)
            on:focusout=move |_| open.set(false)
        >
            {children()}
        </span>
        <Popover
            open=open
            set_open=open.write_only()
            trigger_ref=trigger_ref
            placement=placement
            offset=offset
            role="tooltip"
            class=class
        >
            <span id=move || id.get_value()>{move || text.get()}</span>
        </Popover>
    }
}
