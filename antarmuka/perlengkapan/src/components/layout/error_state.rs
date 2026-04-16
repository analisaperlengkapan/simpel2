use leptos::prelude::*;

use crate::api::AppError;

/// Error block for failed fetches. Takes an `AppError` directly so every
/// page surfaces the user-friendly Bahasa Indonesia message without
/// re-implementing the mapping.
#[component]
pub fn ErrorState(
    #[prop(into)] error: AppError,
    #[prop(optional, into)] title: Option<String>,
    #[prop(optional)] on_retry: Option<Box<dyn Fn()>>,
) -> impl IntoView {
    let headline = title.unwrap_or_else(|| "Gagal memuat data".to_string());
    let message = error.user_message();

    use std::rc::Rc;
    let on_retry_rc = Rc::new(on_retry);
    let handler = {
        let on_retry = Rc::clone(&on_retry_rc);
        move |_| {
            if let Some(ref f) = *on_retry {
                f();
            }
        }
    };
    let has_retry = on_retry_rc.is_some();

    view! {
        <div class="flex flex-col items-start gap-3 rounded-2xl border border-danger-500/30 bg-danger-500/[0.05] p-5"
             role="alert">
            <div class="flex items-start gap-3">
                <span class="mt-0.5 flex h-8 w-8 shrink-0 items-center justify-center rounded-lg bg-danger-500/15 text-danger-400 ring-1 ring-danger-500/25">
                    <i class="fas fa-triangle-exclamation text-sm"></i>
                </span>
                <div>
                    <h3 class="text-sm font-semibold text-white">{headline}</h3>
                    <p class="mt-1 text-sm leading-relaxed text-slate-300">{message}</p>
                </div>
            </div>
            {has_retry.then(|| view! {
                <button
                    type="button"
                    on:click=handler
                    class="focus-ring inline-flex items-center gap-2 rounded-lg border border-danger-500/30 bg-danger-500/10 px-3 py-1.5 text-xs font-medium text-danger-100 transition hover:bg-danger-500/20"
                >
                    <i class="fas fa-rotate-right text-[0.7rem]"></i>
                    "Coba lagi"
                </button>
            })}
        </div>
    }
}
