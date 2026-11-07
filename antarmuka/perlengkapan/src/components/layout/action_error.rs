use leptos::prelude::*;
use lib_ui::components::icon::AppIcon;
use phosphor_leptos::WARNING_CIRCLE;

/// Banner for an ACTION that failed while the page's own data is fine: a refused
/// delete, an export that did not come back.
///
/// `ErrorState` replaces a page that could not load; this sits above content that
/// did. Both delete handlers and the export download used to end their error arm
/// in a console log, so an operator pressed a button, confirmed, and watched
/// nothing happen — the server had answered with the reason (a running campaign
/// is not deleted) and the screen never said it.
#[component]
pub fn ActionError(#[prop(into)] message: Signal<Option<String>>) -> impl IntoView {
    view! {
        <Show when=move || message.get().is_some()>
            <div
                data-testid="action-error"
                role="alert"
                class="mt-4 flex items-start gap-2 rounded-xl border border-danger-500/30 bg-danger-500/[0.08] px-4 py-3 text-sm text-danger-300"
            >
                <span class="mt-0.5 shrink-0">
                    <AppIcon icon=WARNING_CIRCLE />
                </span>
                <span>{move || message.get()}</span>
            </div>
        </Show>
    }
}
