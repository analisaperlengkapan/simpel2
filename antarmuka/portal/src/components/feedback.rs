//! Shared feedback components for admin pages.

use leptos::prelude::*;

#[component]
pub fn ErrorBanner(#[prop(into)] message: String) -> impl IntoView {
    view! {
        <div class="mb-4 p-3 bg-red-50 border border-red-200 rounded-lg text-red-700" role="alert">
            <span class="font-medium">"Terjadi kesalahan: "</span>
            {message}
        </div>
    }
}

#[component]
pub fn SuccessBanner(#[prop(into)] message: String) -> impl IntoView {
    view! {
        <div class="mb-4 p-3 bg-green-50 border border-green-200 rounded-lg text-green-700" role="status" aria-live="polite">
            <span class="font-medium">"Berhasil: "</span>
            {message}
        </div>
    }
}

#[component]
pub fn LoadingPanel(#[prop(optional, into)] message: String) -> impl IntoView {
    let text = if message.is_empty() {
        "Memuat data...".to_string()
    } else {
        message
    };

    view! {
        <div class="p-8 text-center text-gray-500" role="status" aria-live="polite">
            <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-primary-600 mx-auto mb-3"></div>
            {text}
        </div>
    }
}

#[component]
pub fn EmptyPanel(#[prop(into)] title: String, #[prop(into)] message: String) -> impl IntoView {
    view! {
        <div class="text-center py-12 bg-white rounded-xl border">
            <p class="text-base font-medium text-gray-700">{title}</p>
            <p class="text-sm text-gray-500 mt-2">{message}</p>
        </div>
    }
}
