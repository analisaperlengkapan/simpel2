use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CachedItem<T> {
    pub value: T,
    pub expires_at: DateTime<Utc>,
}

impl<T> CachedItem<T> {
    pub fn new(value: T, ttl_seconds: u64) -> Self {
        Self {
            value,
            expires_at: Utc::now() + chrono::Duration::seconds(ttl_seconds as i64),
        }
    }

    pub fn is_expired(&self) -> bool {
        self.expires_at < Utc::now()
    }
}

pub struct CacheManager;

impl CacheManager {
    #[cfg(target_arch = "wasm32")]
    pub fn get<T: for<'de> Deserialize<'de>>(key: &str) -> Option<T> {
        use gloo_storage::{LocalStorage, Storage};

        match LocalStorage::get::<CachedItem<T>>(key) {
            Ok(item) => {
                if item.is_expired() {
                    LocalStorage::delete(key);
                    None
                } else {
                    Some(item.value)
                }
            }
            Err(_) => None,
        }
    }

    #[cfg(target_arch = "wasm32")]
    pub fn set<T: Serialize>(key: &str, value: T, ttl_seconds: u64) {
        use gloo_storage::{LocalStorage, Storage};

        let item = CachedItem::new(value, ttl_seconds);
        let _ = LocalStorage::set(key, item);
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn get<T: for<'de> Deserialize<'de>>(_key: &str) -> Option<T> {
        None
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn set<T: Serialize>(_key: &str, _value: T, _ttl_seconds: u64) {}

    #[cfg(target_arch = "wasm32")]
    pub fn delete(key: &str) {
        use gloo_storage::{LocalStorage, Storage};
        LocalStorage::delete(key);
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn delete(_key: &str) {}

    #[cfg(target_arch = "wasm32")]
    pub fn clear_all() {
        use gloo_storage::{LocalStorage, Storage};
        LocalStorage::clear();
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn clear_all() {}

    #[cfg(target_arch = "wasm32")]
    pub fn clear_expired() {
        use gloo_storage::{LocalStorage, Storage};

        if let Some(window) = web_sys::window() {
            if let Ok(Some(storage)) = window.local_storage() {
                let mut keys_to_remove = Vec::new();

                // Iterate through all keys
                for i in 0..storage.length().unwrap_or(0) {
                    if let Ok(Some(key)) = storage.key(i) {
                        if let Ok(Some(value)) = storage.get_item(&key) {
                            // Try to parse as CachedItem
                            if let Ok(cached_item) =
                                serde_json::from_str::<CachedItem<String>>(&value)
                            {
                                if cached_item.is_expired() {
                                    keys_to_remove.push(key);
                                }
                            }
                        }
                    }
                }

                // Remove expired keys
                for key in keys_to_remove {
                    LocalStorage::delete(&key);
                }
            }
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn clear_expired() {}
}
