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
//! # Usage (anywhere in the component tree)
//!
//! ```rust,ignore
//! use lib_ui::hooks::use_toast::use_toast;
//!
//! let toast = use_toast();
//! toast.success("Data berhasil disimpan!");
//! toast.error("Gagal menyimpan data.");
//! ```

use leptos::prelude::*;

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

/// A single toast message.
#[derive(Clone, Debug)]
pub struct ToastItem {
    pub id: u64,
    pub message: String,
    pub variant: ToastVariant,
    pub duration_ms: u32,
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
    fn push(&self, message: impl Into<String>, variant: ToastVariant, duration_ms: u32) {
        let id = self.next_id.get_untracked();
        self.next_id.set(id + 1);

        let item = ToastItem {
            id,
            message: message.into(),
            variant,
            duration_ms,
        };
        self.toasts.update(|list| list.push(item));

        // Auto-dismiss
        let toasts = self.toasts;
        let dismiss_id = id;
        leptos::task::spawn_local(async move {
            gloo_timers::future::TimeoutFuture::new(duration_ms).await;
            toasts.update(|list| list.retain(|t| t.id != dismiss_id));
        });
    }

    /// Show a success toast (green, 4s).
    pub fn success(&self, msg: impl Into<String>) {
        self.push(msg, ToastVariant::Success, 4000);
    }

    /// Show an error toast (red, 6s).
    pub fn error(&self, msg: impl Into<String>) {
        self.push(msg, ToastVariant::Error, 6000);
    }

    /// Show a warning toast (yellow, 5s).
    pub fn warning(&self, msg: impl Into<String>) {
        self.push(msg, ToastVariant::Warning, 5000);
    }

    /// Show an info toast (blue, 4s).
    pub fn info(&self, msg: impl Into<String>) {
        self.push(msg, ToastVariant::Info, 4000);
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
        <div class="fixed top-4 right-4 z-[9999] flex flex-col gap-2 pointer-events-none">
            <For
                each=move || ctx.toasts.get()
                key=|item| item.id
                children=move |item| {
                    let id = item.id;
                    let (border, bg, icon_color, icon) = match item.variant {
                        ToastVariant::Success => (
                            "border-emerald-500/40",
                            "bg-emerald-950/90",
                            "text-emerald-400",
                            "fas fa-check-circle",
                        ),
                        ToastVariant::Error => (
                            "border-red-500/40",
                            "bg-red-950/90",
                            "text-red-400",
                            "fas fa-exclamation-circle",
                        ),
                        ToastVariant::Warning => (
                            "border-yellow-500/40",
                            "bg-yellow-950/90",
                            "text-yellow-400",
                            "fas fa-exclamation-triangle",
                        ),
                        ToastVariant::Info => (
                            "border-blue-500/40",
                            "bg-blue-950/90",
                            "text-blue-400",
                            "fas fa-info-circle",
                        ),
                    };

                    view! {
                        <div class=format!(
                            "pointer-events-auto flex items-start gap-3 rounded-xl border {} {} px-4 py-3 shadow-lg backdrop-blur-sm min-w-[280px] max-w-sm animate-slide-in-right",
                            border, bg
                        )>
                            <i class=format!("{} {} mt-0.5", icon, icon_color)></i>
                            <p class="flex-1 text-sm text-slate-100">{item.message.clone()}</p>
                            <button
                                type="button"
                                class="text-slate-500 hover:text-slate-300 transition-colors"
                                on:click=move |_| ctx.dismiss(id)
                            >
                                <i class="fas fa-times text-xs"></i>
                            </button>
                        </div>
                    }
                }
            />
        </div>
    }
}
