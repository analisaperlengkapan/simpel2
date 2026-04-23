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

use leptos::prelude::*;

// ============================================================================
// ERROR PANEL — user-friendly error display with retry
// ============================================================================

/// Severity level for error display styling.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ErrorSeverity {
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
    #[prop(default = ErrorSeverity::Error)]
    severity: ErrorSeverity,
    /// Optional retry callback. When provided, a "Coba Lagi" button is shown.
    #[prop(optional)]
    on_retry: Option<Callback<()>>,
) -> impl IntoView {
    let (border, bg, icon_color, icon) = match severity {
        ErrorSeverity::Info => (
            "border-blue-500/30",
            "bg-blue-950/30",
            "text-blue-400",
            "fas fa-info-circle",
        ),
        ErrorSeverity::Warning => (
            "border-yellow-500/30",
            "bg-yellow-950/30",
            "text-yellow-400",
            "fas fa-exclamation-triangle",
        ),
        ErrorSeverity::Error => (
            "border-red-500/30",
            "bg-red-950/30",
            "text-red-400",
            "fas fa-exclamation-circle",
        ),
    };

    view! {
        <div class=format!(
            "flex flex-col items-center justify-center rounded-xl border {} {} p-8 text-center",
            border, bg
        )>
            <i class=format!("{} text-3xl {} mb-4", icon, icon_color)></i>
            <p class="text-lg font-semibold text-slate-100 mb-2">{title}</p>
            <p class="text-sm text-slate-400 max-w-md">{message}</p>
            {on_retry.map(|retry| view! {
                <button
                    type="button"
                    class="mt-4 inline-flex items-center gap-2 rounded-lg bg-slate-700 px-4 py-2 text-sm font-medium text-slate-100 hover:bg-slate-600 transition-colors"
                    on:click=move |_| retry.run(())
                >
                    <i class="fas fa-redo text-xs"></i>
                    "Coba Lagi"
                </button>
            })}
        </div>
    }
}

// ============================================================================
// LOADING SKELETON — consistent loading state
// ============================================================================

/// A page-level loading skeleton with spinner and message.
///
/// Use inside `ResourceView` or `Suspense` as the fallback.
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
                <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
                <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
            </svg>
            <p class="text-sm text-slate-400">{message}</p>
        </div>
    }
}
