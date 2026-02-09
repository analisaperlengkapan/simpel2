//! Form validation utilities
//!
//! Client-side validation helpers for forms

use regex::Regex;
use std::sync::OnceLock;

static EMAIL_REGEX: OnceLock<Regex> = OnceLock::new();
static PHONE_REGEX: OnceLock<Regex> = OnceLock::new();
static NIP_REGEX: OnceLock<Regex> = OnceLock::new();

fn email_regex() -> &'static Regex {
    EMAIL_REGEX.get_or_init(|| {
        Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap()
    })
}

fn phone_regex() -> &'static Regex {
    PHONE_REGEX.get_or_init(|| Regex::new(r"^(\+62|62|0)[0-9]{9,12}$").unwrap())
}

fn nip_regex() -> &'static Regex {
    NIP_REGEX.get_or_init(|| Regex::new(r"^\d{18}$").unwrap())
}

/// Validate email address
pub fn validate_email(email: &str) -> Result<(), String> {
    if email.is_empty() {
        return Err("Email tidak boleh kosong".to_string());
    }
    if !email_regex().is_match(email) {
        return Err("Format email tidak valid".to_string());
    }
    Ok(())
}

/// Validate phone number
pub fn validate_phone(phone: &str) -> Result<(), String> {
    if phone.is_empty() {
        return Err("Nomor telepon tidak boleh kosong".to_string());
    }
    if !phone_regex().is_match(phone) {
        return Err("Format nomor telepon tidak valid".to_string());
    }
    Ok(())
}

/// Validate NIP (Nomor Induk Pegawai)
pub fn validate_nip(nip: &str) -> Result<(), String> {
    if nip.is_empty() {
        return Err("NIP tidak boleh kosong".to_string());
    }
    if !nip_regex().is_match(nip) {
        return Err("NIP harus 18 digit".to_string());
    }
    Ok(())
}

/// Validate required field
pub fn validate_required(value: &str, field_name: &str) -> Result<(), String> {
    if value.trim().is_empty() {
        Err(format!("{} tidak boleh kosong", field_name))
    } else {
        Ok(())
    }
}

/// Validate minimum length
pub fn validate_min_length(value: &str, min: usize, field_name: &str) -> Result<(), String> {
    if value.len() < min {
        Err(format!(
            "{} harus minimal {} karakter",
            field_name, min
        ))
    } else {
        Ok(())
    }
}

/// Validate maximum length
pub fn validate_max_length(value: &str, max: usize, field_name: &str) -> Result<(), String> {
    if value.len() > max {
        Err(format!(
            "{} maksimal {} karakter",
            field_name, max
        ))
    } else {
        Ok(())
    }
}

/// Validate password strength
pub fn validate_password(password: &str) -> Result<(), String> {
    if password.len() < 8 {
        return Err("Password minimal 8 karakter".to_string());
    }
    if !password.chars().any(|c| c.is_uppercase()) {
        return Err("Password harus mengandung huruf besar".to_string());
    }
    if !password.chars().any(|c| c.is_lowercase()) {
        return Err("Password harus mengandung huruf kecil".to_string());
    }
    if !password.chars().any(|c| c.is_numeric()) {
        return Err("Password harus mengandung angka".to_string());
    }
    Ok(())
}
