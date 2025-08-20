//! # Pembinaan Keuangan - Financial Management Microfrontend
//!
//! Sistem manajemen keuangan untuk pengembangan dan pembinaan
//! finansial Kejaksaan Agung RI dengan fitur modern dan terintegrasi.

pub mod components;
pub mod pages;

use leptos::prelude::*;
use leptos_meta::*;

// Import local components
use components::{Footer, Header};
use pages::KeuanganDashboard;

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    view! {
        <Stylesheet id="leptos" href="/pkg/keuangan-microfrontend.css"/>
        <Meta name="description" content="PEMBINAAN KEUANGAN - Sistem Pembinaan Keuangan"/>
        <Title text="PEMBINAAN KEUANGAN - Sistem Pembinaan Keuangan"/>

        <div class="min-h-screen bg-gray-50">
            <Header />
            <main class="flex-1">
                <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
                    <KeuanganDashboard/>
                </div>
            </main>
            <Footer />
        </div>
    }
}
