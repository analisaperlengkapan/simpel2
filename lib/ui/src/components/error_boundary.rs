//! Error Boundary & Resource View components.
//!
//! Shared across all microfrontends to provide consistent error handling
//! and data-fetching patterns — similar to Next.js `error.tsx` + `loading.tsx`.
//!
//! ## ErrorPanel
//! Displays a user-friendly error message with retry capability.
//!
//! ## ResourceView
//! Declarative data fetching with built-in loading/error states (like SWR/React Query).
//!
//! ```rust,ignore
//! let data = Resource::new(move || id.get(), |id| async move { api::get_item(id).await });
//!
//! view! {
//!     <ResourceView resource=data>
//!         {move |data| view! { <ItemDetail item=data /> }}
//!     </ResourceView>
//! }
//! ```

use crate::components::icon::AppIcon;
use leptos::prelude::*;
use phosphor_leptos::{
    ARROW_CLOCKWISE, CARET_LEFT, CARET_RIGHT, INFO, IconData, WARNING, WARNING_CIRCLE,
};

// ============================================================================
// ERROR PANEL — user-friendly error display with retry
// ============================================================================

/// Severity level for error **panel display styling**.
///
/// Renamed from `ErrorSeverity` to avoid a name collision with the
/// telemetry-oriented `utils::error_tracking::ErrorSeverity` (5 variants)
/// when both are glob-reexported at the crate root.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ErrorPanelSeverity {
    /// Informational — blue styling, non-critical.
    Info,
    /// Warning — yellow styling, degraded but functional.
    Warning,
    /// Error — red styling, action required.
    #[default]
    Error,
}

/// A user-friendly error panel with optional retry button.
///
/// Use this instead of raw error text to give users a clear,
/// actionable message when something goes wrong.
///
/// ```rust,ignore
/// <ErrorPanel
///     title="Gagal Memuat Data"
///     message="Koneksi ke server terputus."
///     on_retry=Some(Callback::new(move |_| refetch()))
/// />
/// ```
#[component]
pub fn ErrorPanel(
    /// Short error title (e.g. "Gagal Memuat Data").
    #[prop(into)]
    title: String,
    /// Detailed error message for the user.
    #[prop(into)]
    message: String,
    /// Visual severity level.
    #[prop(default = ErrorPanelSeverity::Error)]
    severity: ErrorPanelSeverity,
    /// Optional retry callback. When provided, a "Coba Lagi" button is shown.
    #[prop(optional)]
    on_retry: Option<Callback<()>>,
) -> impl IntoView {
    let (border, bg, icon_color, icon): (&str, &str, &str, IconData) = match severity {
        ErrorPanelSeverity::Info => (
            "border-blue-500/30",
            "bg-blue-950/30",
            "text-blue-400",
            INFO,
        ),
        ErrorPanelSeverity::Warning => (
            "border-yellow-500/30",
            "bg-yellow-950/30",
            "text-yellow-400",
            WARNING,
        ),
        ErrorPanelSeverity::Error => (
            "border-red-500/30",
            "bg-red-950/30",
            "text-red-400",
            WARNING_CIRCLE,
        ),
    };

    view! {
        <div class=format!(
            "flex flex-col items-center justify-center rounded-xl border {} {} p-8 text-center",
            border,
            bg,
        )>
            <span class=format!("mb-4 inline-flex {}", icon_color)>
                <AppIcon icon=icon size=32 />
            </span>
            <p class="text-lg font-semibold text-slate-100 mb-2">{title}</p>
            <p class="text-sm text-slate-400 max-w-md">{message}</p>
            {on_retry
                .map(|retry| {
                    view! {
                        <button
                            type="button"
                            class="mt-4 inline-flex items-center gap-2 rounded-lg bg-slate-700 px-4 py-2 text-sm font-medium text-slate-100 hover:bg-slate-600 transition-colors"
                            on:click=move |_| retry.run(())
                        >
                            <span class="text-xs">
                                <AppIcon icon=ARROW_CLOCKWISE />
                            </span>
                            "Coba Lagi"
                        </button>
                    }
                })}
        </div>
    }
}

// ============================================================================
// LOADING SKELETON — consistent loading state
// ============================================================================

/// A page-level loading skeleton with spinner and message.
///
/// Use inside `Suspense` as the fallback or standalone.
#[component]
pub fn LoadingPanel(
    /// Loading message shown below the spinner.
    #[prop(default = "Memuat data...".to_string(), into)]
    message: String,
) -> impl IntoView {
    view! {
        <div class="flex flex-col items-center justify-center rounded-xl border border-slate-700/40 bg-slate-900/50 p-8">
            <svg
                class="animate-spin h-8 w-8 text-amber-400 mb-4"
                xmlns="http://www.w3.org/2000/svg"
                fill="none"
                viewBox="0 0 24 24"
            >
                <circle
                    class="opacity-25"
                    cx="12"
                    cy="12"
                    r="10"
                    stroke="currentColor"
                    stroke-width="4"
                ></circle>
                <path
                    class="opacity-75"
                    fill="currentColor"
                    d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
                ></path>
            </svg>
            <p class="text-sm text-slate-400">{message}</p>
        </div>
    }
}

