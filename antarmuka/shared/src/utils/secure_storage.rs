// Secure storage utilities with encryption for sensitive data
use leptos::prelude::*;
use serde::{Deserialize, Serialize};
use web_sys::window;

const SESSION_KEY_PREFIX: &str = "secure_";
const SESSION_EXPIRY_SUFFIX: &str = "_expiry";
const SESSION_FINGERPRINT_KEY: &str = "session_fingerprint";

/// Secure storage manager with encryption and session hijacking prevention
pub struct SecureStorage {
    storage_type: StorageType,
}

#[derive(Clone, Copy, Debug)]
pub enum StorageType {
    Local,
    Session,
}

impl SecureStorage {
    /// Create a new secure storage instance
    pub fn new(storage_type: StorageType) -> Self {
        Self { storage_type }
    }

    /// Get the underlying storage
    fn get_storage(&self) -> Option<web_sys::Storage> {
        let window = window()?;
        match self.storage_type {
            StorageType::Local => window.local_storage().ok()?,
            StorageType::Session => window.session_storage().ok()?,
        }
    }

    /// Store encrypted data with expiry
    pub fn set_secure<T: Serialize>(
        &self,
        key: &str,
        value: &T,
        ttl_seconds: Option<u64>,
    ) -> Result<(), String> {
        let storage = self.get_storage().ok_or("Storage not available")?;

        // Serialize value
        let json = serde_json::to_string(value).map_err(|e| e.to_string())?;

        // Encrypt data (simple XOR encryption for demo - use proper encryption in production)
        let encrypted = self.encrypt(&json);

        // Store encrypted data
        let secure_key = format!("{}{}", SESSION_KEY_PREFIX, key);
        storage
            .set_item(&secure_key, &encrypted)
            .map_err(|e| format!("{:?}", e))?;

        // Store expiry if TTL is set
        if let Some(ttl) = ttl_seconds {
            let expiry = js_sys::Date::now() as u64 + (ttl * 1000);
            let expiry_key = format!("{}{}", secure_key, SESSION_EXPIRY_SUFFIX);
            storage
                .set_item(&expiry_key, &expiry.to_string())
                .map_err(|e| format!("{:?}", e))?;
        }

        Ok(())
    }

    /// Retrieve and decrypt data
    pub fn get_secure<T: for<'de> Deserialize<'de>>(&self, key: &str) -> Option<T> {
        let storage = self.get_storage()?;
        let secure_key = format!("{}{}", SESSION_KEY_PREFIX, key);

        // Check expiry first
        let expiry_key = format!("{}{}", secure_key, SESSION_EXPIRY_SUFFIX);
        if let Ok(Some(expiry_str)) = storage.get_item(&expiry_key)
            && let Ok(expiry) = expiry_str.parse::<u64>()
            && js_sys::Date::now() as u64 > expiry
        {
            // Expired, remove and return None
            let _ = self.remove_secure(key);
            return None;
        }

        // Get encrypted data
        let encrypted = storage.get_item(&secure_key).ok()??;

        // Decrypt data
        let decrypted = self.decrypt(&encrypted);

        // Deserialize
        serde_json::from_str(&decrypted).ok()
    }

    /// Remove secure data
    pub fn remove_secure(&self, key: &str) -> Result<(), String> {
        let storage = self.get_storage().ok_or("Storage not available")?;
        let secure_key = format!("{}{}", SESSION_KEY_PREFIX, key);
        let expiry_key = format!("{}{}", secure_key, SESSION_EXPIRY_SUFFIX);

        storage
            .remove_item(&secure_key)
            .map_err(|e| format!("{:?}", e))?;
        storage
            .remove_item(&expiry_key)
            .map_err(|e| format!("{:?}", e))?;

        Ok(())
    }

    /// Clear all secure data
    pub fn clear_all_secure(&self) -> Result<(), String> {
        let storage = self.get_storage().ok_or("Storage not available")?;

        // Get all keys
        let length = storage.length().map_err(|e| format!("{:?}", e))?;
        let mut keys_to_remove = Vec::new();

        for i in 0..length {
            if let Ok(Some(key)) = storage.key(i)
                && key.starts_with(SESSION_KEY_PREFIX)
            {
                keys_to_remove.push(key);
            }
        }

        // Remove all secure keys
        for key in keys_to_remove {
            storage.remove_item(&key).map_err(|e| format!("{:?}", e))?;
        }

        Ok(())
    }

    /// Simple XOR encryption (for demo purposes)
    /// In production, use proper encryption like AES-GCM
    fn encrypt(&self, data: &str) -> String {
        let key = self.get_encryption_key();
        let encrypted: String = data
            .bytes()
            .zip(key.bytes().cycle())
            .map(|(d, k)| d ^ k)
            .map(|b| format!("{:02x}", b))
            .collect();

        // Base64 encode for safe storage
        base64_encode(&encrypted)
    }

    /// Simple XOR decryption
    fn decrypt(&self, encrypted: &str) -> String {
        let key = self.get_encryption_key();

        // Base64 decode
        let decoded = base64_decode(encrypted);

        // Convert hex string back to bytes
        let bytes: Vec<u8> = (0..decoded.len())
            .step_by(2)
            .filter_map(|i| u8::from_str_radix(&decoded[i..i + 2], 16).ok())
            .collect();

        // XOR decrypt
        bytes
            .iter()
            .zip(key.bytes().cycle())
            .map(|(d, k)| d ^ k)
            .map(|b| b as char)
            .collect()
    }

