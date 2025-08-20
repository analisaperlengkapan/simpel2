use leptos::prelude::*;

mod app;
mod components;

use app::IntelligenceDashboard;
use components::{IntelFooter, IntelHeader};
use leptos_meta::*;

#[component]
fn IntelApp() -> impl IntoView {
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

fn main() {
    // console_error_panic_hook::set_once(); // Temporarily disabled
    leptos::mount::mount_to_body(|| view! { <IntelApp /> });
}
