//! # Shared Utilities for SIMPelv2 - High Performance & Type-Safe
//!
//! Optimized utility functions untuk aplikasi Kejaksaan RI dengan focus pada:
//! - **Performance**: Zero-allocation string operations where possible
//! - **Type Safety**: Comprehensive error handling & validation
//! - **Government Standards**: Sesuai standar sistem pemerintahan Indonesia
//! - **Accessibility**: WCAG 2.1 AA compliance helpers

use chrono::{DateTime, Datelike, NaiveDate, Utc};
use std::collections::HashMap;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::types::*;

// ============================================================================
// PERFORMANCE-OPTIMIZED STRING FORMATTING
// ============================================================================

/// Format Indonesian currency dengan performance optimization
pub fn format_currency(amount: f64) -> String {
    // Pre-allocate capacity to avoid reallocations
    let base = format!("{amount:.0}");
    let mut result = String::with_capacity(base.len() + 6); // "Rp " + separators

    result.push_str("Rp ");
    let chars: Vec<char> = base.chars().collect();

    for (i, &ch) in chars.iter().enumerate() {
        if i > 0 && (chars.len() - i) % 3 == 0 {
            result.push('.');
        }
        result.push(ch);
    }

    result
}

/// Format currency with precision (for detailed financial displays)
pub fn format_currency_precise(amount: f64, precision: usize) -> String {
    let base = format!("{amount:.precision$}");
    let mut result = String::with_capacity(base.len() + 8);

    result.push_str("Rp ");
    let (integer_part, decimal_part) = if let Some(dot_pos) = base.find('.') {
        (&base[..dot_pos], Some(&base[dot_pos..]))
    } else {
        (base.as_str(), None)
    };

    // Format integer part with thousand separators
    let chars: Vec<char> = integer_part.chars().collect();
    for (i, &ch) in chars.iter().enumerate() {
        if i > 0 && (chars.len() - i) % 3 == 0 {
            result.push('.');
        }
        result.push(ch);
    }

    // Add decimal part if exists
    if let Some(decimal) = decimal_part {
        result.push_str(&decimal.replace('.', ","));
    }

    result
}

/// Format Indonesian date dengan fallback handling
pub fn format_date_id(date_str: &str) -> String {
    DateTime::parse_from_rfc3339(date_str)
        .map(|dt| dt.format("%d/%m/%Y").to_string())
        .unwrap_or_else(|_| date_str.to_string())
}

/// Format datetime dengan timezone support
pub fn format_datetime_id(date_str: &str) -> String {
    DateTime::parse_from_rfc3339(date_str)
        .map(|dt| dt.format("%d/%m/%Y %H:%M").to_string())
        .unwrap_or_else(|_| date_str.to_string())
}

/// Format file size dengan appropriate units
pub fn format_file_size(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    const THRESHOLD: f64 = 1024.0;

    if bytes == 0 {
        return "0 B".to_string();
    }

    let size = bytes as f64;
    let unit_index = (size.log(THRESHOLD) as usize).min(UNITS.len() - 1);
    let size_f = size / THRESHOLD.powi(unit_index as i32);

    if unit_index == 0 {
        format!("{} {}", bytes, UNITS[0])
    } else {
        format!("{:.1} {}", size_f, UNITS[unit_index])
    }
}

/// Optimized text truncation dengan word boundary respect
pub fn truncate_text(text: &str, max_length: usize) -> String {
    if text.len() <= max_length {
        return text.to_string();
    }

    if max_length < 3 {
        return "...".to_string();
    }

    let truncate_point = max_length - 3;

    // Try to break at word boundary
    if let Some(last_space) = text[..truncate_point].rfind(' ') {
        if last_space > truncate_point / 2 {
            // Don't break too early
            return format!("{}...", &text[..last_space]);
        }
    }

    format!("{}...", &text[..truncate_point])
}

