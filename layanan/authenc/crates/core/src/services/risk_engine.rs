//! Risk Engine Service
//!
//! This module provides risk assessment for authentication requests based on
//! behavioral patterns and failure history.

use async_trait::async_trait;
use std::sync::Arc;
use std::time::Duration;

// Import cache types - these will be available once cache module is migrated
// For now, we'll use a simplified interface
#[async_trait]
pub trait Cache: Send + Sync {
    async fn get(
        &self,
        key: &str,
    ) -> Result<Option<serde_json::Value>, Box<dyn std::error::Error + Send + Sync>>;
    async fn increment(
        &self,
        key: &str,
        delta: i64,
    ) -> Result<i64, Box<dyn std::error::Error + Send + Sync>>;
    async fn expire(
        &self,
        key: &str,
        ttl: Duration,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
    async fn delete(&self, key: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
}

/// Risk Engine for assessing request risk based on behavioral patterns
pub struct RiskEngine {
    cache: Option<Arc<dyn Cache>>,
}

impl RiskEngine {
    /// Create a new Risk Engine instance
    pub fn new(cache: Option<Arc<dyn Cache>>) -> Self {
        Self { cache }
    }

    /// Calculate dynamic difficulty based on IP risk score
    /// Returns a difficulty level from 1 (Easy) to 10 (Expert)
    pub async fn calculate_difficulty(&self, ip: &str, requested_difficulty: Option<u8>) -> u8 {
        let risk_score = self.get_risk_score(ip).await;

        // Base difficulty from risk (0 failures = 1, 5+ failures = 8-10)
        let risk_based_difficulty = match risk_score {
            0..=2 => 1, // Low risk: Easy
            3..=5 => 4, // Medium risk: Medium
            6..=8 => 7, // High risk: Hard
            _ => 9,     // Critical risk: Expert
        };

        // If client requested a difficulty, take the higher of the two (security first)
        // But cap client request at 10 and ensure min is 1
        let client_diff = requested_difficulty.unwrap_or(1).clamp(1, 10);

        std::cmp::max(risk_based_difficulty, client_diff)
    }

    /// Get current consecutive failure count for an IP
    pub async fn get_risk_score(&self, ip: &str) -> u32 {
        if let Some(cache) = &self.cache {
            let key = format!("risk:failures:{}", ip);
            match cache.get(&key).await {
                Ok(Some(val)) => val.as_u64().unwrap_or(0) as u32,
                _ => 0,
            }
        } else {
            // Fallback if no cache (could use in-memory, but for now 0)
            0
        }
    }

    /// Record a failed attempt
    pub async fn record_failure(&self, ip: &str) {
        if let Some(cache) = &self.cache {
            let key = format!("risk:failures:{}", ip);
            // Increment failure count, set TTL to 30 minutes
            // Redis INCR is atomic
            let _ = cache.increment(&key, 1).await;
            let _ = cache.expire(&key, Duration::from_secs(1800)).await;
        }
    }

    /// Record a successful attempt (decrements risk)
    pub async fn record_success(&self, ip: &str) {
        if let Some(cache) = &self.cache {
            let key = format!("risk:failures:{}", ip);
            // Delete the key to reset trust quickly for good behavior
            let _ = cache.delete(&key).await;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_risk_engine_without_cache() {
        let engine = RiskEngine::new(None);

        // Without cache, risk score should always be 0
        assert_eq!(engine.get_risk_score("192.168.1.1").await, 0);

        // Difficulty should be based on requested difficulty only
        assert_eq!(engine.calculate_difficulty("192.168.1.1", None).await, 1);
        assert_eq!(engine.calculate_difficulty("192.168.1.1", Some(5)).await, 5);
    }

    #[tokio::test]
    async fn test_calculate_difficulty_ranges() {
        let engine = RiskEngine::new(None);

        // Test difficulty clamping
        assert_eq!(engine.calculate_difficulty("192.168.1.1", Some(0)).await, 1); // Min is 1
        assert_eq!(
            engine.calculate_difficulty("192.168.1.1", Some(15)).await,
            10
        ); // Max is 10
        assert_eq!(engine.calculate_difficulty("192.168.1.1", Some(5)).await, 5); // Normal range
    }
}
