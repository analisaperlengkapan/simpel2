//! Global toast notification system for form feedback.
//!
//! Similar to `react-hot-toast` / `sonner` / Laravel `session()->flash()`.
//!
//! # Setup (once in App root)
//!
//! ```rust,ignore
//! use lib_ui::hooks::use_toast::ToastProvider;
//!
//! #[component]
//! fn App() -> impl IntoView {
//!     view! {
//!         <ToastProvider>
//!             <Router> /* routes */ </Router>
//!         </ToastProvider>
//!     }
//! }
//! ```
//!
//! # Usage
//!
//! ```rust,ignore
//! use lib_ui::hooks::use_toast::{use_toast, ToastOptions, ToastVariant};
//!
//! let toast = use_toast();
//!
//! // Convenience (back-compat with the original API)
//! toast.success("Data berhasil disimpan!");
//! toast.error("Gagal menyimpan data.");
//!
//! // Custom duration / progress bar
//! toast.show_with(
//!     ToastOptions::new(ToastVariant::Info, "Sinkronisasi berjalan…")
//!         .duration_ms(8000)
//!         .with_progress(),
//! );
//! ```
//!
//! # Accessibility
//!
//! The overlay region is `aria-live="polite"` and `aria-atomic="false"` so
//! assistive tech announces new toasts without interrupting. Each toast item
//! takes `role="alert"` for `Error`/`Warning` variants (interrupting) and
//! `role="status"` for `Success`/`Info` variants (non-interrupting).
//!
//! Toasts pause their auto-dismiss timer while hovered (`mouseenter`) and
//! resume on `mouseleave`, so users with motor or reading impairments can
//! actually finish reading the message before it disappears.

use crate::components::icon::AppIcon;
use leptos::prelude::*;
use phosphor_leptos::{CHECK_CIRCLE, INFO, WARNING, WARNING_CIRCLE, X};

// ============================================================================
// TYPES
// ============================================================================

/// Toast visual variant.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ToastVariant {
    Success,
    Error,
    Warning,
    Info,
}

impl ToastVariant {
    /// Default auto-dismiss duration per variant.
    pub fn default_duration_ms(self) -> u32 {
        match self {
            ToastVariant::Success | ToastVariant::Info => 4000,
            ToastVariant::Warning => 5000,
            ToastVariant::Error => 6000,
        }
    }

    /// ARIA role for the toast item.
    ///
    /// `alert` is interrupting (used for errors/warnings); `status` is
    /// polite (used for success/info confirmations).
    pub fn aria_role(self) -> &'static str {
        match self {
            ToastVariant::Error | ToastVariant::Warning => "alert",
            ToastVariant::Success | ToastVariant::Info => "status",
        }
    }
}

/// Configuration for a single toast — passed to [`ToastContext::show_with`].
///
/// Use the convenience methods on [`ToastContext`] (`success`, `error`,
/// `warning`, `info`) when defaults are fine; reach for `ToastOptions` only
/// when you need a custom duration, progress bar, or unusual variant pairing.
#[derive(Clone, Debug)]
pub struct ToastOptions {
    pub variant: ToastVariant,
    pub message: String,
    pub duration_ms: u32,
    pub show_progress: bool,
}

impl ToastOptions {
    pub fn new(variant: ToastVariant, message: impl Into<String>) -> Self {
        Self {
            variant,
            message: message.into(),
            duration_ms: variant.default_duration_ms(),
            show_progress: false,
        }
    }

    pub fn duration_ms(mut self, ms: u32) -> Self {
        self.duration_ms = ms;
        self
    }

    pub fn with_progress(mut self) -> Self {
        self.show_progress = true;
        self
    }
}

/// A single toast message in the live queue.
///
/// `paused` is shared between the overlay (which flips it on `mouseenter` /
/// `mouseleave`) and the auto-dismiss task (which only counts down while
/// `paused == false`). Cloning a `ToastItem` clones the signal *handle*, so
/// every observer sees the same state.
#[derive(Clone)]
pub struct ToastItem {
    pub id: u64,
    pub message: String,
    pub variant: ToastVariant,
    pub duration_ms: u32,
    pub show_progress: bool,
    pub paused: RwSignal<bool>,
}

// ============================================================================
// CONTEXT
// ============================================================================

/// Context provided by `ToastProvider`, consumed via `use_toast()`.
#[derive(Clone, Copy)]
pub struct ToastContext {
    toasts: RwSignal<Vec<ToastItem>>,
    next_id: RwSignal<u64>,
}

impl ToastContext {
    /// Push a fully-configured toast onto the queue. Returns the toast id so
    /// the caller can dismiss it programmatically before its timer expires.
    pub fn show_with(&self, options: ToastOptions) -> u64 {
        let id = self.next_id.get_untracked();
        self.next_id.set(id + 1);

        let paused = RwSignal::new(false);
        let duration_ms = options.duration_ms;

        let item = ToastItem {
            id,
            message: options.message,
            variant: options.variant,
            duration_ms,
            show_progress: options.show_progress,
            paused,
        };
        self.toasts.update(|list| list.push(item));

        // Auto-dismiss with pause-on-hover support: tick every 50ms and only
        // decrement remaining time while the toast is not paused. Cheap loop —
        // a 6-second toast wakes up ~120 times. The overhead is invisible
        // compared to the cost of the React-equivalent re-render storm.
        let toasts = self.toasts;
        leptos::task::spawn_local(async move {
            const TICK_MS: u32 = 50;
            let mut remaining = duration_ms as i32;
            while remaining > 0 {
                gloo_timers::future::TimeoutFuture::new(TICK_MS).await;
                if !paused.get_untracked() {
                    remaining -= TICK_MS as i32;
                }
                // If the user dismissed the toast manually, the item is gone
                // from the queue; bail out so we don't re-update the signal.
                if !toasts.with_untracked(|list| list.iter().any(|t| t.id == id)) {
                    return;
                }
            }
            toasts.update(|list| list.retain(|t| t.id != id));
        });

        id
    }

