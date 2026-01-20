#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(clippy::all)]

use crate::pages::{dashboard::Dashboard, schedules::Schedules};
use leptos::prelude::*;
use leptos_router::{
    components::{Route, Router, Routes},
    path,
};

pub mod api;
pub mod components;
pub mod pages;
pub mod types;

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
