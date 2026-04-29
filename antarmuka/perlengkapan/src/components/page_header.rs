use leptos::prelude::*;
use lib_ui::components::icon::AppIcon;
use phosphor_leptos::PLUS;

#[component]
pub fn PageHeader(
    title: &'static str,
    #[prop(optional)] subtitle: &'static str,
    #[prop(optional)] action_href: &'static str,
    #[prop(optional)] action_label: &'static str,
) -> impl IntoView {
    view! {
        <div class="flex flex-col lg:flex-row items-start lg:items-center justify-between gap-4 mb-6">
            <div>
                <h2 class="text-xl font-bold text-gray-800">{title}</h2>
                <Show when=move || !subtitle.is_empty()>
                    <p class="text-sm text-gray-500 mt-1">{subtitle}</p>
                </Show>
            </div>

            <Show when=move || !action_href.is_empty() && !action_label.is_empty()>
                <a
                    href=action_href
                    class="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors inline-flex items-center"
                >
                    <span class="mr-2"><AppIcon icon=PLUS /></span>
                    {action_label}
                </a>
            </Show>
        </div>
    }
}
