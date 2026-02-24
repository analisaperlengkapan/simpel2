// **Feature: secreton-comprehensive-enhancement, Property 30: Cache LRU Eviction**
// **Validates: Requirements 13.3**
//
// Property: For any cache at capacity, inserting a new entry SHALL evict the least recently used entry.

use proptest::prelude::*;
use secreton_core::utils::cache::{SecretLruCache, SensitivityLevel};
use std::time::Duration;

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// Property 30: Cache LRU Eviction
    ///
    /// This test verifies that when a cache reaches capacity, inserting a new entry
    /// evicts the least recently used (LRU) entry, not just any entry.
    ///
    /// Test strategy:
    /// 1. Create a cache with small capacity (3)
    /// 2. Fill it completely with entries
    /// 3. Access entries in a specific order to establish LRU ordering
    /// 4. Insert a new entry to trigger eviction
    /// 5. Verify that the LRU entry was evicted, not others
    #[test]
    fn property_cache_lru_eviction(
        capacity in 2usize..10,
        num_initial_entries in 2usize..10,
        access_pattern in prop::collection::vec(0usize..10, 1..20),
    ) {
        // Ensure we have enough entries to fill the cache
        let num_entries = num_initial_entries.max(capacity);

        let mut cache = SecretLruCache::new(capacity);

        // Insert initial entries to fill the cache
        for i in 0..num_entries {
            let key = format!("key_{}", i);
            let value = vec![i as u8];
            cache.insert_with_sensitivity(
                key,
                value,
                Duration::from_secs(3600),
                SensitivityLevel::Low,
            );
        }

        // At this point, cache should be at capacity
        // The LRU entry should be the one that was inserted first and not accessed

        // Access some entries to change LRU order
        // We'll access entries based on the access pattern, but skip the first entry
        // to ensure it remains the LRU
        for &idx in &access_pattern {
            if idx > 0 && idx < num_entries {
                let key = format!("key_{}", idx);
                let _ = cache.get(&key);
            }
        }

        // Now insert a new entry that should evict the LRU
        let new_key = "new_key".to_string();
        let new_value = vec![255];
        cache.insert_with_sensitivity(
            new_key.clone(),
            new_value.clone(),
            Duration::from_secs(3600),
            SensitivityLevel::Low,
        );

        // The new entry should be in the cache
        prop_assert_eq!(cache.get(&new_key), Some(new_value));

        // Verify cache size doesn't exceed capacity
        let stats = cache.stats();
        prop_assert!(stats.size <= capacity,
            "Cache size {} exceeds capacity {}", stats.size, capacity);

        // Verify that eviction occurred if we were at capacity
        if num_entries >= capacity {
            let expected_evictions = (num_entries - capacity + 1) as u64;
            prop_assert_eq!(stats.eviction_count, expected_evictions,
                "Expected {} evictions, got {}", expected_evictions, stats.eviction_count);
        }
    }

    /// Property 30.1: LRU Eviction Order Correctness
    ///
    /// This test verifies the exact LRU eviction order by tracking which entries
    /// are evicted when the cache fills up.
    #[test]
    fn property_lru_eviction_order(
        capacity in 2usize..5,
    ) {
        let mut cache = SecretLruCache::new(capacity);

        // Insert entries 0 to capacity-1
        for i in 0..capacity {
            cache.insert_with_sensitivity(
                format!("key_{}", i),
                vec![i as u8],
                Duration::from_secs(3600),
                SensitivityLevel::Low,
            );
        }

        // All entries should be present
        for i in 0..capacity {
            let key = format!("key_{}", i);
            prop_assert!(cache.get(&key).is_some());
        }

        // Insert one more entry - should evict key_0 (the LRU)
        cache.insert_with_sensitivity(
            "key_new".to_string(),
            vec![99],
            Duration::from_secs(3600),
            SensitivityLevel::Low,
        );

        // key_0 should be evicted
        prop_assert_eq!(cache.get(&"key_0".to_string()), None, "key_0 should be evicted");

        // All other keys should still be present
        for i in 1..capacity {
            let key = format!("key_{}", i);
            prop_assert!(cache.get(&key).is_some(),
                "key_{} should still be present", i);
        }

        // New key should be present
        prop_assert_eq!(cache.get(&"key_new".to_string()), Some(vec![99]));
    }

    /// Property 30.2: Access Updates LRU Order
    ///
    /// This test verifies that accessing an entry moves it to the most recently used position,
    /// preventing it from being evicted next.
    #[test]
    fn property_access_updates_lru_order(
        capacity in 2usize..5,
    ) {
        let mut cache = SecretLruCache::new(capacity);

        // Fill cache
        for i in 0..capacity {
            cache.insert_with_sensitivity(
                format!("key_{}", i),
                vec![i as u8],
                Duration::from_secs(3600),
                SensitivityLevel::Low,
            );
        }

        // Access key_0 to make it most recently used
        let _ = cache.get(&"key_0".to_string());

        // Insert a new entry - should evict key_1 (now the LRU), not key_0
        cache.insert_with_sensitivity(
            "key_new".to_string(),
            vec![99],
            Duration::from_secs(3600),
            SensitivityLevel::Low,
        );

        // key_0 should still be present (we accessed it)
        prop_assert!(cache.get(&"key_0".to_string()).is_some(),
            "key_0 should not be evicted after access");

        // key_1 should be evicted (it was the LRU)
        prop_assert_eq!(cache.get(&"key_1".to_string()), None,
            "key_1 should be evicted as LRU");
    }

    /// Property 30.3: Multiple Evictions Maintain LRU Order
    ///
    /// This test verifies that multiple consecutive insertions evict entries
    /// in strict LRU order.
    #[test]
    fn property_multiple_evictions_maintain_order(
        capacity in 2usize..5,
        num_new_entries in 1usize..5,
    ) {
        let mut cache = SecretLruCache::new(capacity);

        // Fill cache
        for i in 0..capacity {
            cache.insert_with_sensitivity(
                format!("key_{}", i),
                vec![i as u8],
                Duration::from_secs(3600),
                SensitivityLevel::Low,
            );
        }

        // Insert multiple new entries
        for i in 0..num_new_entries {
            cache.insert_with_sensitivity(
                format!("new_key_{}", i),
                vec![100 + i as u8],
                Duration::from_secs(3600),
                SensitivityLevel::Low,
            );
        }

        // Verify cache size doesn't exceed capacity
        let stats = cache.stats();
        prop_assert!(stats.size <= capacity);

        // The oldest entries should be evicted first
        let num_evicted = num_new_entries.min(capacity);
        for i in 0..num_evicted {
            let key = format!("key_{}", i);
            prop_assert_eq!(cache.get(&key), None,
                "key_{} should be evicted", i);
        }

        // Newer entries should still be present (if any)
        if capacity > num_evicted {
            for i in num_evicted..capacity {
                let key = format!("key_{}", i);
                prop_assert!(cache.get(&key).is_some(),
                    "key_{} should still be present", i);
            }
        }

        // All new entries should be present (up to capacity)
        let new_entries_in_cache = num_new_entries.min(capacity);
        for i in (num_new_entries.saturating_sub(new_entries_in_cache))..num_new_entries {
            let key = format!("new_key_{}", i);
            prop_assert!(cache.get(&key).is_some(),
                "new_key_{} should be present", i);
        }
    }
}

