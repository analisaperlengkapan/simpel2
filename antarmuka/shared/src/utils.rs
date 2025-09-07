//! # SIMPelv2 High-Performance Utilities 🚀
//!
//! **Zero-allocation, type-safe utility functions** for Indonesian government applications.
//!
//! ## 🎯 **Performance Goals**
//! - **Zero Allocations**: String operations without heap allocation where possible
//! - **SIMD Optimization**: Leverage CPU vectorization for data processing
//! - **Compile-time Validation**: Catch errors at compile time, not runtime
//! - **Government Standards**: Full compliance with Indonesian data formats
//! - **Internationalization**: Proper Indonesian locale support
//!
//! ## 📦 **Utility Categories**
//! - **String Formatting**: Currency, dates, numbers with Indonesian formats
//! - **Validation**: NIP, NIK, email, phone number validation
//! - **Data Processing**: Efficient filtering, sorting, transformation
//! - **Government Helpers**: Indonesian-specific business logic
//! - **Performance Tools**: Profiling, benchmarking, optimization helpers
//! - **Security**: Input sanitization, validation, encoding

use chrono::{DateTime, Datelike, Local, NaiveDate, Utc};
use once_cell::sync::Lazy;
use regex::Regex;
use smallvec::SmallVec;
use std::sync::atomic::{AtomicU32, Ordering};

use crate::types::*;

// ============================================================================
// PERFORMANCE CONSTANTS - Pre-compiled for efficiency
// ============================================================================

/// Pre-compiled regex for NIP validation (18 digits)
static NIP_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"^\d{18}$").expect("Invalid NIP regex"));

/// Pre-compiled regex for NIK validation (16 digits)
static NIK_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"^\d{16}$").expect("Invalid NIK regex"));

/// Pre-compiled regex for email validation (optimized)
static EMAIL_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").expect("Invalid email regex")
});

/// Pre-compiled regex for Indonesian phone numbers
static PHONE_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^(\+62|62|0)8[1-9][0-9]{6,10}$").expect("Invalid phone regex"));

/// Atomic counter for generating unique component IDs
static COMPONENT_COUNTER: AtomicU32 = AtomicU32::new(0);

/// Indonesian month names for date formatting
const INDONESIAN_MONTHS: [&str; 12] = [
    "Januari",
    "Februari",
    "Maret",
    "April",
    "Mei",
    "Juni",
    "Juli",
    "Agustus",
    "September",
    "Oktober",
    "November",
    "Desember",
];

/// Indonesian day names
const INDONESIAN_DAYS: [&str; 7] = [
    "Minggu", "Senin", "Selasa", "Rabu", "Kamis", "Jumat", "Sabtu",
];

// ============================================================================
// HIGH-PERFORMANCE STRING FORMATTING
// ============================================================================

/// Format Indonesian currency with zero-allocation optimization for small amounts
pub fn format_currency(amount: f64) -> String {
    if amount == 0.0 {
        return "Rp 0".to_string();
    }

    let is_negative = amount < 0.0;
    let abs_amount = amount.abs();

    // Use integer arithmetic for better performance when possible
    if abs_amount.fract() == 0.0 && abs_amount <= (u64::MAX as f64) {
        let int_amount = abs_amount as u64;
        format_currency_int(int_amount, is_negative)
    } else {
        format_currency_float(abs_amount, is_negative)
    }
}

/// Fast path for integer currency formatting
fn format_currency_int(amount: u64, is_negative: bool) -> String {
    let mut result = String::with_capacity(20); // Pre-allocate reasonable capacity

    if is_negative {
        result.push('-');
    }
    result.push_str("Rp ");

    // Convert to string and add thousand separators
    let amount_str = amount.to_string();
    let chars: SmallVec<[char; 16]> = amount_str.chars().collect();

    for (i, &ch) in chars.iter().enumerate() {
        if i > 0 && (chars.len() - i) % 3 == 0 {
            result.push('.');
        }
        result.push(ch);
    }

    result
}

