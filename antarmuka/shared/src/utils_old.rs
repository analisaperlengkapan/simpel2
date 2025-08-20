//! Utility functions for the shared microfrontend library
//! Provides common functionality for formatting, validation, and UI helpers

use chrono::{DateTime, NaiveDate, Datelike};
use uuid::Uuid;
use regex::Regex;
use crate::types::*;
use web_sys::Window;

// ============================================================================
// WINDOW & DOM UTILITIES
// ============================================================================

/// Get the global window object
pub fn get_window() -> Option<Window> {
    web_sys::window()
}

// ============================================================================
// FORMATTING UTILITIES
// ============================================================================

/// Format amount to Indonesian currency format
pub fn format_currency(amount: f64) -> String {
    // Simple Indonesian Rupiah formatting
    let formatted = format!("{:.0}", amount);
    let mut result = String::new();
    let chars: Vec<char> = formatted.chars().collect();

    for (i, &ch) in chars.iter().enumerate() {
        if i > 0 && (chars.len() - i) % 3 == 0 {
            result.push('.');
        }
        result.push(ch);
    }

    format!("Rp {}", result)
}

/// Format date string to Indonesian format
pub fn format_date(date_str: &str) -> String {
    if let Ok(datetime) = DateTime::parse_from_rfc3339(date_str) {
        datetime.format("%d/%m/%Y").to_string()
    } else {
        date_str.to_string()
    }
}

/// Format file size in human readable format
pub fn format_file_size(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    const THRESHOLD: f64 = 1024.0;

    let size = bytes as f64;
    let mut unit_index = 0;
    let mut size_f = size;

    while size_f >= THRESHOLD && unit_index < UNITS.len() - 1 {
        size_f /= THRESHOLD;
        unit_index += 1;
    }

    if unit_index == 0 {
        format!("{} {}", size as u64, UNITS[unit_index])
    } else {
        format!("{:.1} {}", size_f, UNITS[unit_index])
    }
}

/// Truncate text to specified length with ellipsis
pub fn truncate_text(text: &str, max_length: usize) -> String {
    if text.len() <= max_length {
        text.to_string()
    } else {
        format!("{}...", &text[..max_length.saturating_sub(3)])
    }
}

