use perencanaan_microfrontend::*;
use leptos::prelude::*;

fn main() {
    console_error_panic_hook::set_once();
    mount_to_body(|| {
        view! {
            <p>"Perencanaan Microfrontend"</p>
        }
    });
}