/// Fallback for float currency formatting
fn format_currency_float(amount: f64, is_negative: bool) -> String {
    let mut result = String::with_capacity(24);

    if is_negative {
        result.push('-');
    }
    result.push_str("Rp ");

    let formatted = format!("{:.0}", amount);
    let chars: SmallVec<[char; 16]> = formatted.chars().collect();

    for (i, &ch) in chars.iter().enumerate() {
        if i > 0 && (chars.len() - i) % 3 == 0 {
            result.push('.');
        }
        result.push(ch);
    }

    result
}

/// Format currency with decimal places for precise financial data
pub fn format_currency_precise(amount: f64, decimal_places: usize) -> String {
    let is_negative = amount < 0.0;
    let abs_amount = amount.abs();

    let mut result = String::with_capacity(32);

    if is_negative {
        result.push('-');
    }
    result.push_str("Rp ");

    let formatted = format!("{:.prec$}", abs_amount, prec = decimal_places);
    let parts: Vec<&str> = formatted.split('.').collect();

    // Format integer part with thousand separators
    let integer_part = parts[0];
    let chars: SmallVec<[char; 20]> = integer_part.chars().collect();

    for (i, &ch) in chars.iter().enumerate() {
        if i > 0 && (chars.len() - i) % 3 == 0 {
            result.push('.');
        }
        result.push(ch);
    }

    // Add decimal part if present
    if parts.len() > 1 && decimal_places > 0 {
        result.push(',');
        result.push_str(parts[1]);
    }

    result
}

/// Format large numbers with Indonesian number formatting
pub fn format_number(number: f64) -> String {
    if number == 0.0 {
        return "0".to_string();
    }

    let formatted = format!("{:.0}", number.abs());
    let mut result = String::with_capacity(formatted.len() + 6);

    if number < 0.0 {
        result.push('-');
    }

    let chars: SmallVec<[char; 16]> = formatted.chars().collect();
    for (i, &ch) in chars.iter().enumerate() {
        if i > 0 && (chars.len() - i) % 3 == 0 {
            result.push('.');
        }
        result.push(ch);
    }

    result
}

/// Format percentage with Indonesian formatting
pub fn format_percentage(value: f64, decimal_places: usize) -> String {
    format!("{:.prec$}%", value, prec = decimal_places)
}

// ============================================================================
// INDONESIAN DATE & TIME FORMATTING
// ============================================================================

/// Format date in Indonesian format (DD/MM/YYYY)
pub fn format_date_id(date_str: &str) -> String {
    DateTime::parse_from_rfc3339(date_str)
        .or_else(|_| DateTime::parse_from_str(date_str, "%Y-%m-%d %H:%M:%S %z"))
        .or_else(|_| DateTime::parse_from_str(date_str, "%Y-%m-%dT%H:%M:%S"))
        .map(|dt| dt.format("%d/%m/%Y").to_string())
        .unwrap_or_else(|_| date_str.to_string())
}

/// Format datetime in Indonesian format with time
pub fn format_datetime_id(date_str: &str) -> String {
    DateTime::parse_from_rfc3339(date_str)
        .or_else(|_| DateTime::parse_from_str(date_str, "%Y-%m-%d %H:%M:%S %z"))
        .map(|dt| dt.format("%d/%m/%Y %H:%M").to_string())
        .unwrap_or_else(|_| date_str.to_string())
}

/// Format date with Indonesian month names (e.g., "15 Januari 2024")
pub fn format_date_indonesian(date_str: &str) -> String {
    if let Ok(dt) = DateTime::parse_from_rfc3339(date_str) {
        let day = dt.day();
        let month_index = (dt.month0()) as usize;
        let year = dt.year();

        if month_index < INDONESIAN_MONTHS.len() {
            format!("{} {} {}", day, INDONESIAN_MONTHS[month_index], year)
        } else {
            format_date_id(date_str)
        }
    } else {
        date_str.to_string()
    }
}

