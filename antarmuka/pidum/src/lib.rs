use leptos::prelude::*;
use leptos_meta::*;

mod pages;
use pages::*;

mod types;
pub use types::*;

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    view! {
        <Stylesheet id="leptos" href="/pkg/pidum-microfrontend.css"/>
        <Meta name="description" content="PIDUM - Penyidikan Pidana Umum"/>
        <Title text="PIDUM - Penyidikan Pidana Umum"/>

        <div class="min-h-screen bg-gray-50">
            <main class="flex-1">
                <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
                    <PidumDashboard/>
                </div>
            </main>
        </div>
    }
}
