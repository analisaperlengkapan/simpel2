//! `<Popover>` — controlled floating panel anchored to a trigger element.
//!
//! Wraps `floating-ui-leptos` with project-default middleware
//! (offset 8px → flip → shift) and wires outside-click + Escape-key
//! dismiss via `leptos-use`. The trigger element stays in the
//! caller's hands so domain-specific styling and event wiring can
//! live in the consumer file; this primitive only manages the
//! floating panel.
//!
//! ```ignore
//! use lib_ui::components::floating::Popover;
//! use leptos::{html, prelude::*};
//! use leptos_node_ref::AnyNodeRef;
//!
//! #[component]
//! fn ProfileMenu() -> impl IntoView {
//!     let open = RwSignal::new(false);
//!     let trigger_ref = AnyNodeRef::new();
//!
//!     view! {
//!         <button
//!             node_ref=trigger_ref
//!             on:click=move |_| open.update(|o| *o = !*o)
//!         >"Open"</button>
//!         <Popover open=open set_open=open.write_only() trigger_ref=trigger_ref>
//!             <div class="p-3">"Panel content"</div>
//!         </Popover>
//!     }
//! }
//! ```

use floating_ui_leptos::{
    Flip, FlipOptions, Middleware, Offset, OffsetOptions, Placement, Shift, ShiftOptions,
    UseFloatingOptions, UseFloatingReturn, use_floating,
};
use leptos::{html, prelude::*};
use leptos_node_ref::AnyNodeRef;
use leptos_use::{on_click_outside, use_event_listener};
use send_wrapper::SendWrapper;
use web_sys::{Element, Window};

type FloatingMiddleware = Box<dyn Middleware<Element, Window>>;

#[component]
pub fn Popover(
    /// Open state (controlled).
    #[prop(into)]
    open: Signal<bool>,
    /// Setter called on outside-click and Escape-key dismiss.
    set_open: WriteSignal<bool>,
    /// `AnyNodeRef` attached to the trigger element by the caller.
    trigger_ref: AnyNodeRef,
    /// Where to place the panel relative to the trigger. Default
    /// `BottomEnd` (right-aligned panel sitting under the trigger).
    #[prop(default = Placement::BottomEnd)]
    placement: Placement,
    /// Distance in pixels between trigger and panel. Default 8.
    #[prop(default = 8.0)]
    offset: f64,
    /// Optional Tailwind class string applied alongside the layer.
    #[prop(optional, into)]
    class: Option<String>,
    /// Z-index utility class. Default `z-popover` (60).
    #[prop(default = "z-popover".into(), into)]
    z_class: String,
    /// ARIA role. Default `dialog`; pass `"menu"` for a menu popover.
    #[prop(default = "dialog")]
    role: &'static str,
    /// Panel content. `ChildrenFn` so the closure can be re-rendered
    /// each time the panel toggles open/closed.
    children: ChildrenFn,
) -> impl IntoView {
    let floating_ref = NodeRef::<html::Div>::new();
    let floating_any = AnyNodeRef::from(floating_ref);

    let middleware: Vec<FloatingMiddleware> = vec![
        Box::new(Offset::new(OffsetOptions::Value(offset))),
        Box::new(Flip::new(FlipOptions::default())),
        Box::new(Shift::new(ShiftOptions::default())),
    ];
    let UseFloatingReturn { floating_styles, .. } = use_floating(
        trigger_ref,
        floating_any,
        UseFloatingOptions::default()
            .placement(placement)
            .middleware(SendWrapper::new(middleware)),
    );

    // Outside-click dismiss — leptos-use cleans up the listener on drop.
    let _click_outside = on_click_outside(floating_ref, move |_| {
        if open.get_untracked() {
            set_open.set(false);
        }
    });

    // Escape-key dismiss on the document root.
    let _esc = use_event_listener(
        document().body(),
        leptos::ev::keydown,
        move |ev: web_sys::KeyboardEvent| {
            if ev.key() == "Escape" && open.get_untracked() {
                set_open.set(false);
            }
        },
    );

    let extra = class.unwrap_or_default();
    let layer_class = move || format!("{} {}", z_class, extra);

    view! {
        <Show when=move || open.get()>
            <div
                node_ref=floating_ref
                role=role
                class=layer_class()
                style:position=move || floating_styles.get().style_position()
                style:top=move || floating_styles.get().style_top()
                style:left=move || floating_styles.get().style_left()
                style:transform=move || floating_styles.get().style_transform()
            >
                {children()}
            </div>
        </Show>
    }
}
