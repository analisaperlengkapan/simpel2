use std::sync::Arc;
use crate::services::cache::{RedisCache, Cache};
use std::time::Duration;

/// Risk Engine for assessing request risk based on behavioral patterns
pub struct RiskEngine {
    redis_cache: Option<Arc<RedisCache>>,
}

impl RiskEngine {
    /// Create a new Risk Engine instance
    pub fn new(redis_cache: Option<Arc<RedisCache>>) -> Self {
        Self { redis_cache }
    }

    /// Calculate dynamic difficulty based on IP risk score
    /// Returns a difficulty level from 1 (Easy) to 10 (Expert)
    pub async fn calculate_difficulty(&self, ip: &str, requested_difficulty: Option<u8>) -> u8 {
        let risk_score = self.get_risk_score(ip).await;

        // Base difficulty from risk (0 failures = 1, 5+ failures = 8-10)
        let risk_based_difficulty = match risk_score {
            0..=2 => 1,  // Low risk: Easy
            3..=5 => 4,  // Medium risk: Medium
            6..=8 => 7,  // High risk: Hard
            _ => 9,      // Critical risk: Expert
        };

        // If client requested a difficulty, take the higher of the two (security first)
        // But cap client request at 10 and ensure min is 1
        let client_diff = requested_difficulty.unwrap_or(1).clamp(1, 10);

        std::cmp::max(risk_based_difficulty, client_diff)
    }

    /// Get current consecutive failure count for an IP
    pub async fn get_risk_score(&self, ip: &str) -> u32 {
        if let Some(cache) = &self.redis_cache {
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
        if let Some(cache) = &self.redis_cache {
            let key = format!("risk:failures:{}", ip);
            // Increment failure count, set TTL to 30 minutes
            // Redis INCR is atomic
            let _ = cache.increment(&key, 1).await;
            let _ = cache.expire(&key, Duration::from_secs(1800)).await;
        }
    }

    /// Record a successful attempt (decrements risk)
    pub async fn record_success(&self, ip: &str) {
        if let Some(cache) = &self.redis_cache {
            let key = format!("risk:failures:{}", ip);
            // Delete the key or decrement? Let's delete to reset trust quickly for good behavior
            // Or maybe just decrement? Let's delete for "verification success resets risk"
            let _ = cache.delete(&key).await;
        }
    }
}
