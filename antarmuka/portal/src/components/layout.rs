//! # Layout Utilities
//!
//! Komponen utilitas layout untuk konsistensi design sistem Portal SIMPelv2

use leptos::prelude::*;

/// Container wrapper untuk halaman standar dengan responsive padding
#[component]
#[allow(dead_code)]
pub fn PageContainer(
    /// CSS classes tambahan
    #[prop(optional)]
    class: Option<&'static str>,
    /// Children components
    children: Children,
) -> impl IntoView {
    let _container_class = format!(
        "container mx-auto px-4 sm:px-6 lg:px-8 {}",
        class.unwrap_or("")
    );

    view! {
        <div class=_container_class>
            {children()}
        </div>
    }
}