// ============================================================================
// DARK PAGINATION — dark-theme pagination for list pages
// ============================================================================

/// Dark-themed pagination bar for list pages.
///
/// Replaces the ~30 lines of copy-pasted pagination HTML in each list component.
/// Equivalent to Laravel's `{{ $items->links() }}`.
///
/// ```rust,ignore
/// <DarkPagination
///     current_page=page
///     total_pages=10
///     total_items=200
///     items_shown=20
///     on_prev=Callback::new(move |_| set_page.update(|p| *p -= 1))
///     on_next=Callback::new(move |_| set_page.update(|p| *p += 1))
/// />
/// ```
#[component]
// The `#[allow(unused_variables)]` that used to sit here claimed clippy was
// wrong about `on_next` being unused. Clippy was right: a mangled `>=` had
// turned this button's `on:click` into a text node, so the callback really was
// never called — the suppression was hiding a dead "Selanjutnya" button, not a
// false positive. Do not re-add it; if `on_next` goes unused again, that is
// the same bug returning.
pub fn DarkPagination(
    /// Current page number (1-based).
    #[prop(into)]
    current_page: Signal<i64>,
    /// Total number of pages.
    total_pages: i64,
    /// Total number of items across all pages.
    total_items: i64,
    /// Number of items shown on the current page.
    items_shown: usize,
    /// Callback when "Previous" is clicked.
    on_prev: Callback<()>,
    /// Callback when "Next" is clicked.
    on_next: Callback<()>,
) -> impl IntoView {
    view! {
        <div class="flex items-center justify-between border-t border-white/[0.04] px-5 py-3">
            <p class="text-xs text-slate-400">
                "Menampilkan " <span class="font-medium text-slate-200">{items_shown}</span>
                " dari " <span class="font-medium text-slate-200">{total_items}</span> " data"
            </p>
            <div class="flex items-center gap-2">
                <button
                    type="button"
                    class="inline-flex items-center gap-1.5 rounded-lg border border-white/10 bg-white/[0.04] px-3 py-1.5 text-xs text-slate-300 transition hover:bg-white/[0.08] disabled:opacity-40"
                    prop:disabled=move || current_page.get() <= 1
                    on:click=move |_| on_prev.run(())
                >
                    <span class="text-[0.6rem]">
                        <AppIcon icon=CARET_LEFT />
                    </span>
                    "Sebelumnya"
                </button>
                <span class="text-xs text-slate-400">
                    {move || current_page.get()} " / " {total_pages}
                </span>
                <button
                    type="button"
                    class="inline-flex items-center gap-1.5 rounded-lg border border-white/10 bg-white/[0.04] px-3 py-1.5 text-xs text-slate-300 transition hover:bg-white/[0.08] disabled:opacity-40"
                    prop:disabled=move || { current_page.get() >= total_pages }
                    on:click=move |_| on_next.run(())
                >
                    "Selanjutnya"
                    <span class="text-[0.6rem]">
                        <AppIcon icon=CARET_RIGHT />
                    </span>
                </button>
            </div>
        </div>
    }
}

// ============================================================================
// RESOURCE VIEW — declarative data fetching (like React Query / SWR)
// ============================================================================

/// Declarative data-fetching component with built-in loading/error states.
///
/// Combines Leptos `Resource` + `Suspense` + `ErrorPanel` + `LoadingPanel`
/// into a single component — similar to React Query's `useQuery` or SWR.
///
/// # Usage
///
/// ```rust,ignore
/// use leptos::prelude::*;
/// use lib_ui::components::{ResourceView, LoadingPanel, ErrorPanel};
///
/// #[component]
/// fn ItemListPage() -> impl IntoView {
///     let items = Resource::new(
///         || (),
///         |_| async move { api::fetch_items().await },
///     );
///
///     view! {
///         <ResourceView
///             resource=items
///             loading_message="Memuat daftar barang..."
///             error_title="Gagal Memuat Data"
///         >
///             {move |data: Vec<Item>| view! { <ItemTable items=data /> }}
///         </ResourceView>
///     }
/// }
/// ```
#[component]
pub fn ResourceView<T, V>(
    /// The Leptos `Resource` to observe.
    resource: Resource<Result<T, String>>,
    /// Render function called with the successful data.
    children: Box<dyn Fn(T) -> V + Send + Sync>,
    /// Loading message shown while the resource is pending.
    #[prop(default = "Memuat data...".to_string(), into)]
    loading_message: String,
    /// Error title shown when the resource fails.
    #[prop(default = "Gagal Memuat Data".to_string(), into)]
    error_title: String,
) -> impl IntoView
where
    T: Clone + Send + Sync + 'static,
    V: IntoView + 'static,
{
    let loading_msg = loading_message.clone();
    let err_title = error_title.clone();

    move || {
        match resource.get() {
            None => {
                // Still loading
                view! { <LoadingPanel message=loading_msg.clone() /> }.into_any()
            }
            Some(Ok(data)) => {
                // Success — render children with data
                (children)(data).into_any()
            }
            Some(Err(err)) => {
                // Error — show error panel
                view! { <ErrorPanel title=err_title.clone() message=err /> }.into_any()
            }
        }
    }
}
