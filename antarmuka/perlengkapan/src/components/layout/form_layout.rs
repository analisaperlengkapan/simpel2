use leptos::prelude::*;
use lib_ui::components::icon::AppIcon;
use phosphor_leptos::WARNING_CIRCLE;

/// A labelled field row with helper/error text and an optional required
/// marker. Accepts children so callers can mount whatever input control
/// makes sense (input, select, textarea, custom widget).
///
/// Pass `for_id` and give the control the same `id` to associate the two. It is
/// opt-in rather than automatic because the label element cannot simply wrap
/// the children: several callers mount a GROUP of controls, each with its own
/// `<label>` (the spesifikasi checkboxes on the pengajuan form), and nested
/// labels are invalid HTML that associate with the wrong control.
///
/// Without it the label is decoration: clicking it does not focus the field,
/// assistive technology announces the input as unlabelled, and
/// `getByLabel(...)` finds nothing.
#[component]
pub fn FormField(
    #[prop(into)] label: String,
    /// `id` of the control this labels. Omit for control GROUPS, where no
    /// single element is the label's target.
    #[prop(optional, into)]
    for_id: Option<String>,
    #[prop(optional, into)] helper: Option<String>,
    #[prop(optional, into)] error: Option<String>,
    #[prop(default = false)] required: bool,
    #[prop(default = false)] full_width: bool,
    children: Children,
) -> impl IntoView {
    let col_class = if full_width { "md:col-span-2" } else { "" };

    view! {
        <div class=format!("flex flex-col gap-1.5 {}", col_class)>
            <label
                class="flex items-center gap-1 text-xs font-semibold uppercase tracking-wide text-slate-300"
                for=for_id
            >
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
