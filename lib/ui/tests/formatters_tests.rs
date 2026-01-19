//! Tests for Indonesian localization formatters

use chrono::{Datelike, Local, TimeZone, Timelike};
use lib_ui::utils::formatters::*;

#[test]
fn test_format_number_positive() {
    assert_eq!(format_number(0), "0");
    assert_eq!(format_number(100), "100");
    assert_eq!(format_number(1000), "1.000");
    assert_eq!(format_number(1000000), "1.000.000");
    assert_eq!(format_number(1234567890), "1.234.567.890");
}

#[test]
fn test_format_number_negative() {
    assert_eq!(format_number(-100), "-100");
    assert_eq!(format_number(-1000), "-1.000");
    assert_eq!(format_number(-1000000), "-1.000.000");
    assert_eq!(format_number(-1234567890), "-1.234.567.890");
}

#[test]
fn test_format_decimal() {
    assert_eq!(format_decimal(0.0, 2), "0,00");
    assert_eq!(format_decimal(100.5, 2), "100,50");
    assert_eq!(format_decimal(1000.75, 2), "1.000,75");
    assert_eq!(format_decimal(1000000.99, 2), "1.000.000,99");
    assert_eq!(format_decimal(1234567.123, 3), "1.234.567,123");
}

#[test]
fn test_format_decimal_negative() {
    assert_eq!(format_decimal(-100.5, 2), "-100,50");
    assert_eq!(format_decimal(-1000.75, 2), "-1.000,75");
    assert_eq!(format_decimal(-1000000.99, 2), "-1.000.000,99");
}

#[test]
fn test_format_decimal_no_decimals() {
    assert_eq!(format_decimal(1000.0, 0), "1.000");
    assert_eq!(format_decimal(1234567.0, 0), "1.234.567");
}

#[test]
fn test_format_currency() {
    assert_eq!(format_currency(0), "Rp 0");
    assert_eq!(format_currency(1000), "Rp 1.000");
    assert_eq!(format_currency(1000000), "Rp 1.000.000");
    assert_eq!(format_currency(-1000), "Rp -1.000");
}

#[test]
fn test_format_currency_decimal() {
    assert_eq!(format_currency_decimal(0.0), "Rp 0,00");
    assert_eq!(format_currency_decimal(1000.50), "Rp 1.000,50");
    assert_eq!(format_currency_decimal(1000000.99), "Rp 1.000.000,99");
    assert_eq!(format_currency_decimal(-1000.50), "Rp -1.000,50");
}

#[test]
fn test_format_currency_with_symbol() {
    assert_eq!(format_currency_with_symbol(1000, "USD"), "USD 1.000");
    assert_eq!(format_currency_with_symbol(1000000, "EUR"), "EUR 1.000.000");
}

#[test]
fn test_format_date() {
    let dt = Local.with_ymd_and_hms(2025, 10, 16, 14, 30, 0).unwrap();
    assert_eq!(format_date(&dt), "16/10/2025");
}

#[test]
fn test_format_datetime() {
    let dt = Local.with_ymd_and_hms(2025, 10, 16, 14, 30, 45).unwrap();
    assert_eq!(format_datetime(&dt), "16/10/2025 14:30:45");
}

#[test]
fn test_format_datetime_short() {
    let dt = Local.with_ymd_and_hms(2025, 10, 16, 14, 30, 45).unwrap();
    assert_eq!(format_datetime_short(&dt), "16/10/2025 14:30");
}

#[test]
fn test_format_time() {
    let dt = Local.with_ymd_and_hms(2025, 10, 16, 14, 30, 45).unwrap();
    assert_eq!(format_time(&dt), "14:30:45");
}

#[test]
fn test_format_time_short() {
    let dt = Local.with_ymd_and_hms(2025, 10, 16, 14, 30, 45).unwrap();
    assert_eq!(format_time_short(&dt), "14:30");
}

#[test]
fn test_parse_date() {
    let parsed = parse_date("16/10/2025");
    assert!(parsed.is_some());
    let date = parsed.unwrap();
    assert_eq!(date.day(), 16);
    assert_eq!(date.month(), 10);
    assert_eq!(date.year(), 2025);
}

#[test]
fn test_parse_date_invalid() {
    assert!(parse_date("invalid").is_none());
    assert!(parse_date("32/13/2025").is_none());
    assert!(parse_date("2025-10-16").is_none()); // Wrong format
}

#[test]
fn test_parse_datetime() {
    let parsed = parse_datetime("16/10/2025 14:30:45");
    assert!(parsed.is_some());
    let dt = parsed.unwrap();
    assert_eq!(dt.day(), 16);
    assert_eq!(dt.month(), 10);
    assert_eq!(dt.year(), 2025);
    assert_eq!(dt.hour(), 14);
    assert_eq!(dt.minute(), 30);
    assert_eq!(dt.second(), 45);
}

