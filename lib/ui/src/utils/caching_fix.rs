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