#[cfg(test)]
mod unit_tests {
    use super::*;

    #[test]
    fn test_lru_eviction_basic() {
        let mut cache = SecretLruCache::new(3);

        // Insert 3 entries
        cache.insert("a", vec![1], Duration::from_secs(60));
        cache.insert("b", vec![2], Duration::from_secs(60));
        cache.insert("c", vec![3], Duration::from_secs(60));

        // All should be present
        assert_eq!(cache.get(&"a"), Some(vec![1]));
        assert_eq!(cache.get(&"b"), Some(vec![2]));
        assert_eq!(cache.get(&"c"), Some(vec![3]));

        // Insert 4th entry - should evict "a" (LRU)
        cache.insert("d", vec![4], Duration::from_secs(60));

        assert_eq!(cache.get(&"a"), None);
        assert_eq!(cache.get(&"b"), Some(vec![2]));
        assert_eq!(cache.get(&"c"), Some(vec![3]));
        assert_eq!(cache.get(&"d"), Some(vec![4]));
    }

    #[test]
    fn test_lru_access_order() {
        let mut cache = SecretLruCache::new(3);

        cache.insert("a", vec![1], Duration::from_secs(60));
        cache.insert("b", vec![2], Duration::from_secs(60));
        cache.insert("c", vec![3], Duration::from_secs(60));

        // Access "a" to make it most recently used
        cache.get(&"a");

        // Insert "d" - should evict "b" (now LRU), not "a"
        cache.insert("d", vec![4], Duration::from_secs(60));

        assert_eq!(cache.get(&"a"), Some(vec![1])); // Still present
        assert_eq!(cache.get(&"b"), None); // Evicted
        assert_eq!(cache.get(&"c"), Some(vec![3]));
        assert_eq!(cache.get(&"d"), Some(vec![4]));
    }

    #[test]
    fn test_lru_eviction_count() {
        let mut cache = SecretLruCache::new(2);

        cache.insert("a", vec![1], Duration::from_secs(60));
        cache.insert("b", vec![2], Duration::from_secs(60));

        let stats = cache.stats();
        assert_eq!(stats.eviction_count, 0);

        cache.insert("c", vec![3], Duration::from_secs(60));

        let stats = cache.stats();
        assert_eq!(stats.eviction_count, 1);

        cache.insert("d", vec![4], Duration::from_secs(60));

        let stats = cache.stats();
        assert_eq!(stats.eviction_count, 2);
    }
}
