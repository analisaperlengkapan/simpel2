//! LocalStorage utilities untuk persistent state
//!
//! Simplified version - use directly with signals in components

use leptos::prelude::*;
use serde::{Deserialize, Serialize};
use web_sys::window;

/// Load value from localStorage
pub fn load_from_storage<T>(key: &str) -> Option<T>
where
    T: for<'de> Deserialize<'de>,
{
    window()
        .and_then(|w| w.local_storage().ok().flatten())
        .and_then(|storage| storage.get_item(key).ok().flatten())
        .and_then(|json| serde_json::from_str::<T>(&json).ok())
}

/// Save value to localStorage
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

/// Remove item from localStorage
pub fn remove_from_storage(key: &str) {
    if let Some(window) = window()
        && let Ok(Some(storage)) = window.local_storage()
    {
        let _ = storage.remove_item(key);
    }
}

/// Clear all localStorage
pub fn clear_storage() {
    if let Some(window) = window()
        && let Ok(Some(storage)) = window.local_storage()
    {
        let _ = storage.clear();
    }
}

/// Hook for using localStorage with reactive signals
pub fn use_storage<T>(key: &str, default: T) -> (ReadSignal<T>, WriteSignal<T>)
where
    T: Serialize + for<'de> Deserialize<'de> + Clone + Send + Sync + 'static,
{
    // Load initial value from storage or use default
    let initial = load_from_storage::<T>(key).unwrap_or_else(|| default.clone());

    // Create signal
    let (value, set_value) = signal(initial);

    // Save to storage whenever value changes
    let key_owned = key.to_string();
    Effect::new(move || {
        let current = value.get();
        let _ = save_to_storage(&key_owned, &current);
    });

    (value, set_value)
}
