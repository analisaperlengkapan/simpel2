//! Unified icon component backed by [`phosphor_leptos`].
//!
//! ```ignore
//! use lib_ui::prelude::*;
//! use phosphor_leptos::USER_CIRCLE;
//!
//! view! { <AppIcon icon=USER_CIRCLE size=20 /> }
//! ```
//!
//! Replaces `<i class="fas fa-*">` FontAwesome usage. New code must use
//! the typed phosphor constants; see the crate root of `phosphor_leptos`
//! for the full catalogue.

use leptos::prelude::*;
use phosphor_leptos::{Icon, IconData, IconWeight};

pub use phosphor_leptos::IconWeight as AppIconWeight;

#[component]
pub fn AppIcon(
    #[prop(into)] icon: IconData,
    #[prop(optional, default = 16u32)] size: u32,
    #[prop(optional, default = IconWeight::Regular)] weight: IconWeight,
    #[prop(optional, into)] color: Option<String>,
) -> impl IntoView {
    let size_str = format!("{}px", size);
    let color_str = color.unwrap_or_else(|| "currentColor".to_string());
    view! {
        <Icon icon=icon weight=weight size=size_str color=color_str />
    }
}
