//! Authentication utilities
//!
//! This module provides helper functions for authentication

use gloo_storage::{LocalStorage, Storage};

/// Get authentication token from local storage
pub fn get_token() -> Option<String> {
    LocalStorage::get("auth_token").ok()
}

/// Set authentication token in local storage
pub fn set_token(token: &str) -> Result<(), String> {
    LocalStorage::set("auth_token", token).map_err(|e| e.to_string())
}

/// Remove authentication token from local storage
pub fn clear_token() {
    let _ = LocalStorage::delete("auth_token");
}

/// Check if user is authenticated
pub fn is_authenticated() -> bool {
    get_token().is_some()
}

/// Get user ID from local storage
pub fn get_user_id() -> Option<String> {
    LocalStorage::get("user_id").ok()
}

/// Set user ID in local storage
pub fn set_user_id(user_id: &str) -> Result<(), String> {
    LocalStorage::set("user_id", user_id).map_err(|e| e.to_string())
}

/// Clear all authentication data
pub fn clear_all() {
    clear_token();
    let _ = LocalStorage::delete("user_id");
    let _ = LocalStorage::delete("refresh_token");
}