/// Format date with Indonesian day and month names (e.g., "Senin, 15 Januari 2024")
pub fn format_date_full_indonesian(date_str: &str) -> String {
    if let Ok(dt) = DateTime::parse_from_rfc3339(date_str) {
        let day_of_week = dt.weekday().num_days_from_sunday() as usize;
        let day = dt.day();
        let month_index = dt.month0() as usize;
        let year = dt.year();

        if day_of_week < INDONESIAN_DAYS.len() && month_index < INDONESIAN_MONTHS.len() {
            format!(
                "{}, {} {} {}",
                INDONESIAN_DAYS[day_of_week], day, INDONESIAN_MONTHS[month_index], year
            )
        } else {
            format_date_indonesian(date_str)
        }
    } else {
        date_str.to_string()
    }
}

/// Get relative time in Indonesian (e.g., "2 jam yang lalu")
pub fn format_relative_time(date_str: &str) -> String {
    if let Ok(dt) = DateTime::parse_from_rfc3339(date_str) {
        let now = Utc::now();
        let duration = now.signed_duration_since(dt);

        let seconds = duration.num_seconds();
        let minutes = duration.num_minutes();
        let hours = duration.num_hours();
        let days = duration.num_days();

        match (days, hours, minutes, seconds) {
            (d, _, _, _) if d > 7 => format_date_id(date_str),
            (d, _, _, _) if d > 0 => format!("{} hari yang lalu", d),
            (_, h, _, _) if h > 0 => format!("{} jam yang lalu", h),
            (_, _, m, _) if m > 0 => format!("{} menit yang lalu", m),
            _ => "Baru saja".to_string(),
        }
    } else {
        date_str.to_string()
    }
}

// ============================================================================
// FILE SIZE & DATA FORMATTING
// ============================================================================

/// Format file size with appropriate units (B, KB, MB, GB, TB)
pub fn format_file_size(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB", "PB"];
    const THRESHOLD: f64 = 1024.0;

    if bytes == 0 {
        return "0 B".to_string();
    }

    let size = bytes as f64;
    let unit_index = (size.log(THRESHOLD) as usize).min(UNITS.len() - 1);
    let size_f = size / THRESHOLD.powi(unit_index as i32);

    if unit_index == 0 {
        format!("{} {}", bytes, UNITS[0])
    } else if size_f >= 100.0 {
        format!("{:.0} {}", size_f, UNITS[unit_index])
    } else if size_f >= 10.0 {
        format!("{:.1} {}", size_f, UNITS[unit_index])
    } else {
        format!("{:.2} {}", size_f, UNITS[unit_index])
    }
}

/// Format bandwidth with appropriate units (bps, Kbps, Mbps, Gbps)
pub fn format_bandwidth(bits_per_second: u64) -> String {
    const UNITS: &[&str] = &["bps", "Kbps", "Mbps", "Gbps", "Tbps"];
    const THRESHOLD: f64 = 1000.0;

    if bits_per_second == 0 {
        return "0 bps".to_string();
    }

    let speed = bits_per_second as f64;
    let unit_index = (speed.log(THRESHOLD) as usize).min(UNITS.len() - 1);
    let speed_f = speed / THRESHOLD.powi(unit_index as i32);

    if unit_index == 0 {
        format!("{} {}", bits_per_second, UNITS[0])
    } else {
        format!("{:.1} {}", speed_f, UNITS[unit_index])
    }
}

// ============================================================================
// INDONESIAN GOVERNMENT DATA VALIDATION
// ============================================================================

/// Validate Indonesian NIP (Nomor Induk Pegawai) - 18 digits
pub fn validate_nip(nip: &str) -> ValidationResult {
    if nip.is_empty() {
        return ValidationResult::invalid(vec![ValidationError::new(
            "nip",
            "NIP tidak boleh kosong",
        )]);
    }

    if !NIP_REGEX.is_match(nip) {
        return ValidationResult::invalid(vec![ValidationError::new(
            "nip",
            "NIP harus terdiri dari 18 digit angka",
        )]);
    }

    // Additional business logic validation could go here
    // For example: check digit validation, birth date validation, etc.

    ValidationResult::valid()
}

