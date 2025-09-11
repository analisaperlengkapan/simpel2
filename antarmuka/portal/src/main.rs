use leptos::prelude::*;
use leptos::wasm_bindgen::JsCast;

mod app;
mod components;

use app::App;

fn main() {
    // Set panic hook for better error reporting
    console_error_panic_hook::set_once();

    // Log that we're starting
    leptos::logging::log!("🚀 Starting SIMPelv2 Portal...");

    // Mount to the main div element
    let window = leptos::web_sys::window();
    if window.is_none() {
        leptos::logging::error!("❌ Window object not found!");
        return;
    }

    let document = window.unwrap().document();
    if document.is_none() {
        leptos::logging::error!("❌ Document object not found!");
        return;
    }

    let document = document.unwrap();
    let main_element = document.get_element_by_id("main");

    if main_element.is_none() {
        leptos::logging::error!("❌ Main element with id='main' not found!");
        return;
    }

    let main_element = main_element.unwrap();
    let html_element = main_element.dyn_into::<leptos::web_sys::HtmlElement>();

    if html_element.is_err() {
        leptos::logging::error!("❌ Failed to cast main element to HtmlElement!");
        return;
    }

    let html_element = html_element.unwrap();

    // Clear existing content before mounting
    leptos::logging::log!("🧹 Clearing existing content in main element...");
    html_element.set_inner_html("");

    leptos::logging::log!("✅ Mounting Leptos app to main element...");

    let _handle = leptos::mount::mount_to(html_element, || view! { <App /> });
    _handle.forget(); // Keep the app mounted permanently

    leptos::logging::log!("✅ SIMPelv2 Portal mounted successfully!");
}
