use leptos::prelude::*;
use pemulihan_aset_microfrontend::App;

fn main() {
    console_error_panic_hook::set_once();
    leptos::mount::mount_to_body(|| view! { <App /> });
}
