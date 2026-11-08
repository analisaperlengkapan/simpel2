use leptos::html;
use leptos::prelude::*;
use lib_ui::components::icon::AppIcon;
use lib_ui::utils::helpers::is_in_viewport;
use phosphor_leptos::WARNING_CIRCLE;

/// Banner for an ACTION that failed while the page's own data is fine: a refused
/// delete, an export that did not come back.
///
/// `ErrorState` replaces a page that could not load; this sits above content that
/// did. Both delete handlers and the export download used to end their error arm
/// in a console log, so an operator pressed a button, confirmed, and watched
/// nothing happen — the server had answered with the reason (a running campaign
/// is not deleted) and the screen never said it.
///
/// The banner sits above the content, and the button that caused it can be
/// screens below: on a long list the reason was on the page but out of sight, so
/// the press still looked like it did nothing. When the message appears and the
/// banner is not entirely on screen, the page scrolls to it.
#[component]
pub fn ActionError(#[prop(into)] message: Signal<Option<String>>) -> impl IntoView {
    let banner = NodeRef::<html::Div>::new();
    Effect::new(move |_| {
        if message.get().is_none() {
            return;
        }
        if let Some(el) = banner.get()
            && !is_in_viewport(&el)
        {
            el.scroll_into_view();
        }
    });
    view! {
        <Show when=move || message.get().is_some()>
            <div
                node_ref=banner
                // Leave room for the fixed top bar when the page scrolls to it.
                style="scroll-margin-top: 5rem"
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
