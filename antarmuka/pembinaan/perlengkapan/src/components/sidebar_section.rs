use leptos::prelude::*;

#[derive(Clone, Debug)]
pub struct MenuItem {
    #[allow(dead_code)]
    pub href: String,
    #[allow(dead_code)]
    pub label: String,
}

impl MenuItem {
    #[allow(dead_code)]
    pub fn new(href: &str, label: &str) -> Self {
        Self {
            href: href.to_string(),
            label: label.to_string(),
        }
    }
}

#[component]
#[allow(dead_code)]
pub fn SidebarSection(
    title: String,
    icon: String,
    is_expanded: RwSignal<bool>,
    items: Vec<MenuItem>,
) -> impl IntoView {
    let toggle_expand = move |_| {
        is_expanded.update(|v| *v = !*v);
    };

    let chevron_class = move || {
        if is_expanded.get() {
            "transform rotate-90"
        } else {
            ""
        }
    };

    let items = StoredValue::new(items);

    view! {
        <div class="space-y-1">
            <button
                on:click=toggle_expand
                class="w-full text-gray-300 hover:text-white hover:bg-blue-800 px-4 py-3 rounded-lg flex items-center justify-between transition-all duration-200"
            >
                <div class="flex items-center space-x-3">
                    <div class="w-5 h-5" inner_html=icon></div>
                    <span>{title}</span>
                </div>
                <svg class=move || format!("w-4 h-4 transition-transform duration-200 {}", chevron_class())
                     fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7"/>
                </svg>
            </button>

            <Show when=move || is_expanded.get()>
                <div class="ml-4 space-y-1">
                    {move || items.with_value(|items| {
                        items.iter().map(|item| {
                            let href = item.href.clone();
                            let label = item.label.clone();

                            view! {
                                <a
                                    href=href
                                    class="block w-full text-left text-gray-400 hover:text-white hover:bg-blue-700 px-3 py-2 rounded text-sm transition-all duration-200"
                                >
                                    {label}
                                </a>
                            }
                        }).collect::<Vec<_>>()
                    })}
                </div>
            </Show>
        </div>
    }
}
