use leptos::prelude::*;
use lib_ui::components::icon::{AppIcon, icon_from_fa_class};

#[component]
pub fn LoadingState(#[prop(optional)] message: &'static str) -> impl IntoView {
    let text = if message.is_empty() {
        "Memuat data..."
    } else {
        message
    };

    view! {
        <div class="text-center py-12">
            <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600 mx-auto mb-3"></div>
            <p class="text-gray-500">{text}</p>
        </div>
    }
}

#[component]
pub fn EmptyState(
    title: &'static str,
    #[prop(optional)] description: &'static str,
    #[prop(optional)] icon_class: &'static str,
) -> impl IntoView {
    let icon = if icon_class.is_empty() {
        "fas fa-folder-open"
    } else {
        icon_class
    };

    view! {
        <div class="text-center py-12 text-gray-500">
            <span class="mb-3 inline-flex text-gray-300">
                <AppIcon icon=icon_from_fa_class(icon) size=40 />
            </span>
            <p class="font-medium">{title}</p>
            <Show when=move || !description.is_empty()>
                <p class="text-sm mt-1">{description}</p>
            </Show>
        </div>
    }
}
