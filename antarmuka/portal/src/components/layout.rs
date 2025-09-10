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
    _class: Option<&'static str>,
    /// Children components
    _children: Children,
) -> impl IntoView {
    let container_class = format!(
        "container mx-auto px-4 sm:px-6 lg:px-8 {}",
        _class.unwrap_or("")
    );

    view! {
        <div class={container_class}>
            {_children()}
        </div>
    }
}