    /// Show a success toast (green, default 4s).
    pub fn success(&self, msg: impl Into<String>) {
        self.show_with(ToastOptions::new(ToastVariant::Success, msg));
    }

    /// Show an error toast (red, default 6s).
    pub fn error(&self, msg: impl Into<String>) {
        self.show_with(ToastOptions::new(ToastVariant::Error, msg));
    }

    /// Show a warning toast (yellow, default 5s).
    pub fn warning(&self, msg: impl Into<String>) {
        self.show_with(ToastOptions::new(ToastVariant::Warning, msg));
    }

    /// Show an info toast (blue, default 4s).
    pub fn info(&self, msg: impl Into<String>) {
        self.show_with(ToastOptions::new(ToastVariant::Info, msg));
    }

    /// Manually dismiss a toast by id.
    pub fn dismiss(&self, id: u64) {
        self.toasts.update(|list| list.retain(|t| t.id != id));
    }
}

/// Get the toast context. Panics if `<ToastProvider>` is not an ancestor.
pub fn use_toast() -> ToastContext {
    use_context::<ToastContext>().expect("use_toast() called outside <ToastProvider>")
}

// ============================================================================
// PROVIDER COMPONENT
// ============================================================================

/// Place at the root of your app to enable `use_toast()` everywhere.
#[component]
pub fn ToastProvider(children: Children) -> impl IntoView {
    let ctx = ToastContext {
        toasts: RwSignal::new(Vec::new()),
        next_id: RwSignal::new(1),
    };
    provide_context(ctx);

    view! {
        {children()}
        <ToastOverlay ctx=ctx />
    }
}

// ============================================================================
// OVERLAY — renders the floating toast stack
// ============================================================================

#[component]
fn ToastOverlay(ctx: ToastContext) -> impl IntoView {
    view! {
        <div
            class="fixed top-4 right-4 z-toast flex flex-col gap-2 pointer-events-none"
            aria-live="polite"
            aria-atomic="false"
        >
            <For
                each=move || ctx.toasts.get()
                key=|item| item.id
                children=move |item| {
                    let id = item.id;
                    let role = item.variant.aria_role();
                    let duration_ms = item.duration_ms;
                    let show_progress = item.show_progress;
                    let paused = item.paused;
                    let (border, bg, icon_color, progress_bg, icon_data) = match item.variant {
                        ToastVariant::Success => {
                            (
                                "border-emerald-500/40",
                                "bg-emerald-950/90",
                                "text-emerald-400",
                                "bg-emerald-400",
                                CHECK_CIRCLE,
                            )
                        }
                        ToastVariant::Error => {
                            (
                                "border-red-500/40",
                                "bg-red-950/90",
                                "text-red-400",
                                "bg-red-400",
                                WARNING_CIRCLE,
                            )
                        }
                        ToastVariant::Warning => {
                            (
                                "border-yellow-500/40",
                                "bg-yellow-950/90",
                                "text-yellow-400",
                                "bg-yellow-400",
                                WARNING,
                            )
                        }
                        ToastVariant::Info => {
                            (
                                "border-blue-500/40",
                                "bg-blue-950/90",
                                "text-blue-400",
                                "bg-blue-400",
                                INFO,
                            )
                        }
                    };

                    view! {
                        <div
                            role=role
                            class=format!(
                                "pointer-events-auto relative overflow-hidden flex items-start gap-3 rounded-xl border {} {} px-4 py-3 shadow-lg backdrop-blur-sm min-w-[280px] max-w-sm animate-slide-in-right",
                                border,
                                bg,
                            )
                            on:mouseenter=move |_| paused.set(true)
                            on:mouseleave=move |_| paused.set(false)
                            on:focusin=move |_| paused.set(true)
                            on:focusout=move |_| paused.set(false)
                        >
                            <span class=format!("{} mt-0.5", icon_color)>
                                <AppIcon icon=icon_data size=18 />
                            </span>
                            <p class="flex-1 text-sm text-slate-100">{item.message.clone()}</p>
                            <button
                                type="button"
                                class="text-slate-500 hover:text-slate-300 transition-colors"
                                aria-label="Tutup notifikasi"
                                on:click=move |_| ctx.dismiss(id)
                            >
                                <AppIcon icon=X size=12 />
                            </button>
                            {show_progress
                                .then(|| {
                                    view! {
                                        <span
                                            class=format!(
                                                "absolute bottom-0 left-0 h-0.5 {} origin-left animate-toast-progress",
                                                progress_bg,
                                            )
                                            style=move || {
                                                format!(
                                                    "animation-duration: {}ms; animation-play-state: {};",
                                                    duration_ms,
                                                    if paused.get() { "paused" } else { "running" },
                                                )
                                            }
                                        ></span>
                                    }
                                })}
                        </div>
                    }
                }
            />
        </div>
    }
}