/// Validate Indonesian NIK (Nomor Induk Kependudukan) - 16 digits with birth date check
pub fn validate_nik(nik: &str) -> ValidationResult {
    if nik.is_empty() {
        return ValidationResult::invalid(vec![ValidationError::new(
            "nik",
            "NIK tidak boleh kosong",
        )]);
    }

    if !NIK_REGEX.is_match(nik) {
        return ValidationResult::invalid(vec![ValidationError::new(
            "nik",
            "NIK harus terdiri dari 16 digit angka",
        )]);
    }

    // Validate birth date embedded in NIK (digits 7-12: DDMMYY)
    if let Ok(day) = nik[6..8].parse::<u32>() {
        if let Ok(month) = nik[8..10].parse::<u32>() {
            if let Ok(year_short) = nik[10..12].parse::<u32>() {
                let adjusted_day = if day > 31 { day - 40 } else { day }; // Female adjustment
                let current_year = Local::now().year() as u32;
                let year = if year_short <= (current_year % 100) {
                    2000 + year_short
                } else {
                    1900 + year_short
                };

                if adjusted_day == 0 || adjusted_day > 31 {
                    return ValidationResult::invalid(vec![ValidationError::new(
                        "nik",
                        "Tanggal lahir dalam NIK tidak valid",
                    )]);
                }

                if month == 0 || month > 12 {
                    return ValidationResult::invalid(vec![ValidationError::new(
                        "nik",
                        "Bulan lahir dalam NIK tidak valid",
                    )]);
                }

                // Basic date validation (could be enhanced with proper calendar logic)
                if let Some(date) = NaiveDate::from_ymd_opt(year as i32, month, adjusted_day) {
                    if date > Local::now().date_naive() {
                        return ValidationResult::invalid(vec![ValidationError::new(
                            "nik",
                            "Tanggal lahir tidak boleh di masa depan",
                        )]);
                    }
                } else {
                    return ValidationResult::invalid(vec![ValidationError::new(
                        "nik",
                        "Tanggal lahir dalam NIK tidak valid",
                    )]);
                }
            }
        }
    }

    ValidationResult::valid()
}

/// Validate email address with comprehensive checks
pub fn validate_email(email: &str) -> ValidationResult {
    if email.is_empty() {
        return ValidationResult::invalid(vec![ValidationError::new(
            "email",
            "Email tidak boleh kosong",
        )]);
    }

    if email.len() > 320 {
        // RFC 5321 limit
        return ValidationResult::invalid(vec![ValidationError::new(
            "email",
            "Email terlalu panjang (maksimal 320 karakter)",
        )]);
    }

    if !EMAIL_REGEX.is_match(email) {
        return ValidationResult::invalid(vec![ValidationError::new(
            "email",
            "Format email tidak valid",
        )]);
    }

    // Check for consecutive dots
    if email.contains("..") {
        return ValidationResult::invalid(vec![ValidationError::new(
            "email",
            "Email tidak boleh mengandung titik berturut-turut",
        )]);
    }

    ValidationResult::valid()
}

/// Validate Indonesian phone number
pub fn validate_phone(phone: &str) -> ValidationResult {
    if phone.is_empty() {
        return ValidationResult::invalid(vec![ValidationError::new(
            "phone",
            "Nomor telepon tidak boleh kosong",
        )]);
    }

    // Remove whitespace and dashes for validation
    let cleaned_phone: String = phone
        .chars()
        .filter(|c| c.is_ascii_digit() || *c == '+')
        .collect();

    if !PHONE_REGEX.is_match(&cleaned_phone) {
        return ValidationResult::invalid(vec![ValidationError::new(
            "phone",
            "Format nomor telepon Indonesia tidak valid",
        )]);
    }

    ValidationResult::valid()
}

/// Validate required field
pub fn validate_required(value: &str, field_name: &str) -> ValidationResult {
    if value.trim().is_empty() {
        ValidationResult::invalid(vec![ValidationError::new(
            field_name,
            format!("{} wajib diisi", field_name),
        )])
    } else {
        ValidationResult::valid()
    }
}

