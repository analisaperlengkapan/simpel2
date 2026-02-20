//! Anomaly Detection Service
//!
//! This module provides anomaly detection for tracking user IP addresses
//! and detecting suspicious activity patterns.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Trait for anomaly detection functionality
pub trait AnomalyDetectorTrait: Send + Sync {
    /// Check if an IP address is new for a given user
    fn is_new_ip(&self, user_id: &str, ip: &str) -> Result<bool, String>;
}

/// Anomaly detector for tracking user IP addresses and detecting suspicious activity
pub struct AnomalyDetector {
    /// Map of user IDs to their known IP addresses for anomaly detection
    known_ips: Arc<RwLock<HashMap<String, Vec<String>>>>,
}

impl Default for AnomalyDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl AnomalyDetector {
    /// Create a new anomaly detector instance
    pub fn new() -> Self {
        Self {
            known_ips: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Check if an IP address is new for a given user (async version)
    pub async fn is_new_ip_async(&self, user_id: &str, ip: &str) -> Result<bool, String> {
        let mut map = self.known_ips.write().await;
        let ips = map.entry(user_id.to_string()).or_default();
        if !ips.contains(&ip.to_string()) {
            ips.push(ip.to_string());
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl AnomalyDetectorTrait for AnomalyDetector {
    /// Check if an IP address is new for a given user (sync version for trait)
    fn is_new_ip(&self, user_id: &str, ip: &str) -> Result<bool, String> {
        // For the sync trait implementation, we use a blocking approach
        // In production, prefer using is_new_ip_async
        let runtime = tokio::runtime::Handle::try_current()
            .map_err(|_| "No tokio runtime available".to_string())?;

        runtime.block_on(async {
            self.is_new_ip_async(user_id, ip).await
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_anomaly_detector_creation() {
        let detector = AnomalyDetector::new();
        assert!(detector.known_ips.read().await.is_empty());
    }

    #[tokio::test]
    async fn test_anomaly_detector_default() {
        let detector = AnomalyDetector::default();
        assert!(detector.known_ips.read().await.is_empty());
    }

    #[tokio::test]
    async fn test_is_new_ip_first_ip() {
        let detector = AnomalyDetector::new();
        let result = detector.is_new_ip_async("user1", "192.168.1.1").await.unwrap();
        assert!(result); // First IP should be new

        let ips = detector.known_ips.read().await;
        assert_eq!(ips.len(), 1);
        assert!(ips.contains_key("user1"));
        assert_eq!(ips["user1"], vec!["192.168.1.1"]);
    }

    #[tokio::test]
    async fn test_is_new_ip_existing_ip() {
        let detector = AnomalyDetector::new();

        // First call should return true (new IP)
        let result1 = detector.is_new_ip_async("user1", "192.168.1.1").await.unwrap();
        assert!(result1);

        // Second call with same IP should return false (not new)
        let result2 = detector.is_new_ip_async("user1", "192.168.1.1").await.unwrap();
        assert!(!result2);

        let ips = detector.known_ips.read().await;
        assert_eq!(ips["user1"], vec!["192.168.1.1"]);
    }

    #[tokio::test]
    async fn test_is_new_ip_multiple_ips_same_user() {
        let detector = AnomalyDetector::new();

        // Add first IP
        let result1 = detector.is_new_ip_async("user1", "192.168.1.1").await.unwrap();
        assert!(result1);

        // Add second IP
        let result2 = detector.is_new_ip_async("user1", "192.168.1.2").await.unwrap();
        assert!(result2);

        // Check first IP again
        let result3 = detector.is_new_ip_async("user1", "192.168.1.1").await.unwrap();
        assert!(!result3);

        let ips = detector.known_ips.read().await;
        assert_eq!(ips["user1"], vec!["192.168.1.1", "192.168.1.2"]);
    }

    #[tokio::test]
    async fn test_is_new_ip_different_users() {
        let detector = AnomalyDetector::new();

        // User 1
        let result1 = detector.is_new_ip_async("user1", "192.168.1.1").await.unwrap();
        assert!(result1);

        // User 2 with same IP
        let result2 = detector.is_new_ip_async("user2", "192.168.1.1").await.unwrap();
        assert!(result2); // Should be new for user2

        // User 1 with different IP
        let result3 = detector.is_new_ip_async("user1", "192.168.1.2").await.unwrap();
        assert!(result3);

        let ips = detector.known_ips.read().await;
        assert_eq!(ips.len(), 2);
        assert_eq!(ips["user1"], vec!["192.168.1.1", "192.168.1.2"]);
        assert_eq!(ips["user2"], vec!["192.168.1.1"]);
    }
}
