//! Utility functions for Perlengkapan domain

use chrono::{DateTime, Utc};

/// Format currency in Indonesian Rupiah format
///
/// The sign is set apart BEFORE the digits are grouped in threes from the right: grouped with the
/// minus sign in the string, a number whose digit count is a multiple of three put the sign in a
/// group of its own, and -125000 read "Rp -.125.000,00". A negative amount reads "Rp -125.000,00";
/// one that rounds to zero is just "Rp 0,00" (never "-0,00").
pub fn format_rupiah(amount: f64) -> String {
    let formatted = format!("{:.2}", amount.abs());
    let sign = if amount < 0.0 && formatted != "0.00" {
        "-"
    } else {
        ""
    };
    let parts: Vec<&str> = formatted.split('.').collect();
    let whole = parts[0];
    let decimal = parts.get(1).unwrap_or(&"00");

    // Add thousand separators
    let chars: Vec<char> = whole.chars().rev().collect();
    let grouped: String = chars
        .chunks(3)
        .map(|chunk| chunk.iter().collect::<String>())
        .collect::<Vec<String>>()
        .join(".")
        .chars()
        .rev()
        .collect();

    format!("Rp {sign}{grouped},{decimal}")
}

/// Format datetime for display
pub fn format_datetime(dt: DateTime<Utc>) -> String {
    dt.format("%d-%m-%Y %H:%M").to_string()
}

/// Format date for display
pub fn format_date(dt: DateTime<Utc>) -> String {
    dt.format("%d-%m-%Y").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_rupiah() {
        assert_eq!(format_rupiah(1000.0), "Rp 1.000,00");
        assert_eq!(format_rupiah(1000000.0), "Rp 1.000.000,00");
        assert_eq!(format_rupiah(1234567.89), "Rp 1.234.567,89");
    }

    #[test]
    fn a_negative_amount_keeps_its_sign_outside_the_digit_groups() {
        // A digit count that is a multiple of three used to put the minus sign in a group of its
        // own: "Rp -.125.000,00".
        assert_eq!(format_rupiah(-125000.0), "Rp -125.000,00");
        assert_eq!(format_rupiah(-1234567.89), "Rp -1.234.567,89");
        assert_eq!(format_rupiah(-5.5), "Rp -5,50");
        // Zero and what rounds to zero carry no sign.
        assert_eq!(format_rupiah(0.0), "Rp 0,00");
        assert_eq!(format_rupiah(-0.0), "Rp 0,00");
        assert_eq!(format_rupiah(-0.001), "Rp 0,00");
    }
}
