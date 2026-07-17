use leptos::prelude::*;
use lib_ui::components::icon::AppIcon;
use phosphor_leptos::WARNING_CIRCLE;

/// A labelled field row with helper/error text and an optional required
/// marker. Accepts children so callers can mount whatever input control
/// makes sense (input, select, textarea, custom widget).
#[component]
pub fn FormField(
    #[prop(into)] label: String,
    #[prop(optional, into)] helper: Option<String>,
    #[prop(optional, into)] error: Option<String>,
    #[prop(default = false)] required: bool,
    #[prop(default = false)] full_width: bool,
    children: Children,
) -> impl IntoView {
    let col_class = if full_width { "md:col-span-2" } else { "" };

    view! {
        <div class=format!("flex flex-col gap-1.5 {}", col_class)>
            <label class="flex items-center gap-1 text-xs font-semibold uppercase tracking-wide text-slate-300">
                <span>{label}</span>
                {required
                    .then(|| {
                        view! {
                            <span class="text-danger-400" aria-hidden="true">
                                "*"
                            </span>
                        }
                    })}
            </label>
            <div>{children()}</div>
            {helper
                .as_ref()
                .map(|h| {
                    let text = h.clone();
                    view! { <p class="text-xs text-slate-500">{text}</p> }
                })}
            {error
                .map(|e| {
                    view! {
                        <p class="text-xs font-medium text-danger-300">
                            <span class="mr-1 text-[0.65rem]">
                                <AppIcon icon=WARNING_CIRCLE />
                            </span>
                            {e}
                        </p>
                    }
                })}
        </div>
    }
}