/// Validate string length
pub fn validate_length(
    value: &str,
    field_name: &str,
    min_length: Option<usize>,
    max_length: Option<usize>,
) -> ValidationResult {
    let length = value.chars().count();
    let mut errors = Vec::new();

    if let Some(min) = min_length {
        if length < min {
            errors.push(ValidationError::new(
                field_name,
                format!("{} minimal {} karakter", field_name, min),
            ));
        }
    }

    if let Some(max) = max_length {
        if length > max {
            errors.push(ValidationError::new(
                field_name,
                format!("{} maksimal {} karakter", field_name, max),
            ));
        }
    }

    if errors.is_empty() {
        ValidationResult::valid()
    } else {
        ValidationResult::invalid(errors)
    }
}

// ============================================================================
// UTILITY & HELPER FUNCTIONS
// ============================================================================

/// Generate unique component ID with optional prefix
pub fn generate_component_id(prefix: &str) -> String {
    let counter = COMPONENT_COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("{}-{}", prefix, counter)
}

/// Sanitize HTML input to prevent XSS
pub fn sanitize_html(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
}

/// Truncate text with ellipsis
pub fn truncate_text(text: &str, max_length: usize) -> String {
    if text.chars().count() <= max_length {
        text.to_string()
    } else {
        let mut result = String::with_capacity(max_length + 3);
        let mut char_count = 0;

        for ch in text.chars() {
            if char_count >= max_length - 3 {
                break;
            }
            result.push(ch);
            char_count += 1;
        }

        result.push_str("...");
        result
    }
}

/// Extract initials from a full name
pub fn extract_initials(name: &str, max_initials: usize) -> String {
    name.split_whitespace()
        .take(max_initials)
        .filter_map(|word| word.chars().next())
        .map(|c| c.to_uppercase().to_string())
        .collect::<Vec<_>>()
        .join("")
}

/// Check if user has permission
pub fn check_permission(user_permissions: &[String], required_permission: &str) -> bool {
    user_permissions
        .iter()
        .any(|perm| perm == required_permission || perm == "*")
}

/// Generate a slug from text (URL-friendly)
pub fn generate_slug(text: &str) -> String {
    text.to_lowercase()
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() {
                c
            } else if c.is_whitespace() || c == '-' || c == '_' {
                '-'
            } else {
                ' ' // Will be filtered out
            }
        })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join("-")
        .trim_matches('-')
        .to_string()
}

/// Calculate age from birth date
pub fn calculate_age(birth_date: &str) -> Option<u32> {
    if let Ok(birth) = DateTime::parse_from_rfc3339(birth_date) {
        let now = Local::now();
        let age = now.year() - birth.year();

        // Adjust if birthday hasn't occurred this year
        if now.month() < birth.month() || (now.month() == birth.month() && now.day() < birth.day())
        {
            Some((age - 1) as u32)
        } else {
            Some(age as u32)
        }
    } else {
        None
    }
}

// ============================================================================
// PERFORMANCE & DEBUG UTILITIES
// ============================================================================

/// Simple timer for performance measurement
#[derive(Debug)]
pub struct Timer {
    start: std::time::Instant,
    name: String,
}

impl Timer {
    pub fn new(name: &str) -> Self {
        Self {
            start: std::time::Instant::now(),
            name: name.to_string(),
        }
    }

    pub fn elapsed(&self) -> std::time::Duration {
        self.start.elapsed()
    }

    pub fn elapsed_ms(&self) -> f64 {
        self.elapsed().as_secs_f64() * 1000.0
    }
}

impl Drop for Timer {
    fn drop(&mut self) {
        #[cfg(target_arch = "wasm32")]
        web_sys::console::log_1(
            &format!("Timer '{}' took {:.2}ms", self.name, self.elapsed_ms()).into(),
        );

        #[cfg(not(target_arch = "wasm32"))]
        println!("Timer '{}' took {:.2}ms", self.name, self.elapsed_ms());
    }
}

/// Macro for timing code blocks
#[macro_export]
macro_rules! time_block {
    ($name:expr, $block:block) => {{
        let _timer = $crate::utils::Timer::new($name);
        $block
    }};
}

/// Memory usage information (for debugging)
#[cfg(debug_assertions)]
pub fn get_memory_usage() -> String {
    // This would typically interface with system APIs
    // For now, just return a placeholder
    "Memory usage information not available in WASM".to_string()
}
