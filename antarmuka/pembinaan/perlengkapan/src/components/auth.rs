use leptos::prelude::*;

// Auth utility functions
pub fn get_auth_token() -> Option<String> {
    if let Some(window) = web_sys::window() {
        if let Ok(storage) = window.local_storage() {
            if let Some(storage) = storage {
                return storage.get_item("auth_token").unwrap_or(None);
            }
        }
    }
    None
}

pub fn set_auth_token(token: &str) {
    if let Some(window) = web_sys::window() {
        if let Ok(storage) = window.local_storage() {
            if let Some(storage) = storage {
                let _ = storage.set_item("auth_token", token);
            }
        }
    }
}

pub fn is_authenticated() -> bool {
    get_auth_token().is_some()
}
