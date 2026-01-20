#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(clippy::all)]

use leptos::prelude::*;
use leptos_meta::*;

pub mod app;
pub mod components;

pub use app::App;
pub use components::{IntelFooter, IntelHeader};

use app::IntelligenceDashboard;

#[component]
pub fn IntelApp() -> impl IntoView {
    provide_meta_context();

    view! {
        <Title text="Intel - SIMPelv2"/>
        <Meta name="description" content="Sistem Intelligence - Kejaksaan Agung RI"/>
        <Meta name="keywords" content="intelligence, kejaksaan, monitoring, surveillance, intel"/>

        <div class="min-h-screen bg-gray-50">
            <IntelHeader />

            <main class="container mx-auto px-4 py-8">
                <IntelligenceDashboard />
            </main>

            <IntelFooter />
        </div>
    }
}
