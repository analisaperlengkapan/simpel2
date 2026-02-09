//! Utility functions for Perlengkapan domain

use chrono::{DateTime, Utc};

/// Format currency in Indonesian Rupiah format
pub fn format_rupiah(amount: f64) -> String {
    let formatted = format!("{:.2}", amount);
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

    format!("Rp {},{}", grouped, decimal)
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
}
