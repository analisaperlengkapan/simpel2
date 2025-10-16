//! Data formatting utilities untuk Indonesia
//!
//! This module provides Indonesian localization formatters for:
//! - Dates (DD/MM/YYYY format)
//! - Numbers (dot as thousand separator)
//! - Currency (Rupiah with Rp prefix)
//! - Time (relative and absolute)
//! - File sizes, percentages, and more

use chrono::{DateTime, Local, NaiveDate, NaiveDateTime};

/// Format integer number dengan Indonesian thousand separator
/// Example: 1000000 -> "1.000.000", -1000000 -> "-1.000.000"
pub fn format_number(num: i64) -> String {
    let is_negative = num < 0;
    let abs_num = num.abs().to_string();
    let mut formatted = String::new();

    for (i, c) in abs_num.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            formatted.push('.');
        }
        formatted.push(c);
    }

    let result: String = formatted.chars().rev().collect();
    if is_negative {
        format!("-{}", result)
    } else {
        result
    }
}

/// Format float number dengan Indonesian separators
/// Example: 1000000.50 -> "1.000.000,50"
pub fn format_decimal(num: f64, decimal_places: usize) -> String {
    let is_negative = num < 0.0;
    let abs_num = num.abs();

    // Split into integer and decimal parts
    let integer_part = abs_num.floor() as i64;
    let decimal_part =
        ((abs_num - abs_num.floor()) * 10_f64.powi(decimal_places as i32)).round() as i64;

    let formatted_integer = format_number(integer_part);
    let formatted_decimal = format!("{:0width$}", decimal_part, width = decimal_places);

    let result = if decimal_places > 0 {
        format!("{},{}", formatted_integer, formatted_decimal)
    } else {
        formatted_integer
    };

    if is_negative {
        format!("-{}", result)
    } else {
        result
    }
}

/// Format currency to Indonesian Rupiah (integer)
/// Example: 1000000 -> "Rp 1.000.000"
pub fn format_currency(amount: i64) -> String {
    format!("Rp {}", format_number(amount))
}

/// Format currency to Indonesian Rupiah (decimal)
/// Example: 1000000.50 -> "Rp 1.000.000,50"
pub fn format_currency_decimal(amount: f64) -> String {
    format!("Rp {}", format_decimal(amount, 2))
}

/// Format currency with custom symbol
/// Example: format_currency_with_symbol(1000000, "USD") -> "USD 1.000.000"
pub fn format_currency_with_symbol(amount: i64, symbol: &str) -> String {
    format!("{} {}", symbol, format_number(amount))
}

/// Format date to Indonesian format (DD/MM/YYYY)
pub fn format_date(datetime: &DateTime<Local>) -> String {
    datetime.format("%d/%m/%Y").to_string()
}

/// Format datetime to Indonesian format (DD/MM/YYYY HH:MM:SS)
pub fn format_datetime(datetime: &DateTime<Local>) -> String {
    datetime.format("%d/%m/%Y %H:%M:%S").to_string()
}

/// Format datetime to Indonesian format with short time (DD/MM/YYYY HH:MM)
pub fn format_datetime_short(datetime: &DateTime<Local>) -> String {
    datetime.format("%d/%m/%Y %H:%M").to_string()
}

/// Format time to Indonesian format (HH:MM:SS)
pub fn format_time(datetime: &DateTime<Local>) -> String {
    datetime.format("%H:%M:%S").to_string()
}

/// Format time to Indonesian format (HH:MM)
pub fn format_time_short(datetime: &DateTime<Local>) -> String {
    datetime.format("%H:%M").to_string()
}

/// Parse date from Indonesian format (DD/MM/YYYY)
/// Returns None if parsing fails
pub fn parse_date(date_str: &str) -> Option<NaiveDate> {
    NaiveDate::parse_from_str(date_str, "%d/%m/%Y").ok()
}

/// Parse datetime from Indonesian format (DD/MM/YYYY HH:MM:SS)
/// Returns None if parsing fails
pub fn parse_datetime(datetime_str: &str) -> Option<NaiveDateTime> {
    NaiveDateTime::parse_from_str(datetime_str, "%d/%m/%Y %H:%M:%S").ok()
}

/// Format date with Indonesian month names
/// Example: "15 Oktober 2025"
pub fn format_date_indonesian(datetime: &DateTime<Local>) -> String {
    let day = datetime.format("%d").to_string();
    let month = match datetime.format("%m").to_string().as_str() {
        "01" => "Januari",
        "02" => "Februari",
        "03" => "Maret",
        "04" => "April",
        "05" => "Mei",
        "06" => "Juni",
        "07" => "Juli",
        "08" => "Agustus",
        "09" => "September",
        "10" => "Oktober",
        "11" => "November",
        "12" => "Desember",
        _ => "Unknown",
    };
    let year = datetime.format("%Y").to_string();
    format!("{} {} {}", day, month, year)
}

