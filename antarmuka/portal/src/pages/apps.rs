//! Applications page — microfrontend launcher with submenu support

use crate::components::layout::MainLayout;
use crate::features::auth::UserSession;
use crate::features::microfrontends::{AppCategory, MicrofrontendRegistry};
use leptos::prelude::*;

/// Applications page — reads session from context
#[component]
pub fn AppsPage() -> impl IntoView {
    let (search_query, set_search_query) = signal(String::new());
    let (selected_category, set_selected_category) = signal(None::<AppCategory>);
    let (expanded_app, set_expanded_app) = signal(Option::<String>::None);

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

                // Apps Grid with Submenu Support
                <div class="space-y-4">
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
                                let has_submenu = app.submenu.is_some();
                                let app_id = app.id.clone();
                                let app_id_expanded = app_id.clone();
                                let is_expanded = move || expanded_app.get().as_ref() == Some(&app_id_expanded);

                                if has_submenu {
                                    // App with submenu
                                    view! {
                                        <div class="rounded-xl border border-slate-100 dark:border-navy-700 overflow-hidden bg-white dark:bg-navy-800">
                                            <button
                                                on:click=move |_| {
                                                    if is_expanded() {
                                                        set_expanded_app.set(None);
                                                    } else {
                                                        set_expanded_app.set(Some(app_id.clone()));
                                                    }
                                                }
                                                class="w-full text-left p-5 hover:bg-slate-50 dark:hover:bg-navy-700 transition-colors border-b border-slate-100 dark:border-navy-700"
                                            >
                                                <div class="flex items-start justify-between">
                                                    <div class="flex items-start gap-4 flex-1">
                                                        <div class=format!("w-12 h-12 rounded-xl bg-gradient-to-br {} flex items-center justify-center flex-shrink-0 transition-transform", color_classes)>
                                                            <span class="text-xl text-white">{app.icon.clone()}</span>
                                                        </div>
                                                        <div class="flex-1 min-w-0">
                                                            <div class="flex items-center gap-2">
                                                                <h3 class="text-sm font-semibold text-slate-800 dark:text-white">{app.name.clone()}</h3>
                                                                {is_beta.then(|| view! {
                                                                    <span class="px-1.5 py-0.5 text-[10px] font-medium bg-amber-100 dark:bg-amber-900/30 text-amber-700 dark:text-amber-400 rounded">
                                                                        {status_text}
                                                                    </span>\n                                                                })}\n                                                            </div>\n                                                            <p class="text-xs text-slate-500 dark:text-slate-400 mt-0.5">{app.description.clone()}</p>\n                                                            <p class="text-xs text-slate-400 dark:text-slate-500 mt-2">{app.category.display_name()}</p>\n                                                        </div>\n                                                    </div>\n                                                    <svg \n                                                        class=move || format!(\n                                                            "w-5 h-5 text-slate-400 flex-shrink-0 transition-transform duration-300 {}\",\n                                                            if is_expanded() { \"rotate-180\" } else { \"\" }\n                                                        )\n                                                        fill=\"none\" \n                                                        stroke=\"currentColor\" \n                                                        viewBox=\"0 0 24 24\"\n                                                    >\n                                                        <path stroke-linecap=\"round\" stroke-linejoin=\"round\" stroke-width=\"2\" d=\"M19 14l-7 7m0 0l-7-7m7 7V3\"/>\n                                                    </svg>\n                                                </div>\n                                            </button>\n                                            \n                                            // Submenu with smooth animation\n                                            <div \n                                                class=move || format!(\n                                                    \"transition-all duration-300 overflow-hidden {}\",\n                                                    if is_expanded() { \"max-h-96\" } else { \"max-h-0\" }\n                                                )\n                                            >\n                                                <div class=\"p-4 space-y-2\">\n                                                    {app.submenu.as_ref().map(|submenu| {\n                                                        submenu.iter().map(|sub| {\n                                                            let sub_url = sub.url.clone();\n                                                            view! {\n                                                                <a\n                                                                    href=sub_url\n                                                                    class=\"block p-4 bg-slate-50 dark:bg-navy-900 rounded-lg border border-slate-200 dark:border-navy-700 hover:bg-slate-100 dark:hover:bg-navy-800 hover:border-navy-300 dark:hover:border-gold-500 transition-all group/sub\"\n                                                                >\n                                                                    <h4 class=\"text-sm font-medium text-slate-700 dark:text-slate-300 group-hover/sub:text-navy-600 dark:group-hover/sub:text-gold-400 transition-colors\">\n                                                                        {sub.name.clone()}\n                                                                    </h4>\n                                                                    <p class=\"text-xs text-slate-500 dark:text-slate-400 mt-1\">\n                                                                        {sub.description.clone()}\n                                                                    </p>\n                                                                </a>\n                                                            }\n                                                        }).collect_view()\n                                                    })}\n                                                </div>\n                                            </div>\n                                        </div>\n                                    }.into_any()
                                } else {
                                    // App without submenu - regular card\n                                    view! {\n                                        <a\n                                            href=app.url.clone()\n                                            target=\"_blank\"\n                                            class=\"block bg-white dark:bg-navy-800 rounded-xl p-5 shadow-sm border border-slate-100 dark:border-navy-700 hover:shadow-lg hover:border-navy-200 dark:hover:border-gold-600 transition-all group\"\n                                        >\n                                            <div class=\"flex items-start gap-4\">\n                                                <div class=format!(\"w-12 h-12 rounded-xl bg-gradient-to-br {} flex items-center justify-center flex-shrink-0 group-hover:scale-105 transition-transform\", color_classes)>\n                                                    <span class=\"text-xl text-white\">{app.icon.clone()}</span>\n                                                </div>\n                                                <div class=\"flex-1 min-w-0\">\n                                                    <div class=\"flex items-center gap-2\">\n                                                        <h3 class=\"text-sm font-semibold text-slate-800 dark:text-white\">{app.name.clone()}</h3>\n                                                        {is_beta.then(|| view! {\n                                                            <span class=\"px-1.5 py-0.5 text-[10px] font-medium bg-amber-100 dark:bg-amber-900/30 text-amber-700 dark:text-amber-400 rounded\">\n                                                                {status_text}\n                                                            </span>\n                                                        })}\n                                                    </div>\n                                                    <p class=\"text-xs text-slate-500 dark:text-slate-400 mt-0.5 line-clamp-2\">{app.description.clone()}</p>\n                                                    <p class=\"text-xs text-slate-400 dark:text-slate-500 mt-2\">{app.category.display_name()}</p>\n                                                </div>\n                                                <svg class=\"w-4 h-4 text-slate-400 group-hover:text-navy-600 dark:group-hover:text-gold-400 transition-colors flex-shrink-0 mt-1\" fill=\"none\" stroke=\"currentColor\" viewBox=\"0 0 24 24\">\n                                                    <path stroke-linecap=\"round\" stroke-linejoin=\"round\" stroke-width=\"2\" d=\"M10 6H6a2 2 0 00-2 2v10a2 2 0 002 2h10a2 2 0 002-2v-4M14 4h6m0 0v6m0-6L10 14\"/>\n                                                </svg>\n                                            </div>\n                                        </a>\n                                    }.into_any()
                                }
                            }).collect_view().into_any()
                        }
                    }}
                </div>
            </div>
        </MainLayout>
    }
}
