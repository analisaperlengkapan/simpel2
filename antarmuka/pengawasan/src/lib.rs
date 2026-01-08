use leptos::prelude::*;
use leptos_router::{components::{Router, Routes, Route}, path};
use crate::pages::{dashboard::Dashboard, schedules::Schedules};

pub mod api;
pub mod components;
pub mod pages;

#[component]
pub fn App() -> impl IntoView {
    view! {
        <Router>
            <Routes fallback=|| "Not Found">
                <Route path=path!("/pengawasan") view=Dashboard />
                <Route path=path!("/pengawasan/schedules") view=Schedules />
            </Routes>
        </Router>
    }
}
