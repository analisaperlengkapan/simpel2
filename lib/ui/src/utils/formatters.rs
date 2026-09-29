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

/// Indonesian short month names, indexed by `month() - 1`.
const BULAN_SINGKAT: [&str; 12] = [
    "Jan", "Feb", "Mar", "Apr", "Mei", "Jun", "Jul", "Agu", "Sep", "Okt", "Nov", "Des",
];

/// `"2026-08-28T09:26:04.708688+00:00"` → `"28 Agu 2026, 16:26"` for a reader
/// in WIB.
///
/// Backends here store timestamps in UTC and serialise them RFC 3339. Four
/// surfaces printed that string exactly as stored — microseconds, offset and
/// all — and because the offset was `+00:00` while the reader is in WIB, the
/// hour shown was **wrong by seven**, not merely ugly: a notification created
/// at 16:26 read 09:26. `with_timezone(&Local)` moves it to the reader's own
/// clock (in wasm, the browser's).
///
/// Lives here rather than beside any one caller because every hand-written
/// copy of a mapping in this repo has eventually drifted from its siblings.
///
/// A value that will not parse is returned unchanged: hiding data behind a
/// formatter error is worse than showing it in the wrong shape.
pub fn format_iso_local(iso: &str) -> String {
    use chrono::{Datelike, Timelike};
    match DateTime::parse_from_rfc3339(iso) {
        Ok(dt) => {
            let local = dt.with_timezone(&Local);
            format!(
                "{} {} {}, {:02}:{:02}",
                local.day(),
                BULAN_SINGKAT[(local.month() as usize).saturating_sub(1).min(11)],
                local.year(),
                local.hour(),
                local.minute(),
            )
        }
        Err(_) => iso.to_string(),
    }
}

/// Same as [`format_iso_local`] for an optional value; `None` and an empty
/// string both render as an em dash rather than as "None".
pub fn format_iso_local_opt(iso: Option<&str>) -> String {
    match iso {
        Some(s) if !s.trim().is_empty() => format_iso_local(s),
        _ => "—".to_string(),
    }
}

/// Turn a raw `User-Agent` header into a short, human label.
///
/// The header is a protocol artifact — `Mozilla/5.0 (X11; Linux x86_64)
/// AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0 Safari/537.36` — and
/// printing it verbatim into a session card shows the reader a parser puzzle
/// instead of the one fact they came for: which device is logged in.
///
/// The ORDER of the checks is the whole trick. Every browser lies about being
/// every other browser: Chrome claims `Safari` and `Mozilla`, Edge claims
/// `Chrome` and `Safari`, and Opera claims all three. Testing `Safari` first
/// would label Edge as Safari. So the most specific brands are matched before
/// the ones they impersonate.
pub fn device_label(user_agent: &str) -> String {
    let ua = user_agent.to_lowercase();

    let platform = if ua.contains("iphone") {
        "iPhone"
    } else if ua.contains("ipad") {
        "iPad"
    } else if ua.contains("android") {
        "Android"
    } else if ua.contains("windows") {
        "Windows"
    } else if ua.contains("mac os") || ua.contains("macintosh") {
        "macOS"
    } else if ua.contains("linux") {
        "Linux"
    } else {
        ""
    };

    let browser = if ua.contains("edg/") || ua.contains("edgios") || ua.contains("edga/") {
        "Edge"
    } else if ua.contains("opr/") || ua.contains("opera") {
        "Opera"
    } else if ua.contains("firefox") || ua.contains("fxios") {
        "Firefox"
    } else if ua.contains("chrome") || ua.contains("crios") {
        "Chrome"
    } else if ua.contains("safari") {
        "Safari"
    } else {
        ""
    };

    match (browser, platform) {
        ("", "") => "Perangkat tidak dikenal".to_string(),
        (b, "") => b.to_string(),
        ("", p) => p.to_string(),
        (b, p) => format!("{b} di {p}"),
    }
}

#[cfg(test)]
mod iso_local_tests {
    use super::{format_iso_local, format_iso_local_opt};

    /// The rendered hour depends on the reader's clock, so pin what does not:
    /// none of the storage format may survive to the screen.
    #[test]
    fn the_storage_format_never_reaches_the_reader() {
        let out = format_iso_local("2026-08-28T09:26:04.708688+00:00");
        assert!(!out.contains('T'), "ISO separator survived: {out}");
        assert!(!out.contains("+00:00"), "offset survived: {out}");
        assert!(!out.contains("708688"), "microseconds survived: {out}");
        assert!(out.contains("2026"), "the year should remain: {out}");

        // The `Z` spelling is the one the helpdesk tickets use.
        let z = format_iso_local("2026-08-28T09:28:53.886456Z");
        assert!(!z.contains('Z') && !z.contains('T'), "Z form survived: {z}");
    }

    /// Both ends of the month table must be reachable: December must not fall
    /// off the end, January must not wrap to December.
    #[test]
    fn both_ends_of_the_month_table_are_reachable() {
        assert!(format_iso_local("2026-01-15T00:00:00+07:00").contains("Jan"));
        assert!(format_iso_local("2026-12-15T00:00:00+07:00").contains("Des"));
    }

    #[test]
    fn an_unparseable_value_is_passed_through_not_hidden() {
        assert_eq!(format_iso_local("kemarin"), "kemarin");
        assert_eq!(format_iso_local_opt(None), "—");
        assert_eq!(format_iso_local_opt(Some("  ")), "—");
    }
}

#[cfg(test)]
mod device_label_tests {
    use super::device_label;

    /// The whole point of the ordering: Edge and Chrome both claim to be
    /// Safari, and Opera claims to be Chrome. A naive "check Safari first"
    /// table labels two of these three wrongly.
    #[test]
    fn impersonating_browsers_are_not_mislabelled() {
        let edge = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36 Edg/120.0.0.0";
        assert_eq!(device_label(edge), "Edge di Windows");

        let opera = "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/119.0.0.0 Safari/537.36 OPR/105.0.0.0";
        assert_eq!(device_label(opera), "Opera di Linux");

        let chrome = "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36";
        assert_eq!(device_label(chrome), "Chrome di Linux");
    }

    #[test]
    fn mobile_platforms_and_the_empty_case() {
        let iphone = "Mozilla/5.0 (iPhone; CPU iPhone OS 17_0 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.0 Mobile/15E148 Safari/604.1";
        assert_eq!(device_label(iphone), "Safari di iPhone");

        assert_eq!(device_label(""), "Perangkat tidak dikenal");
        assert_eq!(device_label("curl/8.5.0"), "Perangkat tidak dikenal");
    }
}
