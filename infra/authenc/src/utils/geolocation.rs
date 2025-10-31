//! Geolocation utilities for IP address lookup
//!
//! Provides optional geolocation data for audit logging.
//! This is a lightweight implementation that can be extended with
//! external services like MaxMind GeoIP2 or IP2Location.

use serde::{Deserialize, Serialize};
use std::net::IpAddr;

/// Geolocation data for an IP address
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeolocationData {
    /// Country code (ISO 3166-1 alpha-2)
    pub country_code: Option<String>,
    /// Country name
    pub country_name: Option<String>,
    /// City name
    pub city: Option<String>,
    /// Region/state name
    pub region: Option<String>,
    /// Latitude
    pub latitude: Option<f64>,
    /// Longitude
    pub longitude: Option<f64>,
    /// Timezone
    pub timezone: Option<String>,
}

/// Geolocation service trait for extensibility
#[async_trait::async_trait]
pub trait GeolocationService: Send + Sync {
    /// Lookup geolocation data for an IP address
    async fn lookup(&self, ip: &str) -> Option<GeolocationData>;
}

/// Simple geolocation service that identifies private/local IPs
/// Can be extended to use external services
pub struct SimpleGeolocationService;

impl SimpleGeolocationService {
    pub fn new() -> Self {
        Self
    }

    /// Check if an IP is private/local
    fn is_private_ip(ip: &IpAddr) -> bool {
        match ip {
            IpAddr::V4(ipv4) => {
                ipv4.is_private()
                    || ipv4.is_loopback()
                    || ipv4.is_link_local()
                    || ipv4.is_broadcast()
            }
            IpAddr::V6(ipv6) => ipv6.is_loopback() || ipv6.is_unicast_link_local(),
        }
    }
}

impl Default for SimpleGeolocationService {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl GeolocationService for SimpleGeolocationService {
    async fn lookup(&self, ip: &str) -> Option<GeolocationData> {
        // Parse IP address
        let ip_addr = ip.parse::<IpAddr>().ok()?;

        // For private/local IPs, return basic info
        if Self::is_private_ip(&ip_addr) {
            return Some(GeolocationData {
                country_code: Some("XX".to_string()),
                country_name: Some("Private Network".to_string()),
                city: Some("Local".to_string()),
                region: None,
                latitude: None,
                longitude: None,
                timezone: None,
            });
        }

        // For public IPs, return None (can be extended with external service)
        // In production, integrate with MaxMind GeoIP2, IP2Location, or similar
        None
    }
}

/// Stub for MaxMind GeoIP2 integration (future implementation)
#[allow(dead_code)]
pub struct MaxMindGeolocationService {
    // database_path: String,
}

#[allow(dead_code)]
impl MaxMindGeolocationService {
    /// Create a new MaxMind geolocation service
    ///
    /// # Example Integration
    /// ```ignore
    /// use maxminddb::Reader;
    ///
    /// let reader = Reader::open_readfile("GeoLite2-City.mmdb")?;
    /// let service = MaxMindGeolocationService::new(reader);
    /// ```
    pub fn new() -> Self {
        Self {
            // database_path: database_path.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_private_ip_lookup() {
        let service = SimpleGeolocationService::new();

        let result = service.lookup("192.168.1.1").await;
        assert!(result.is_some());

        let geo = result.unwrap();
        assert_eq!(geo.country_code, Some("XX".to_string()));
        assert_eq!(geo.country_name, Some("Private Network".to_string()));
    }

    #[tokio::test]
    async fn test_loopback_ip_lookup() {
        let service = SimpleGeolocationService::new();

        let result = service.lookup("127.0.0.1").await;
        assert!(result.is_some());

        let geo = result.unwrap();
        assert_eq!(geo.country_code, Some("XX".to_string()));
    }

    #[tokio::test]
    async fn test_public_ip_lookup() {
        let service = SimpleGeolocationService::new();

        // Public IP - returns None without external service
        let result = service.lookup("8.8.8.8").await;
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_invalid_ip() {
        let service = SimpleGeolocationService::new();

        let result = service.lookup("invalid-ip").await;
        assert!(result.is_none());
    }
}
