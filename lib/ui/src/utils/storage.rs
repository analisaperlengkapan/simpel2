//! Non-reactive `localStorage` helpers.
//!
//! For reactive storage bound to a signal, use `leptos_use::storage::use_local_storage`
//! (re-exported from `lib_ui::prelude`).

use serde::{Deserialize, Serialize};
use web_sys::window;

pub fn load_from_storage<T>(key: &str) -> Option<T>
where
    T: for<'de> Deserialize<'de>,
{
    window()
        .and_then(|w| w.local_storage().ok().flatten())
        .and_then(|storage| storage.get_item(key).ok().flatten())
        .and_then(|json| serde_json::from_str::<T>(&json).ok())
}

pub fn save_to_storage<T>(key: &str, value: &T) -> Result<(), String>
where
    T: Serialize,
{
    let window = window().ok_or("No window")?;
    let storage = window
        .local_storage()
        .map_err(|_| "No localStorage")?
        .ok_or("localStorage not available")?;
    let json = serde_json::to_string(value).map_err(|_| "Serialization failed")?;
    storage
        .set_item(key, &json)
        .map_err(|_| "Failed to set item")?;
    Ok(())
}

pub fn remove_from_storage(key: &str) {
    if let Some(window) = window()
        && let Ok(Some(storage)) = window.local_storage()
    {
        let _ = storage.remove_item(key);
    }
}

pub fn clear_storage() {
    if let Some(window) = window()
        && let Ok(Some(storage)) = window.local_storage()
    {
        let _ = storage.clear();
    }
}