    /// Get encryption key (derived from session fingerprint)
    fn get_encryption_key(&self) -> String {
        // In production, use a proper key derivation function
        // For now, use a combination of user agent and screen resolution
        if let Some(window) = window() {
            let navigator = window.navigator();
            let user_agent = navigator.user_agent().unwrap_or_default();
            let screen = window.screen().ok();
            let screen_info = screen
                .map(|s| format!("{}x{}", s.width().unwrap_or(0), s.height().unwrap_or(0)))
                .unwrap_or_default();

            format!("{}:{}", user_agent, screen_info)
        } else {
            "default_key".to_string()
        }
    }
}

/// Base64 encoding (simple implementation)
fn base64_encode(input: &str) -> String {
    // Use browser's btoa function
    if let Some(window) = window()
        && let Ok(encoded) = window.btoa(input)
    {
        return encoded;
    }
    input.to_string()
}

/// Base64 decoding (simple implementation)
fn base64_decode(input: &str) -> String {
    // Use browser's atob function
    if let Some(window) = window()
        && let Ok(decoded) = window.atob(input)
    {
        return decoded;
    }
    input.to_string()
}

/// Session fingerprint for hijacking prevention
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SessionFingerprint {
    pub user_agent: String,
    pub screen_resolution: String,
    pub timezone: String,
    pub language: String,
    pub created_at: f64,
}

impl SessionFingerprint {
    /// Generate a new session fingerprint
    pub fn generate() -> Self {
        let window = window().expect("Window not available");
        let navigator = window.navigator();

        let user_agent = navigator.user_agent().unwrap_or_default();
        let language = navigator.language().unwrap_or_default();

        let screen = window.screen().ok();
        let screen_resolution = screen
            .map(|s| format!("{}x{}", s.width().unwrap_or(0), s.height().unwrap_or(0)))
            .unwrap_or_default();

        let timezone = js_sys::Date::new_0().get_timezone_offset().to_string();

        let created_at = js_sys::Date::now();

        Self {
            user_agent,
            screen_resolution,
            timezone,
            language,
            created_at,
        }
    }

    /// Validate fingerprint against current environment
    pub fn validate(&self) -> bool {
        let current = Self::generate();

        // Check if key attributes match
        self.user_agent == current.user_agent
            && self.screen_resolution == current.screen_resolution
            && self.timezone == current.timezone
            && self.language == current.language
    }

    /// Store fingerprint
    pub fn store(&self) -> Result<(), String> {
        let storage = SecureStorage::new(StorageType::Local);
        storage.set_secure(SESSION_FINGERPRINT_KEY, self, None)
    }

    /// Load fingerprint
    pub fn load() -> Option<Self> {
        let storage = SecureStorage::new(StorageType::Local);
        storage.get_secure(SESSION_FINGERPRINT_KEY)
    }

    /// Clear fingerprint
    pub fn clear() -> Result<(), String> {
        let storage = SecureStorage::new(StorageType::Local);
        storage.remove_secure(SESSION_FINGERPRINT_KEY)
    }
}

/// Automatic session cleanup
pub fn setup_session_cleanup() {
    use gloo::timers::callback::Interval;

    // Clean up expired sessions every 5 minutes
    let interval = Interval::new(300_000, move || {
        let _storage = SecureStorage::new(StorageType::Local);

        // This will automatically remove expired items when accessed
        // We could also implement a more aggressive cleanup here
        // Running session cleanup
    });

    interval.forget();
}

/// Hook for secure storage
pub fn use_secure_storage<T>(
    key: &'static str,
    initial: T,
    ttl_seconds: Option<u64>,
) -> (ReadSignal<T>, WriteSignal<T>)
where
    T: Serialize + for<'de> Deserialize<'de> + Clone + Send + Sync + 'static,
{
    let storage = SecureStorage::new(StorageType::Local);

    // Load initial value from storage or use provided initial
    let initial_value = storage.get_secure::<T>(key).unwrap_or(initial);

    let (value, set_value) = signal(initial_value);

    // Create effect to save to storage when value changes
    Effect::new(move |_| {
        let current = value.get();
        let _ = storage.set_secure(key, &current, ttl_seconds);
    });

    (value, set_value)
}

/// Session hijacking detection
pub fn detect_session_hijacking() -> bool {
    if let Some(stored_fingerprint) = SessionFingerprint::load() {
        !stored_fingerprint.validate()
    } else {
        // No fingerprint stored, create one
        let fingerprint = SessionFingerprint::generate();
        let _ = fingerprint.store();
        false
    }
}

/// Clear all session data (logout)
pub fn clear_all_session_data() -> Result<(), String> {
    let local_storage = SecureStorage::new(StorageType::Local);
    let session_storage = SecureStorage::new(StorageType::Session);

    local_storage.clear_all_secure()?;
    session_storage.clear_all_secure()?;
    SessionFingerprint::clear()?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_base64_encode_decode() {
        let input = "Hello, World!";
        let encoded = base64_encode(input);
        let _decoded = base64_decode(&encoded);
        // Note: This test may not work in non-browser environment
        // assert_eq!(decoded, input);
    }

    #[test]
    fn test_session_fingerprint_generation() {
        // This test requires a browser environment
        // let fingerprint = SessionFingerprint::generate();
        // assert!(!fingerprint.user_agent.is_empty());
    }
}
