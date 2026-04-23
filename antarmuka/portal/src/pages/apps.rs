//! Applications page — microfrontend launcher

use crate::components::layout::MainLayout;
use crate::features::auth::UserSession;
use crate::features::microfrontends::{AppCategory, MicrofrontendRegistry};
use leptos::prelude::*;

/// Applications page — reads session from context
#[component]
pub fn AppsPage() -> impl IntoView {
    let (search_query, set_search_query) = signal(String::new());
    let (selected_category, set_selected_category) = signal(None::<AppCategory>);

    let all_apps = MicrofrontendRegistry::get_all_apps();
    let categories = MicrofrontendRegistry::get_all_categories();

    view! {
        <MainLayout>
            <div class="p-6 lg:p-8">
                // Header
                <div class="mb-8">
                    <h1 class="text-2xl font-bold text-slate-800 dark:text-white">"Aplikasi"</h1>
                    <p class="text-sm text-slate-500 dark:text-slate-400 mt-1">"Daftar layanan dan aplikasi terintegrasi SIMPEL"</p>
                </div>

                // Search
                <div class="mb-6">
                    <div class="relative max-w-md">
                        <svg class="absolute left-3.5 top-1/2 -translate-y-1/2 w-4 h-4 text-slate-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z"/>
                        </svg>
                        <input
                            type="text"
                            placeholder="Cari aplikasi..."
                            class="w-full pl-10 pr-4 py-2.5 bg-white dark:bg-navy-800 border border-slate-200 dark:border-navy-700 rounded-xl text-sm text-slate-700 dark:text-slate-200 placeholder-slate-400 focus:border-navy-500 dark:focus:border-gold-500 focus:outline-none transition-colors"
                            prop:value=move || search_query.get()
                            on:input=move |ev| set_search_query.set(event_target_value(&ev))
                        />
                    </div>
                </div>

                // Category Filter
                <div class="flex flex-wrap gap-2 mb-6">
                    <button
                        on:click=move |_| set_selected_category.set(None)
                        class=move || format!(
                            "px-4 py-1.5 text-sm font-medium rounded-lg transition-colors {}",
                            if selected_category.get().is_none() {
                                "bg-navy-800 text-white dark:bg-gold-500 dark:text-navy-900"
                            } else {
                                "bg-slate-100 dark:bg-navy-800 text-slate-600 dark:text-slate-300 hover:bg-slate-200 dark:hover:bg-navy-700"
                            }
                        )
                    >
                        "Semua"
                    </button>
                    {categories.into_iter().map(|category| {
                        let cat = category.clone();
                        let cat2 = category.clone();
                        let cat3 = category.clone();
                        view! {
                            <button
                                on:click=move |_| set_selected_category.set(Some(cat.clone()))
                                class=move || format!(
                                    "px-4 py-1.5 text-sm font-medium rounded-lg transition-colors {}",
                                    if selected_category.get().as_ref() == Some(&cat2) {
                                        "bg-navy-800 text-white dark:bg-gold-500 dark:text-navy-900"
                                    } else {
                                        "bg-slate-100 dark:bg-navy-800 text-slate-600 dark:text-slate-300 hover:bg-slate-200 dark:hover:bg-navy-700"
                                    }
                                )
                            >
                                {cat3.display_name()}
                            </button>
                        }
                    }).collect_view()}
                </div>

                // Apps Grid
                <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4">
                    {move || {
                        let query = search_query.get().to_lowercase();
                        let category = selected_category.get();

                        let filtered: Vec<_> = all_apps.iter()
                            .filter(|app| {
                                let matches_search = query.is_empty()
                                    || app.name.to_lowercase().contains(&query)
                                    || app.description.to_lowercase().contains(&query);
                                let matches_category = category.as_ref()
                                    .is_none_or(|cat| &app.category == cat);
                                matches_search && matches_category && app.status.is_available()
                            })
                            .collect();

                        if filtered.is_empty() {
                            view! {
                                <div class="col-span-full text-center py-16">
                                    <span class="text-4xl block mb-3">"🔍"</span>
                                    <p class="text-slate-500 dark:text-slate-400">"Tidak ada aplikasi yang ditemukan"</p>
                                </div>
                            }.into_any()
                        } else {
                            filtered.into_iter().map(|app| {
                                let color_classes = app.color.to_classes();
                                let status_text = app.status.badge_text();
                                let is_beta = app.status == crate::features::microfrontends::AppStatus::Beta;
                                view! {
                                    <a
                                        href=app.url.clone()
                                        target="_blank"
                                        class="block bg-white dark:bg-navy-800 rounded-xl p-5 shadow-sm border border-slate-100 dark:border-navy-700 hover:shadow-lg hover:border-navy-200 dark:hover:border-gold-600 transition-all group"
                                    >
                                        <div class="flex items-start gap-4">
                                            <div class=format!("w-12 h-12 rounded-xl bg-gradient-to-br {} flex items-center justify-center flex-shrink-0 group-hover:scale-105 transition-transform", color_classes)>
                                                <span class="text-xl text-white">{app.icon.clone()}</span>
                                            </div>
                                            <div class="flex-1 min-w-0">
                                                <div class="flex items-center gap-2">
                                                    <h3 class="text-sm font-semibold text-slate-800 dark:text-white">{app.name.clone()}</h3>
                                                    {is_beta.then(|| view! {
                                                        <span class="px-1.5 py-0.5 text-[10px] font-medium bg-amber-100 dark:bg-amber-900/30 text-amber-700 dark:text-amber-400 rounded">
                                                            {status_text}
                                                        </span>
                                                    })}
                                                </div>
                                                <p class="text-xs text-slate-500 dark:text-slate-400 mt-0.5 line-clamp-2">{app.description.clone()}</p>
                                                <p class="text-xs text-slate-400 dark:text-slate-500 mt-2">{app.category.display_name()}</p>
                                            </div>
                                            <svg class="w-4 h-4 text-slate-400 group-hover:text-navy-600 dark:group-hover:text-gold-400 transition-colors flex-shrink-0 mt-1" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 6H6a2 2 0 00-2 2v10a2 2 0 002 2h10a2 2 0 002-2v-4M14 4h6m0 0v6m0-6L10 14"/>
                                            </svg>
                                        </div>
                                    </a>
                                }
                            }).collect_view().into_any()
                        }
                    }}
                </div>
            </div>
        </MainLayout>
    }
}