/// Format date with Indonesian day and month names
/// Example: "Kamis, 15 Oktober 2025"
pub fn format_date_full_indonesian(datetime: &DateTime<Local>) -> String {
    let day_name = match datetime.format("%u").to_string().as_str() {
        "1" => "Senin",
        "2" => "Selasa",
        "3" => "Rabu",
        "4" => "Kamis",
        "5" => "Jumat",
        "6" => "Sabtu",
        "7" => "Minggu",
        _ => "Unknown",
    };
    format!("{}, {}", day_name, format_date_indonesian(datetime))
}

/// Format relative time (Indonesian)
/// Example: "5 menit yang lalu", "2 jam yang lalu"
pub fn format_relative_time(datetime: &DateTime<Local>) -> String {
    let now = Local::now();
    let diff = now.signed_duration_since(*datetime);

    if diff.num_seconds() < 60 {
        "Baru saja".to_string()
    } else if diff.num_minutes() < 60 {
        format!("{} menit yang lalu", diff.num_minutes())
    } else if diff.num_hours() < 24 {
        format!("{} jam yang lalu", diff.num_hours())
    } else if diff.num_days() < 30 {
        format!("{} hari yang lalu", diff.num_days())
    } else if diff.num_days() < 365 {
        format!("{} bulan yang lalu", diff.num_days() / 30)
    } else {
        format!("{} tahun yang lalu", diff.num_days() / 365)
    }
}

/// Truncate string dengan ellipsis
/// Example: "Lorem ipsum dolor" with max 10 -> "Lorem ip..."
pub fn truncate_string(s: &str, max_length: usize) -> String {
    if s.len() <= max_length {
        s.to_string()
    } else {
        format!("{}...", &s[..max_length.saturating_sub(3)])
    }
}

/// Format file size
/// Example: 1024 -> "1 KB", 1048576 -> "1 MB"
pub fn format_file_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes < KB {
        format!("{} B", bytes)
    } else if bytes < MB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else if bytes < GB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    }
}

/// Format percentage
/// Example: 0.75 -> "75%"
pub fn format_percentage(value: f64) -> String {
    format!("{:.0}%", value * 100.0)
}

/// Format percentage with decimal places
/// Example: 0.7534 -> "75,34%"
pub fn format_percentage_decimal(value: f64, decimal_places: usize) -> String {
    let percentage = value * 100.0;
    format!("{}%", format_decimal(percentage, decimal_places))
}

/// Format phone number (Indonesian format)
/// Example: "081234567890" -> "0812-3456-7890"
pub fn format_phone_number(phone: &str) -> String {
    let digits: String = phone.chars().filter(|c| c.is_ascii_digit()).collect();

    if digits.len() < 10 {
        return digits;
    }

    // Format: 0XXX-XXXX-XXXX or 0XX-XXXX-XXXX
    if digits.starts_with("08") {
        // Mobile format: 08XX-XXXX-XXXX
        if digits.len() >= 11 {
            format!("{}-{}-{}", &digits[0..4], &digits[4..8], &digits[8..])
        } else {
            digits
        }
    } else if digits.starts_with("0") {
        // Landline format: 0XX-XXXX-XXXX
        if digits.len() >= 10 {
            format!("{}-{}-{}", &digits[0..3], &digits[3..7], &digits[7..])
        } else {
            digits
        }
    } else {
        digits
    }
}

/// Format Indonesian ID number (NIK)
/// Example: "1234567890123456" -> "1234-5678-9012-3456"
pub fn format_nik(nik: &str) -> String {
    let digits: String = nik.chars().filter(|c| c.is_ascii_digit()).collect();

    if digits.len() != 16 {
        return digits;
    }

    format!(
        "{}-{}-{}-{}",
        &digits[0..4],
        &digits[4..8],
        &digits[8..12],
        &digits[12..16]
    )
}

/// Format NPWP (Indonesian tax ID)
/// Example: "123456789012345" -> "12.345.678.9-012.345"
pub fn format_npwp(npwp: &str) -> String {
    let digits: String = npwp.chars().filter(|c| c.is_ascii_digit()).collect();

    if digits.len() != 15 {
        return digits;
    }

    format!(
        "{}.{}.{}.{}-{}.{}",
        &digits[0..2],
        &digits[2..5],
        &digits[5..8],
        &digits[8..9],
        &digits[9..12],
        &digits[12..15]
    )
}

/// Capitalize first letter
pub fn capitalize(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().chain(chars).collect(),
    }
}

/// Convert to title case
pub fn title_case(s: &str) -> String {
    s.split_whitespace()
        .map(capitalize)
        .collect::<Vec<_>>()
        .join(" ")
}