/// Convert text to title case
pub fn title_case(text: &str) -> String {
    text.split_whitespace()
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().collect::<String>() + &chars.as_str().to_lowercase(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// Get initials from a name
pub fn get_initials(name: &str) -> String {
    name.split_whitespace()
        .filter_map(|word| word.chars().next())
        .take(2)
        .map(|c| c.to_uppercase().to_string())
        .collect::<String>()
}

/// Validasi NIP (Nomor Induk Pegawai) sesuai standar Kejaksaan
pub fn validate_nip(nip: &str) -> Result<(), String> {
    // NIP format: 18 digit, format YYYYMMDDNNNNNNNNN
    if nip.len() != 18 {
        return Err("NIP harus 18 digit".to_string());
    }

    if !nip.chars().all(|c| c.is_ascii_digit()) {
        return Err("NIP hanya boleh berisi angka".to_string());
    }

    // Validasi tahun (1950-2050)
    let year_part = &nip[0..4];
    if let Ok(year) = year_part.parse::<i32>() {
        if !(1950..=2050).contains(&year) {
            return Err("Tahun dalam NIP tidak valid".to_string());
        }
    } else {
        return Err("Format tahun dalam NIP tidak valid".to_string());
    }

    // Validasi bulan (01-12)
    let month_part = &nip[4..6];
    if let Ok(month) = month_part.parse::<i32>() {
        if !(1..=12).contains(&month) {
            return Err("Bulan dalam NIP tidak valid".to_string());
        }
    } else {
        return Err("Format bulan dalam NIP tidak valid".to_string());
    }

    // Validasi tanggal (01-31)
    let day_part = &nip[6..8];
    if let Ok(day) = day_part.parse::<i32>() {
        if !(1..=31).contains(&day) {
            return Err("Tanggal dalam NIP tidak valid".to_string());
        }
    } else {
        return Err("Format tanggal dalam NIP tidak valid".to_string());
    }

    Ok(())
}

/// Validate email format
pub fn is_valid_email(email: &str) -> bool {
    let email_regex = Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap();
    email_regex.is_match(email)
}

/// Validate Indonesian phone number
pub fn is_valid_phone(phone: &str) -> bool {
    let cleaned = phone.replace(&[' ', '-', '(', ')', '+'][..], "");
    cleaned.len() >= 10 &&
    (cleaned.starts_with("08") || cleaned.starts_with("628") || cleaned.starts_with("+628")) &&
    cleaned.chars().skip_while(|&c| c == '+').all(|c| c.is_numeric())
}

/// Generate unique ID for components
pub fn generate_id() -> String {
    Uuid::new_v4().to_string()
}

/// Get button variant based on status string
pub fn get_status_variant(status: &str) -> ButtonVariant {
    match status.to_lowercase().as_str() {
        "success" | "berhasil" | "aktif" => ButtonVariant::Success,
        "warning" | "peringatan" | "pending" => ButtonVariant::Warning,
        "error" | "gagal" | "nonaktif" => ButtonVariant::Error,
        "secondary" | "sekunder" => ButtonVariant::Secondary,
        _ => ButtonVariant::Primary,
    }
}

/// Convert enum value to display string
pub fn enum_to_display(value: &str) -> String {
    value.replace('_', " ")
         .split_whitespace()
         .map(|word| title_case(word))
         .collect::<Vec<_>>()
         .join(" ")
}

/// Generate breadcrumb items from path
pub fn generate_breadcrumbs(path: &str, base_title: &str) -> Vec<BreadcrumbItem> {
    let mut items = vec![BreadcrumbItem {
        label: "Beranda".to_string(),
        path: "/".to_string(),
        icon: Some("fa-home".to_string()),
    }];

    if !path.is_empty() && path != "/" {
        items.push(BreadcrumbItem {
            label: base_title.to_string(),
            path: path.to_string(),
            icon: Some("fa-folder".to_string()),
        });
    }

    items
}

/// Clean and format Indonesian phone number
pub fn format_phone(phone: &str) -> String {
    let cleaned = phone.replace(&[' ', '-', '(', ')'][..], "");
    if cleaned.starts_with("08") {
        format!("+62{}", &cleaned[1..])
    } else if cleaned.starts_with("628") {
        format!("+{}", cleaned)
    } else {
        cleaned
    }
}

/// Check if user has permission (placeholder for actual permission system)
pub fn has_permission(user_role: &str, required_permission: &str) -> bool {
    match user_role {
        "admin" => true,
        "operator" => match required_permission {
            "read" | "create" | "update" => true,
            "delete" => false,
            _ => false,
        },
        "viewer" => match required_permission {
            "read" => true,
            _ => false,
        },
        _ => false,
    }
}

/// Get current user session (placeholder)
pub fn get_current_user() -> Option<String> {
    None
}

/// Check if current user is authenticated
pub fn is_authenticated() -> bool {
    get_current_user().is_some()
}

/// Logout user (placeholder)
pub fn logout_user() {
    // Implementation here
}

/// Generate pagination info
pub fn calculate_pagination(current_page: usize, total_items: usize, items_per_page: usize) -> Pagination {
    let total_pages = (total_items + items_per_page - 1) / items_per_page;

    Pagination {
        page: current_page as i32,
        per_page: items_per_page as i32,
        total: total_items as i64,
        total_pages: total_pages as i32,
        has_previous: current_page > 1,
        has_next: current_page < total_pages,
    }
}

/// Generate table sorting info
pub fn get_sort_direction(current_column: &str, current_direction: &str, clicked_column: &str) -> String {
    if current_column == clicked_column {
        match current_direction {
            "asc" => "desc".to_string(),
            "desc" => "asc".to_string(),
            _ => "asc".to_string(),
        }
    } else {
        "asc".to_string()
    }
}

/// Format nomor surat keputusan/surat resmi Kejaksaan
pub fn format_sk_number(
    nomor_urut: u32,
    kode_unit: &str,
    tahun: i32,
    jenis_surat: &str
) -> String {
    format!(
        "{:03}/{}/{}/{}",
        nomor_urut,
        jenis_surat,
        kode_unit,
        tahun
    )
}

pub mod storage {
    use super::*;

    pub fn save_to_local_storage(key: &str, value: &str) -> Result<(), String> {
        if let Some(window) = get_window() {
            if let Ok(Some(storage)) = window.local_storage() {
                storage.set_item(key, value)
                    .map_err(|_| "Failed to save to localStorage".to_string())
            } else {
                Err("localStorage not available".to_string())
            }
        } else {
            Err("Window not available".to_string())
        }
    }

    pub fn load_from_local_storage(key: &str) -> Option<String> {
        if let Some(window) = get_window() {
            if let Ok(Some(storage)) = window.local_storage() {
                storage.get_item(key).ok().flatten()
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn get_item(key: &str) -> Option<String> {
        load_from_local_storage(key)
    }
}

pub mod url {
    pub fn get_query_param(url: &str, param: &str) -> Option<String> {
        url.split('?')
            .nth(1)?
            .split('&')
            .find_map(|pair| {
                let mut parts = pair.split('=');
                let key = parts.next()?;
                let value = parts.next()?;
                if key == param {
                    Some(value.to_string())
                } else {
                    None
                }
            })
    }
}

pub mod date {
    use super::*;

    pub fn is_weekend(date: NaiveDate) -> bool {
        let weekday = date.weekday();
        weekday == chrono::Weekday::Sat || weekday == chrono::Weekday::Sun
    }
}

pub mod validation {
    pub fn validate_required(value: &str) -> Result<(), String> {
        if value.trim().is_empty() {
            Err("Field ini wajib diisi".to_string())
        } else {
            Ok(())
        }
    }
}

pub mod forms {
    use std::collections::HashMap;

    pub fn has_form_errors(errors: &HashMap<String, Vec<String>>) -> bool {
        !errors.is_empty()
    }
}