#[test]
fn test_format_date_indonesian() {
    let dt = Local.with_ymd_and_hms(2025, 1, 15, 0, 0, 0).unwrap();
    assert_eq!(format_date_indonesian(&dt), "15 Januari 2025");

    let dt = Local.with_ymd_and_hms(2025, 10, 16, 0, 0, 0).unwrap();
    assert_eq!(format_date_indonesian(&dt), "16 Oktober 2025");

    let dt = Local.with_ymd_and_hms(2025, 12, 31, 0, 0, 0).unwrap();
    assert_eq!(format_date_indonesian(&dt), "31 Desember 2025");
}

#[test]
fn test_format_date_full_indonesian() {
    // Note: This test assumes the date falls on the expected day of week
    let dt = Local.with_ymd_and_hms(2025, 10, 16, 0, 0, 0).unwrap();
    let formatted = format_date_full_indonesian(&dt);
    assert!(formatted.contains("16 Oktober 2025"));
    assert!(formatted.contains(","));
}

#[test]
fn test_format_relative_time() {
    let now = Local::now();

    // Just now
    assert_eq!(format_relative_time(&now), "Baru saja");

    // Minutes ago
    let five_min_ago = now - chrono::Duration::minutes(5);
    assert_eq!(format_relative_time(&five_min_ago), "5 menit yang lalu");

    // Hours ago
    let two_hours_ago = now - chrono::Duration::hours(2);
    assert_eq!(format_relative_time(&two_hours_ago), "2 jam yang lalu");

    // Days ago
    let three_days_ago = now - chrono::Duration::days(3);
    assert_eq!(format_relative_time(&three_days_ago), "3 hari yang lalu");
}

#[test]
fn test_format_file_size() {
    assert_eq!(format_file_size(0), "0 B");
    assert_eq!(format_file_size(500), "500 B");
    assert_eq!(format_file_size(1024), "1.00 KB");
    assert_eq!(format_file_size(1536), "1.50 KB");
    assert_eq!(format_file_size(1048576), "1.00 MB");
    assert_eq!(format_file_size(1073741824), "1.00 GB");
}

#[test]
fn test_format_percentage() {
    assert_eq!(format_percentage(0.0), "0%");
    assert_eq!(format_percentage(0.5), "50%");
    assert_eq!(format_percentage(0.75), "75%");
    assert_eq!(format_percentage(1.0), "100%");
}

#[test]
fn test_format_percentage_decimal() {
    assert_eq!(format_percentage_decimal(0.7534, 2), "75,34%");
    assert_eq!(format_percentage_decimal(0.12345, 3), "12,345%");
    assert_eq!(format_percentage_decimal(1.0, 2), "100,00%");
}

#[test]
fn test_truncate_string() {
    assert_eq!(truncate_string("Hello", 10), "Hello");
    assert_eq!(truncate_string("Hello World", 8), "Hello...");
    assert_eq!(truncate_string("Test", 3), "...");
    assert_eq!(truncate_string("", 5), "");
}

#[test]
fn test_capitalize() {
    assert_eq!(capitalize("hello"), "Hello");
    assert_eq!(capitalize("HELLO"), "HELLO");
    assert_eq!(capitalize("h"), "H");
    assert_eq!(capitalize(""), "");
}

#[test]
fn test_title_case() {
    assert_eq!(title_case("hello world"), "Hello World");
    assert_eq!(title_case("the quick brown fox"), "The Quick Brown Fox");
    assert_eq!(title_case(""), "");
}

#[test]
fn test_format_phone_number() {
    // Mobile numbers
    assert_eq!(format_phone_number("081234567890"), "0812-3456-7890");
    assert_eq!(format_phone_number("08123456789"), "0812-3456-789");
    assert_eq!(format_phone_number("0812 3456 7890"), "0812-3456-7890");

    // Landline numbers
    assert_eq!(format_phone_number("0211234567"), "021-1234-567");
    assert_eq!(format_phone_number("02112345678"), "021-1234-5678");

    // Short numbers
    assert_eq!(format_phone_number("123"), "123");
}

#[test]
fn test_format_nik() {
    assert_eq!(format_nik("1234567890123456"), "1234-5678-9012-3456");
    assert_eq!(format_nik("1234 5678 9012 3456"), "1234-5678-9012-3456");

    // Invalid length
    assert_eq!(format_nik("12345"), "12345");
}

#[test]
fn test_format_npwp() {
    assert_eq!(format_npwp("123456789012345"), "12.345.678.9-012.345");
    assert_eq!(format_npwp("12.345.678.9-012.345"), "12.345.678.9-012.345");

    // Invalid length
    assert_eq!(format_npwp("12345"), "12345");
}
