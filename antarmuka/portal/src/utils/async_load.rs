//! Async loading helpers for common list-page patterns.

use leptos::prelude::*;
use leptos::task::spawn_local;
use std::future::Future;

/// Run a single async list fetch and map result into `items/error/loading` signals.
pub fn load_vec_once<T, E, Fut>(
    fetch: impl FnOnce() -> Fut + 'static,
    set_items: WriteSignal<Vec<T>>,
    set_error: WriteSignal<Option<String>>,
    set_loading: WriteSignal<bool>,
    error_prefix: &'static str,
) where
    T: Send + Sync + 'static,
    E: std::fmt::Display + 'static,
    Fut: Future<Output = Result<Vec<T>, E>> + 'static,
{
    set_loading.set(true);
    set_error.set(None);

    spawn_local(async move {
        match fetch().await {
            Ok(items) => set_items.set(items),
            Err(err) => set_error.set(Some(format!("{}: {}", error_prefix, err))),
        }
        set_loading.set(false);
    });
}

/// Run a single async fetch for non-list payloads and map result into `value/error/loading` signals.
pub fn load_value_once<T, E, Fut>(
    fetch: impl FnOnce() -> Fut + 'static,
    set_value: WriteSignal<T>,
    set_error: WriteSignal<Option<String>>,
    set_loading: WriteSignal<bool>,
    error_prefix: &'static str,
) where
    T: Send + Sync + 'static,
    E: std::fmt::Display + 'static,
    Fut: Future<Output = Result<T, E>> + 'static,
{
    set_loading.set(true);
    set_error.set(None);

    spawn_local(async move {
        match fetch().await {
            Ok(value) => set_value.set(value),
            Err(err) => set_error.set(Some(format!("{}: {}", error_prefix, err))),
        }
        set_loading.set(false);
    });
}