/// Convert to title case dengan Indonesian language support
pub fn title_case(text: &str) -> String {
    let exceptions = &["dan", "atau", "di", "ke", "dari", "pada", "untuk", "yang"];

    text.split_whitespace()
        .enumerate()
        .map(|(i, word)| {
            let lower_word = word.to_lowercase();
            if i > 0 && exceptions.contains(&lower_word.as_str()) {
                lower_word
            } else {
                let mut chars = word.chars();
                match chars.next() {
                    None => String::new(),
                    Some(first) => {
                        first.to_uppercase().collect::<String>() + &chars.as_str().to_lowercase()
                    }
                }
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// Get initials dengan better Unicode support
pub fn get_initials(name: &str) -> String {
    name.split_whitespace()
        .filter_map(|word| word.chars().next())
        .take(2)
        .map(|c| c.to_uppercase().to_string())
        .collect::<String>()
}

// ============================================================================
// GOVERNMENT-SPECIFIC VALIDATION
// ============================================================================

/// Comprehensive NIP validation sesuai standar Kejaksaan RI
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct NipValidationResult {
    pub valid: bool,
    pub errors: Vec<String>,
    pub parsed_data: Option<NipData>,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct NipData {
    pub birth_year: u16,
    pub birth_month: u8,
    pub birth_day: u8,
    pub sequential_number: u32,
}

impl NipData {
    /// Get birth date if valid
    pub fn birth_date(&self) -> Option<NaiveDate> {
        NaiveDate::from_ymd_opt(
            self.birth_year as i32,
            self.birth_month as u32,
            self.birth_day as u32,
        )
    }

    /// Calculate age
    pub fn age(&self) -> Option<u32> {
        self.birth_date().map(|birth| {
            let today = Utc::now().date_naive();
            let age_years = today.year() - birth.year();
            if today.ordinal() < birth.ordinal() {
                (age_years - 1) as u32
            } else {
                age_years as u32
            }
        })
    }
}

/// Validate NIP dengan detailed parsing
pub fn validate_nip(nip: &str) -> NipValidationResult {
    let mut errors = Vec::new();

    // Basic format check
    if nip.len() != 18 {
        errors.push("NIP harus terdiri dari 18 digit".to_string());
        return NipValidationResult {
            valid: false,
            errors,
            parsed_data: None,
        };
    }

    if !nip.chars().all(|c| c.is_ascii_digit()) {
        errors.push("NIP hanya boleh berisi angka".to_string());
        return NipValidationResult {
            valid: false,
            errors,
            parsed_data: None,
        };
    }

    // Parse components
    let year_str = &nip[0..4];
    let month_str = &nip[4..6];
    let day_str = &nip[6..8];
    let sequential_str = &nip[8..18];

    let year = year_str.parse::<u16>().unwrap();
    let month = month_str.parse::<u8>().unwrap();
    let day = day_str.parse::<u8>().unwrap();
    let sequential = sequential_str.parse::<u32>().unwrap();

    // Validate year (reasonable range for government employees)
    if !(1950..=2010).contains(&year) {
        errors.push(format!(
            "Tahun lahir {year} tidak dalam rentang valid (1950-2010)"
        ));
    }

    // Validate month
    if !(1..=12).contains(&month) {
        errors.push(format!("Bulan {month} tidak valid"));
    }

    // Validate day
    if !(1..=31).contains(&day) {
        errors.push(format!("Tanggal {day} tidak valid"));
    }

    // Validate date exists
    if let Some(birth_date) = NaiveDate::from_ymd_opt(year as i32, month as u32, day as u32) {
        // Check if not future date
        if birth_date > Utc::now().date_naive() {
            errors.push("Tanggal lahir tidak boleh di masa depan".to_string());
        }

        // Check minimum age (16 years for government employees)
        let min_birth_date = Utc::now().date_naive() - chrono::Duration::days(16 * 365);
        if birth_date > min_birth_date {
            errors.push("Umur minimal untuk pegawai adalah 16 tahun".to_string());
        }
    } else {
        errors.push(format!("Tanggal {day}/{month}/{year} tidak valid"));
    }

    let parsed_data = if errors.is_empty() {
        Some(NipData {
            birth_year: year,
            birth_month: month,
            birth_day: day,
            sequential_number: sequential,
        })
    } else {
        None
    };

    NipValidationResult {
        valid: errors.is_empty(),
        errors,
        parsed_data,
    }
}

/// Validate Indonesian email addresses dengan domain checking
pub fn validate_email(email: &str) -> Result<(), Vec<String>> {
    let mut errors = Vec::new();

    if email.is_empty() {
        errors.push("Email tidak boleh kosong".to_string());
        return Err(errors);
    }

    // Basic format check
    if !email.contains('@') {
        errors.push("Format email tidak valid: harus mengandung @".to_string());
    }

    let parts: Vec<&str> = email.split('@').collect();
    if parts.len() != 2 {
        errors.push("Format email tidak valid".to_string());
        return Err(errors);
    }

    let (local, domain) = (parts[0], parts[1]);

    // Validate local part
    if local.is_empty() {
        errors.push("Bagian sebelum @ tidak boleh kosong".to_string());
    } else if local.len() > 64 {
        errors.push("Bagian sebelum @ terlalu panjang (maksimal 64 karakter)".to_string());
    }

    // Validate domain
    if domain.is_empty() {
        errors.push("Domain tidak boleh kosong".to_string());
    } else if !domain.contains('.') {
        errors.push("Domain harus mengandung titik".to_string());
    }

    // Government email domain preferences
    let gov_domains = &["kejaksaan.go.id", "gmail.com", "yahoo.com"];
    let is_preferred = gov_domains.iter().any(|&d| email.ends_with(d));

    if !is_preferred {
        // Warning, not error
        // errors.push("Disarankan menggunakan domain resmi atau email umum".to_string());
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

/// Validate Indonesian phone number dengan format normalization
pub fn validate_phone(phone: &str) -> Result<String, Vec<String>> {
    let mut errors = Vec::new();

    if phone.is_empty() {
        errors.push("Nomor telepon tidak boleh kosong".to_string());
        return Err(errors);
    }

    // Clean the phone number
    let cleaned = phone
        .chars()
        .filter(|c| c.is_ascii_digit() || *c == '+')
        .collect::<String>();

    if cleaned.is_empty() {
        errors.push("Nomor telepon harus mengandung angka".to_string());
        return Err(errors);
    }

    // Normalize to international format
    let normalized = if cleaned.starts_with("08") {
        format!("+62{}", &cleaned[1..])
    } else if cleaned.starts_with("62") && !cleaned.starts_with("+62") {
        format!("+{cleaned}")
    } else if cleaned.starts_with("+62") {
        cleaned
    } else {
        errors.push("Format nomor telepon Indonesia tidak valid".to_string());
        return Err(errors);
    };

    // Validate length (Indonesian mobile numbers)
    if normalized.len() < 12 || normalized.len() > 15 {
        errors.push("Panjang nomor telepon tidak valid (10-13 digit)".to_string());
    }

    if errors.is_empty() {
        Ok(normalized)
    } else {
        Err(errors)
    }
}

// ============================================================================
// UI HELPERS & UTILITIES
// ============================================================================

/// Generate unique component ID dengan prefix
pub fn generate_component_id(prefix: &str) -> String {
    use web_sys::js_sys;
    let timestamp = js_sys::Date::now() as u64;
    let random = (js_sys::Math::random() * 10000.0) as u32;
    format!("{prefix}-{timestamp}-{random}")
}

/// Get button variant berdasarkan status dengan context awareness
pub fn get_status_variant(status: &str, context: Option<&str>) -> ButtonVariant {
    let status_lower = status.to_lowercase();

    match context {
        Some("government") | Some("official") => match status_lower.as_str() {
            "approved" | "disetujui" | "aktif" => ButtonVariant::Success,
            "pending" | "menunggu" | "review" => ButtonVariant::Warning,
            "rejected" | "ditolak" | "nonaktif" => ButtonVariant::Danger,
            "draft" | "konsep" => ButtonVariant::Ghost,
            _ => ButtonVariant::Primary,
        },
        _ => match status_lower.as_str() {
            "success" | "berhasil" | "aktif" | "completed" => ButtonVariant::Success,
            "warning" | "peringatan" | "pending" | "processing" => ButtonVariant::Warning,
            "error" | "gagal" | "nonaktif" | "failed" => ButtonVariant::Danger,
            "secondary" | "sekunder" | "draft" => ButtonVariant::Secondary,
            _ => ButtonVariant::Primary,
        },
    }
}

/// Generate breadcrumbs dengan hierarchical support
pub fn generate_breadcrumbs(path: &str, labels: &HashMap<String, String>) -> Vec<BreadcrumbItem> {
    let mut items = vec![BreadcrumbItem {
        label: "Beranda".to_string(),
        href: Some("/".to_string()),
    }];
    if path == "/" {
        return items;
    }
    let path_parts: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
    let mut current_path = String::new();
    for part in path_parts.iter() {
        current_path.push('/');
        current_path.push_str(part);
        let label = labels
            .get(&current_path)
            .cloned()
            .unwrap_or_else(|| title_case(&part.replace(['-', '_'], " ")));
        items.push(BreadcrumbItem {
            label,
            href: Some(current_path.clone()),
        });
    }
    items
}

/// Check user permission dengan role hierarchy
pub fn check_permission(user_role: &UserRole, required_permission: &str) -> bool {
    match user_role {
        UserRole::SuperAdmin => true,
        UserRole::Admin => match required_permission {
            "read" | "create" | "update" | "delete" | "manage" => true,
            "system_config" | "user_management" => false,
            _ => false,
        },
        UserRole::User => match required_permission {
            "read" | "create" | "update" => true,
            "delete" | "manage" => false,
            _ => false,
        },
        UserRole::Viewer => matches!(required_permission, "read"),
        UserRole::Guest => match required_permission {
            "read" => true, // Limited read access
            _ => false,
        },
    }
}

/// Calculate pagination dengan edge case handling
pub fn calculate_pagination(
    current_page: usize,
    total_items: usize,
    items_per_page: usize,
) -> PaginatedResponse<()> {
    let page_size = items_per_page.max(1); // Prevent division by zero
    let total_pages = if total_items == 0 {
        1
    } else {
        total_items.div_ceil(page_size)
    };

    let page = current_page.max(1).min(total_pages);

    PaginatedResponse {
        data: vec![], // Empty for calculation only
        total: total_items,
        page,
        page_size,
        total_pages,
        has_next: page < total_pages,
        has_prev: page > 1,
    }
}

/// Format SK number sesuai standar Kejaksaan
pub fn format_sk_number(nomor_urut: u32, kode_unit: &str, tahun: i32, jenis_surat: &str) -> String {
    format!(
        "KEP-{:03}/{}/{}/{}",
        nomor_urut,
        jenis_surat.to_uppercase(),
        kode_unit.to_uppercase(),
        tahun
    )
}

// ============================================================================
// WEB STORAGE UTILITIES - Type-safe & Error-handled
// ============================================================================

/// High-level storage interface dengan automatic serialization
pub mod storage {
    use super::*;
    use web_sys::{window, Storage};

    /// Storage error types
    #[derive(Debug, Clone, PartialEq)]
    pub enum StorageError {
        NotAvailable,
        QuotaExceeded,
        SecurityError,
        SerializationError,
        UnknownError(String),
    }

    impl std::fmt::Display for StorageError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                StorageError::NotAvailable => write!(f, "Storage tidak tersedia"),
                StorageError::QuotaExceeded => write!(f, "Kapasitas storage penuh"),
                StorageError::SecurityError => write!(f, "Akses storage ditolak"),
                StorageError::SerializationError => write!(f, "Error serialization data"),
                StorageError::UnknownError(msg) => write!(f, "Error storage: {msg}"),
            }
        }
    }

    /// Get localStorage dengan error handling
    fn get_local_storage() -> Result<Storage, StorageError> {
        window()
            .and_then(|w| w.local_storage().ok().flatten())
            .ok_or(StorageError::NotAvailable)
    }

    /// Save string value to localStorage
    pub fn save_string(key: &str, value: &str) -> Result<(), StorageError> {
        let storage = get_local_storage()?;
        storage
            .set_item(key, value)
            .map_err(|_| StorageError::SecurityError)
    }

    /// Load string value from localStorage
    pub fn load_string(key: &str) -> Result<Option<String>, StorageError> {
        let storage = get_local_storage()?;
        storage
            .get_item(key)
            .map_err(|_| StorageError::SecurityError)
    }

    /// Save JSON-serializable value
    #[cfg(feature = "serde")]
    pub fn save_json<T: Serialize>(key: &str, value: &T) -> Result<(), StorageError> {
        let json = serde_json::to_string(value).map_err(|_| StorageError::SerializationError)?;
        save_string(key, &json)
    }

    /// Load JSON-serializable value
    #[cfg(feature = "serde")]
    pub fn load_json<T: for<'de> Deserialize<'de>>(key: &str) -> Result<Option<T>, StorageError> {
        if let Some(json) = load_string(key)? {
            serde_json::from_str(&json)
                .map(Some)
                .map_err(|_| StorageError::SerializationError)
        } else {
            Ok(None)
        }
    }

    /// Remove item from storage
    pub fn remove(key: &str) -> Result<(), StorageError> {
        let storage = get_local_storage()?;
        storage
            .remove_item(key)
            .map_err(|_| StorageError::SecurityError)
    }

    /// Clear all storage
    pub fn clear() -> Result<(), StorageError> {
        let storage = get_local_storage()?;
        storage.clear().map_err(|_| StorageError::SecurityError)
    }
}

// ============================================================================
// URL & QUERY UTILITIES
// ============================================================================

/// URL manipulation utilities
pub mod url {
    use std::collections::HashMap;

    /// Parse query parameters dari URL
    pub fn parse_query_params(url: &str) -> HashMap<String, String> {
        url.split('?')
            .nth(1)
            .map(|query| {
                query
                    .split('&')
                    .filter_map(|pair| {
                        let mut parts = pair.split('=');
                        let key = parts.next()?.to_string();
                        let value = parts.next().unwrap_or("").to_string();
                        Some((key, value))
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Build query string dari parameters
    pub fn build_query_string(params: &HashMap<String, String>) -> String {
        if params.is_empty() {
            return String::new();
        }

        let query_pairs: Vec<String> = params.iter().map(|(k, v)| format!("{k}={v}")).collect();

        format!("?{}", query_pairs.join("&"))
    }
}

// ============================================================================
// DATE & TIME UTILITIES
// ============================================================================

/// Date manipulation utilities
pub mod date {
    use super::*;

    /// Check if date is weekend
    pub fn is_weekend(date: NaiveDate) -> bool {
        matches!(date.weekday(), chrono::Weekday::Sat | chrono::Weekday::Sun)
    }

    /// Check if date is Indonesian holiday (basic implementation)
    pub fn is_holiday(date: NaiveDate) -> bool {
        // Basic Indonesian holidays - should be extended with proper holiday data
        let year = date.year();
        let holidays = vec![
            NaiveDate::from_ymd_opt(year, 1, 1),   // New Year
            NaiveDate::from_ymd_opt(year, 8, 17),  // Independence Day
            NaiveDate::from_ymd_opt(year, 12, 25), // Christmas
        ];

        holidays.into_iter().any(|h| h == Some(date))
    }

    /// Get working days between two dates
    pub fn working_days_between(start: NaiveDate, end: NaiveDate) -> i64 {
        let mut current = start;
        let mut count = 0i64;

        while current <= end {
            if !is_weekend(current) && !is_holiday(current) {
                count += 1;
            }
            current += chrono::Duration::days(1);
        }

        count
    }

    /// Format relative time (e.g., "2 jam yang lalu")
    pub fn format_relative_time(datetime: DateTime<Utc>) -> String {
        let now = Utc::now();
        let diff = now.signed_duration_since(datetime);

        if diff.num_days() > 0 {
            format!("{} hari yang lalu", diff.num_days())
        } else if diff.num_hours() > 0 {
            format!("{} jam yang lalu", diff.num_hours())
        } else if diff.num_minutes() > 0 {
            format!("{} menit yang lalu", diff.num_minutes())
        } else {
            "Baru saja".to_string()
        }
    }
}

// ============================================================================
// FORM VALIDATION UTILITIES
// ============================================================================

/// Form validation helpers
pub mod validation {
    use regex;

    /// Validate required field
    pub fn validate_required(value: &str, field_name: &str) -> Result<(), String> {
        if value.trim().is_empty() {
            Err(format!("{field_name} wajib diisi"))
        } else {
            Ok(())
        }
    }

    /// Validate minimum length
    pub fn validate_min_length(
        value: &str,
        min_length: usize,
        field_name: &str,
    ) -> Result<(), String> {
        if value.len() < min_length {
            Err(format!("{field_name} minimal {min_length} karakter"))
        } else {
            Ok(())
        }
    }

    /// Validate maximum length
    pub fn validate_max_length(
        value: &str,
        max_length: usize,
        field_name: &str,
    ) -> Result<(), String> {
        if value.len() > max_length {
            Err(format!("{field_name} maksimal {max_length} karakter"))
        } else {
            Ok(())
        }
    }

    /// Comprehensive form field validation
    pub fn validate_field<T: AsRef<str>>(
        value: T,
        field_name: &str,
        rules: &ValidationRules,
    ) -> Vec<String> {
        let value = value.as_ref();
        let mut errors = Vec::new();

        if rules.required {
            if let Err(e) = validate_required(value, field_name) {
                errors.push(e);
                return errors; // Don't continue if required field is empty
            }
        }

        if let Some(min) = rules.min_length {
            if let Err(e) = validate_min_length(value, min, field_name) {
                errors.push(e);
            }
        }

        if let Some(max) = rules.max_length {
            if let Err(e) = validate_max_length(value, max, field_name) {
                errors.push(e);
            }
        }

        if let Some(pattern) = &rules.pattern {
            if !pattern.is_match(value) {
                errors.push(format!("{field_name} format tidak valid"));
            }
        }

        errors
    }

    /// Validation rules untuk form fields
    #[derive(Debug, Clone, Default)]
    pub struct ValidationRules {
        pub required: bool,
        pub min_length: Option<usize>,
        pub max_length: Option<usize>,
        pub pattern: Option<regex::Regex>,
    }

    impl ValidationRules {
        pub fn required() -> Self {
            Self {
                required: true,
                ..Default::default()
            }
        }

        pub fn with_length_range(mut self, min: usize, max: usize) -> Self {
            self.min_length = Some(min);
            self.max_length = Some(max);
            self
        }
    }
}

/// High-level form utilities
pub mod forms {
    use super::*;

    /// Check if form has any errors
    pub fn has_errors(errors: &HashMap<String, Vec<String>>) -> bool {
        errors.values().any(|field_errors| !field_errors.is_empty())
    }

    /// Get first error for a field
    pub fn get_first_error<'a>(
        errors: &'a HashMap<String, Vec<String>>,
        field: &str,
    ) -> Option<&'a String> {
        errors.get(field)?.first()
    }

    /// Count total errors across all fields
    pub fn count_errors(errors: &HashMap<String, Vec<String>>) -> usize {
        errors.values().map(|field_errors| field_errors.len()).sum()
    }
}
