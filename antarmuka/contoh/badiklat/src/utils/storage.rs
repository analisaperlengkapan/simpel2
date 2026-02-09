//! Browser storage utilities
//!
//! Wrappers for LocalStorage and SessionStorage

use gloo_storage::{LocalStorage, SessionStorage, Storage};
use serde::{de::DeserializeOwned, Serialize};

/// Store value in LocalStorage
pub fn local_set<T: Serialize>(key: &str, value: &T) -> Result<(), String> {
    LocalStorage::set(key, value).map_err(|e| e.to_string())
}

/// Get value from LocalStorage
pub fn local_get<T: DeserializeOwned>(key: &str) -> Option<T> {
    LocalStorage::get(key).ok()
}

/// Remove value from LocalStorage
pub fn local_remove(key: &str) {
    let _ = LocalStorage::delete(key);
}

/// Clear all LocalStorage
pub fn local_clear() {
    LocalStorage::clear();
}

/// Store value in SessionStorage
pub fn session_set<T: Serialize>(key: &str, value: &T) -> Result<(), String> {
    SessionStorage::set(key, value).map_err(|e| e.to_string())
}

/// Get value from SessionStorage
pub fn session_get<T: DeserializeOwned>(key: &str) -> Option<T> {
    SessionStorage::get(key).ok()
}

/// Remove value from SessionStorage
pub fn session_remove(key: &str) {
    let _ = SessionStorage::delete(key);
}

/// Clear all SessionStorage
pub fn session_clear() {
    SessionStorage::clear();
}
