//! Input validation utilities untuk Indonesia

use crate::core::{constants::*, types::*};
use regex::Regex;

/// Validate Indonesian NIK (16 digits)
pub fn validate_nik(nik: &str) -> ValidationResult {
    let regex = Regex::new(REGEX_NIK).unwrap();

    if nik.is_empty() {
        return ValidationResult::invalid(vec![ValidationError::new(
            "nik",
            "NIK tidak boleh kosong",
        )]);
    }

    if !regex.is_match(nik) {
        return ValidationResult::invalid(vec![ValidationError::new(
            "nik",
            "NIK harus 16 digit angka",
        )]);
    }

    ValidationResult::valid()
}

/// Validate Indonesian NIP (18 digits)
pub fn validate_nip(nip: &str) -> ValidationResult {
    let regex = Regex::new(REGEX_NIP).unwrap();

    if nip.is_empty() {
        return ValidationResult::invalid(vec![ValidationError::new(
            "nip",
            "NIP tidak boleh kosong",
        )]);
    }

    if !regex.is_match(nip) {
        return ValidationResult::invalid(vec![ValidationError::new(
            "nip",
            "NIP harus 18 digit angka",
        )]);
    }

    ValidationResult::valid()
}

/// Validate email address
pub fn validate_email(email: &str) -> ValidationResult {
    let regex = Regex::new(REGEX_EMAIL).unwrap();

    if email.is_empty() {
        return ValidationResult::invalid(vec![ValidationError::new(
            "email",
            "Email tidak boleh kosong",
        )]);
    }

    if email.len() > EMAIL_MAX_LENGTH {
        return ValidationResult::invalid(vec![ValidationError::new(
            "email",
            format!("Email maksimal {} karakter", EMAIL_MAX_LENGTH),
        )]);
    }

    if !regex.is_match(email) {
        return ValidationResult::invalid(vec![ValidationError::new(
            "email",
            "Format email tidak valid",
        )]);
    }

    ValidationResult::valid()
}

/// Validate Indonesian phone number
pub fn validate_phone(phone: &str) -> ValidationResult {
    let regex = Regex::new(REGEX_PHONE).unwrap();

    if phone.is_empty() {
        return ValidationResult::invalid(vec![ValidationError::new(
            "phone",
            "Nomor telepon tidak boleh kosong",
        )]);
    }

    if !regex.is_match(phone) {
        return ValidationResult::invalid(vec![ValidationError::new(
            "phone",
            "Format nomor telepon tidak valid (contoh: 08xxxxxxxxxx, +62xxxxxxxxxx)",
        )]);
    }

    ValidationResult::valid()
}

/// Validate name (min/max length, no special chars)
pub fn validate_name(name: &str) -> ValidationResult {
    if name.is_empty() {
        return ValidationResult::invalid(vec![ValidationError::new(
            "name",
            "Nama tidak boleh kosong",
        )]);
    }

    if name.len() < NAME_MIN_LENGTH {
        return ValidationResult::invalid(vec![ValidationError::new(
            "name",
            format!("Nama minimal {} karakter", NAME_MIN_LENGTH),
        )]);
    }

    if name.len() > NAME_MAX_LENGTH {
        return ValidationResult::invalid(vec![ValidationError::new(
            "name",
            format!("Nama maksimal {} karakter", NAME_MAX_LENGTH),
        )]);
    }

    // Check for invalid characters (allow Indonesian letters)
    let regex = Regex::new(r"^[a-zA-Z\s.,'-]+$").unwrap();
    if !regex.is_match(name) {
        return ValidationResult::invalid(vec![ValidationError::new(
            "name",
            "Nama hanya boleh mengandung huruf, spasi, dan tanda baca dasar",
        )]);
    }

    ValidationResult::valid()
}

/// Validate required field (not empty)
pub fn validate_required(value: &str, field_name: &str) -> ValidationResult {
    if value.trim().is_empty() {
        return ValidationResult::invalid(vec![ValidationError::new(
            field_name,
            format!("{} tidak boleh kosong", field_name),
        )]);
    }

    ValidationResult::valid()
}

/// Validate string length
pub fn validate_length(value: &str, field_name: &str, min: usize, max: usize) -> ValidationResult {
    let len = value.len();

    if len < min {
        return ValidationResult::invalid(vec![ValidationError::new(
            field_name,
            format!("{} minimal {} karakter", field_name, min),
        )]);
    }

    if len > max {
        return ValidationResult::invalid(vec![ValidationError::new(
            field_name,
            format!("{} maksimal {} karakter", field_name, max),
        )]);
    }

    ValidationResult::valid()
}

/// Validate Indonesian postal code (5 digits)
pub fn validate_postal_code(code: &str) -> ValidationResult {
    let regex = Regex::new(REGEX_POSTAL_CODE).unwrap();

    if code.is_empty() {
        return ValidationResult::invalid(vec![ValidationError::new(
            "postal_code",
            "Kode pos tidak boleh kosong",
        )]);
    }

    if !regex.is_match(code) {
        return ValidationResult::invalid(vec![ValidationError::new(
            "postal_code",
            "Kode pos harus 5 digit angka",
        )]);
    }

    ValidationResult::valid()
}
