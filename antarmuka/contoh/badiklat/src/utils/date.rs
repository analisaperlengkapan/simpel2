//! Date and time utilities
//!
//! Helper functions for date formatting and manipulation

use chrono::{DateTime, Local, NaiveDateTime, Utc};

/// Format timestamp to Indonesian date format
pub fn format_date_id(timestamp: i64) -> String {
    let datetime = DateTime::<Utc>::from_timestamp(timestamp, 0)
        .unwrap_or_else(|| DateTime::from_timestamp(0, 0).unwrap());
    let local: DateTime<Local> = datetime.into();
    local.format("%d-%m-%Y").to_string()
}

/// Format timestamp to Indonesian datetime format
pub fn format_datetime_id(timestamp: i64) -> String {
    let datetime = DateTime::<Utc>::from_timestamp(timestamp, 0)
        .unwrap_or_else(|| DateTime::from_timestamp(0, 0).unwrap());
    let local: DateTime<Local> = datetime.into();
    local.format("%d-%m-%Y %H:%M:%S").to_string()
}

/// Get current timestamp
pub fn now_timestamp() -> i64 {
    Utc::now().timestamp()
}

/// Parse Indonesian date format to timestamp
pub fn parse_date_id(date_str: &str) -> Result<i64, String> {
    NaiveDateTime::parse_from_str(&format!("{} 00:00:00", date_str), "%d-%m-%Y %H:%M:%S")
        .map(|dt| dt.and_utc().timestamp())
        .map_err(|e| e.to_string())
}
