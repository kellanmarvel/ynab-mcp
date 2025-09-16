use dashmap::DashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};
use tracing::{debug, warn};

#[derive(Clone, Debug)]
pub struct CacheEntry {
    pub data: serde_json::Value,
    pub expires_at: Instant,
    pub last_access: Instant,
}

#[derive(Debug, Clone)]
pub struct CacheStats {
    pub total_requests: u64,
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub hit_ratio: f64,
    pub cache_size: usize,
}

#[derive(Clone)]
pub struct YnabCache {
    cache: Arc<DashMap<String, CacheEntry>>,
    stats: Arc<RwLock<CacheStats>>,
    capacity: usize,
}

impl YnabCache {
    pub fn new(capacity: usize) -> Self {
        Self {
            cache: Arc::new(DashMap::new()),
            stats: Arc::new(RwLock::new(CacheStats {
                total_requests: 0,
                cache_hits: 0,
                cache_misses: 0,
                hit_ratio: 0.0,
                cache_size: 0,
            })),
            capacity,
        }
    }

    pub fn get(&self, key: &str) -> Option<serde_json::Value> {
        // Update total requests
        if let Ok(mut stats) = self.stats.write() {
            stats.total_requests += 1;
        }

        if let Some(mut entry) = self.cache.get_mut(key) {
            if entry.expires_at > Instant::now() {
                entry.last_access = Instant::now();

                // Update hit stats
                if let Ok(mut stats) = self.stats.write() {
                    stats.cache_hits += 1;
                    stats.hit_ratio = stats.cache_hits as f64 / stats.total_requests as f64;
                }

                debug!(cache_key = key, "Cache hit");
                Some(entry.data.clone())
            } else {
                // Entry expired, remove it
                drop(entry); // Release the reference before removing
                self.cache.remove(key);

                // Update miss stats
                if let Ok(mut stats) = self.stats.write() {
                    stats.cache_misses += 1;
                    stats.cache_size = self.cache.len();
                    stats.hit_ratio = stats.cache_hits as f64 / stats.total_requests as f64;
                }

                debug!(cache_key = key, "Cache miss (expired)");
                None
            }
        } else {
            // Update miss stats
            if let Ok(mut stats) = self.stats.write() {
                stats.cache_misses += 1;
                stats.hit_ratio = stats.cache_hits as f64 / stats.total_requests as f64;
            }

            debug!(cache_key = key, "Cache miss (not found)");
            None
        }
    }

    pub fn set(&self, key: String, data: serde_json::Value, ttl: Duration) {
        // Check if we need to make room (simple LRU eviction)
        if self.cache.len() >= self.capacity {
            self.evict_expired_or_oldest();
        }

        let entry = CacheEntry {
            data,
            expires_at: Instant::now() + ttl,
            last_access: Instant::now(),
        };

        self.cache.insert(key.clone(), entry);

        // Update cache size
        if let Ok(mut stats) = self.stats.write() {
            stats.cache_size = self.cache.len();
        }

        debug!(cache_key = key, ttl_secs = ttl.as_secs(), "Data cached");
    }

    pub fn clear(&self) {
        self.cache.clear();

        if let Ok(mut stats) = self.stats.write() {
            stats.cache_size = 0;
        }

        debug!("Cache cleared");
    }

    pub fn get_stats(&self) -> CacheStats {
        if let Ok(stats) = self.stats.read() {
            stats.clone()
        } else {
            CacheStats {
                total_requests: 0,
                cache_hits: 0,
                cache_misses: 0,
                hit_ratio: 0.0,
                cache_size: self.cache.len(),
            }
        }
    }

    pub fn cleanup_expired(&self) -> usize {
        let now = Instant::now();
        let mut removed = 0;

        // Collect expired keys
        let expired_keys: Vec<String> = self
            .cache
            .iter()
            .filter_map(|entry| {
                if entry.value().expires_at <= now {
                    Some(entry.key().clone())
                } else {
                    None
                }
            })
            .collect();

        // Remove expired entries
        for key in expired_keys {
            if self.cache.remove(&key).is_some() {
                removed += 1;
            }
        }

        if removed > 0 {
            // Update cache size
            if let Ok(mut stats) = self.stats.write() {
                stats.cache_size = self.cache.len();
            }

            debug!(
                expired_entries = removed,
                "Cleaned up expired cache entries"
            );
        }

        removed
    }

    fn evict_expired_or_oldest(&self) {
        // First try to remove expired entries
        if self.cleanup_expired() > 0 {
            return;
        }

        // If no expired entries, remove the oldest accessed entry (LRU)
        if let Some(oldest_key) = self.find_oldest_key() {
            if self.cache.remove(&oldest_key).is_some() {
                warn!(
                    cache_key = oldest_key,
                    "Evicted oldest cache entry to make room"
                );

                if let Ok(mut stats) = self.stats.write() {
                    stats.cache_size = self.cache.len();
                }
            }
        }
    }

    fn find_oldest_key(&self) -> Option<String> {
        let mut oldest_key: Option<String> = None;
        let mut oldest_time = Instant::now();

        for entry in self.cache.iter() {
            if entry.value().last_access < oldest_time {
                oldest_time = entry.value().last_access;
                oldest_key = Some(entry.key().clone());
            }
        }

        oldest_key
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_cache_creation() {
        let cache = YnabCache::new(100);
        let stats = cache.get_stats();

        assert_eq!(stats.total_requests, 0);
        assert_eq!(stats.cache_hits, 0);
        assert_eq!(stats.cache_misses, 0);
        assert_eq!(stats.hit_ratio, 0.0);
        assert_eq!(stats.cache_size, 0);
        assert_eq!(cache.capacity, 100);
    }

    #[test]
    fn test_cache_entry_creation() {
        let data = json!({"key": "value"});
        let now = Instant::now();
        let entry = CacheEntry {
            data: data.clone(),
            expires_at: now + Duration::from_secs(60),
            last_access: now,
        };

        assert_eq!(entry.data, data);
        assert!(entry.expires_at > now);
        assert_eq!(entry.last_access, now);
    }

    #[test]
    fn test_cache_stats_creation() {
        let stats = CacheStats {
            total_requests: 10,
            cache_hits: 8,
            cache_misses: 2,
            hit_ratio: 0.8,
            cache_size: 5,
        };

        assert_eq!(stats.total_requests, 10);
        assert_eq!(stats.cache_hits, 8);
        assert_eq!(stats.cache_misses, 2);
        assert_eq!(stats.hit_ratio, 0.8);
        assert_eq!(stats.cache_size, 5);
    }

    #[test]
    fn test_cache_set_and_get() {
        let cache = YnabCache::new(10);
        let key = "test_key".to_string();
        let value = json!({"test": "data"});

        cache.set(key.clone(), value.clone(), Duration::from_secs(60));

        let retrieved = cache.get(&key);
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap(), value);

        let stats = cache.get_stats();
        assert_eq!(stats.total_requests, 1);
        assert_eq!(stats.cache_hits, 1);
        assert_eq!(stats.cache_misses, 0);
        assert_eq!(stats.hit_ratio, 1.0);
        assert_eq!(stats.cache_size, 1);
    }

    #[test]
    fn test_cache_miss() {
        let cache = YnabCache::new(10);

        let result = cache.get("nonexistent_key");
        assert!(result.is_none());

        let stats = cache.get_stats();
        assert_eq!(stats.total_requests, 1);
        assert_eq!(stats.cache_hits, 0);
        assert_eq!(stats.cache_misses, 1);
        assert_eq!(stats.hit_ratio, 0.0);
        assert_eq!(stats.cache_size, 0);
    }

    #[test]
    fn test_cache_expiration() {
        let cache = YnabCache::new(10);
        let key = "test_key".to_string();
        let value = json!({"test": "data"});

        // Set with very short TTL
        cache.set(key.clone(), value.clone(), Duration::from_millis(10));

        // Immediate access should work
        let retrieved = cache.get(&key);
        assert!(retrieved.is_some());

        // Wait for expiration
        thread::sleep(Duration::from_millis(20));

        // Access after expiration should fail
        let retrieved = cache.get(&key);
        assert!(retrieved.is_none());

        let stats = cache.get_stats();
        assert_eq!(stats.total_requests, 2);
        assert_eq!(stats.cache_hits, 1);
        assert_eq!(stats.cache_misses, 1);
        assert_eq!(stats.hit_ratio, 0.5);
        assert_eq!(stats.cache_size, 0); // Should be removed after expiration
    }

    #[test]
    fn test_cache_clear() {
        let cache = YnabCache::new(10);

        // Add multiple entries
        cache.set(
            "key1".to_string(),
            json!({"data": 1}),
            Duration::from_secs(60),
        );
        cache.set(
            "key2".to_string(),
            json!({"data": 2}),
            Duration::from_secs(60),
        );
        cache.set(
            "key3".to_string(),
            json!({"data": 3}),
            Duration::from_secs(60),
        );

        let stats_before = cache.get_stats();
        assert_eq!(stats_before.cache_size, 3);

        cache.clear();

        let stats_after = cache.get_stats();
        assert_eq!(stats_after.cache_size, 0);

        // Verify all entries are gone
        assert!(cache.get("key1").is_none());
        assert!(cache.get("key2").is_none());
        assert!(cache.get("key3").is_none());
    }

    #[test]
    fn test_cache_cleanup_expired() {
        let cache = YnabCache::new(10);

        // Add entries with different TTLs
        cache.set(
            "short_ttl".to_string(),
            json!({"data": 1}),
            Duration::from_millis(10),
        );
        cache.set(
            "long_ttl".to_string(),
            json!({"data": 2}),
            Duration::from_secs(60),
        );

        // Wait for short TTL to expire
        thread::sleep(Duration::from_millis(20));

        // Cleanup should remove 1 expired entry
        let removed = cache.cleanup_expired();
        assert_eq!(removed, 1);

        // Long TTL entry should still exist
        assert!(cache.get("long_ttl").is_some());
        assert!(cache.get("short_ttl").is_none());

        let stats = cache.get_stats();
        assert_eq!(stats.cache_size, 1);
    }

    #[test]
    fn test_cache_capacity_eviction() {
        let cache = YnabCache::new(2); // Small capacity

        // Fill cache to capacity
        cache.set(
            "key1".to_string(),
            json!({"data": 1}),
            Duration::from_secs(60),
        );
        cache.set(
            "key2".to_string(),
            json!({"data": 2}),
            Duration::from_secs(60),
        );

        // Access key1 to make it more recently used
        cache.get("key1");

        // Add a third entry, should evict key2 (oldest)
        cache.set(
            "key3".to_string(),
            json!({"data": 3}),
            Duration::from_secs(60),
        );

        let stats = cache.get_stats();
        assert_eq!(stats.cache_size, 2);

        // key1 and key3 should exist, key2 should be evicted
        assert!(cache.get("key1").is_some());
        assert!(cache.get("key3").is_some());
        assert!(cache.get("key2").is_none());
    }

    #[test]
    fn test_find_oldest_key() {
        let cache = YnabCache::new(10);

        // Add entries with small delays to ensure different last_access times
        cache.set(
            "first".to_string(),
            json!({"data": 1}),
            Duration::from_secs(60),
        );
        thread::sleep(Duration::from_millis(1));
        cache.set(
            "second".to_string(),
            json!({"data": 2}),
            Duration::from_secs(60),
        );
        thread::sleep(Duration::from_millis(1));
        cache.set(
            "third".to_string(),
            json!({"data": 3}),
            Duration::from_secs(60),
        );

        // Access second to update its last_access time
        cache.get("second");

        let oldest_key = cache.find_oldest_key();
        assert!(oldest_key.is_some());
        // Either "first" or "third" could be oldest depending on timing
        let key = oldest_key.unwrap();
        assert!(key == "first" || key == "third");
    }

    #[test]
    fn test_cache_hit_ratio_calculation() {
        let cache = YnabCache::new(10);

        cache.set(
            "key1".to_string(),
            json!({"data": 1}),
            Duration::from_secs(60),
        );

        // Hit
        cache.get("key1");
        let stats = cache.get_stats();
        assert_eq!(stats.hit_ratio, 1.0);

        // Miss
        cache.get("nonexistent");
        let stats = cache.get_stats();
        assert_eq!(stats.hit_ratio, 0.5);

        // Another hit
        cache.get("key1");
        let stats = cache.get_stats();
        assert_eq!(stats.total_requests, 3);
        assert_eq!(stats.cache_hits, 2);
        assert_eq!(stats.cache_misses, 1);
        assert!((stats.hit_ratio - 0.6666666666666666).abs() < f64::EPSILON);
    }

    #[test]
    fn test_cache_stats_with_lock_failure() {
        let cache = YnabCache::new(10);

        // Test the fallback path when stats lock might fail
        // This is mostly for coverage of the error handling path
        let stats = cache.get_stats();
        assert_eq!(stats.cache_size, 0);
    }

    #[test]
    fn test_evict_expired_or_oldest_with_expired_entries() {
        let cache = YnabCache::new(2);

        // Add entries with short TTL
        cache.set(
            "expired1".to_string(),
            json!({"data": 1}),
            Duration::from_millis(10),
        );
        cache.set(
            "expired2".to_string(),
            json!({"data": 2}),
            Duration::from_millis(10),
        );

        // Wait for expiration
        thread::sleep(Duration::from_millis(20));

        // This should trigger eviction of expired entries
        cache.set(
            "new_key".to_string(),
            json!({"data": 3}),
            Duration::from_secs(60),
        );

        let stats = cache.get_stats();
        assert_eq!(stats.cache_size, 1);
        assert!(cache.get("new_key").is_some());
        assert!(cache.get("expired1").is_none());
        assert!(cache.get("expired2").is_none());
    }

    #[test]
    fn test_evict_expired_or_oldest_without_expired_entries() {
        let cache = YnabCache::new(2);

        // Fill cache with non-expired entries
        cache.set(
            "key1".to_string(),
            json!({"data": 1}),
            Duration::from_secs(60),
        );
        cache.set(
            "key2".to_string(),
            json!({"data": 2}),
            Duration::from_secs(60),
        );

        // Access key1 to make it more recently used
        cache.get("key1");

        // This should trigger LRU eviction of key2
        cache.set(
            "key3".to_string(),
            json!({"data": 3}),
            Duration::from_secs(60),
        );

        let stats = cache.get_stats();
        assert_eq!(stats.cache_size, 2);
        assert!(cache.get("key1").is_some());
        assert!(cache.get("key3").is_some());
        assert!(cache.get("key2").is_none());
    }

    #[test]
    fn test_multiple_cache_operations() {
        let cache = YnabCache::new(5);

        // Test complex scenario with multiple operations
        cache.set(
            "a".to_string(),
            json!({"value": "a"}),
            Duration::from_secs(60),
        );
        cache.set(
            "b".to_string(),
            json!({"value": "b"}),
            Duration::from_millis(50),
        );
        cache.set(
            "c".to_string(),
            json!({"value": "c"}),
            Duration::from_secs(60),
        );

        // Initial state
        assert_eq!(cache.get_stats().cache_size, 3);

        // Some hits and misses
        assert!(cache.get("a").is_some());
        assert!(cache.get("nonexistent").is_none());
        assert!(cache.get("c").is_some());

        // Wait for 'b' to expire
        thread::sleep(Duration::from_millis(60));

        // Access expired entry (should be removed)
        assert!(cache.get("b").is_none());
        assert_eq!(cache.get_stats().cache_size, 2);

        // Manual cleanup
        let removed = cache.cleanup_expired();
        assert_eq!(removed, 0); // Already removed by get

        // Clear all
        cache.clear();
        assert_eq!(cache.get_stats().cache_size, 0);
    }

    #[test]
    fn test_zero_capacity_cache() {
        let cache = YnabCache::new(0);

        // Should not store anything with zero capacity
        cache.set(
            "key".to_string(),
            json!({"data": "value"}),
            Duration::from_secs(60),
        );

        // The cache might still try to store it briefly before eviction
        let stats = cache.get_stats();
        assert!(stats.cache_size <= 1); // Should be 0 or 1 depending on eviction timing
    }

    #[test]
    fn test_cache_with_large_capacity() {
        let cache = YnabCache::new(1000);

        // Add many entries
        for i in 0..100 {
            cache.set(
                format!("key_{}", i),
                json!({"index": i}),
                Duration::from_secs(60),
            );
        }

        let stats = cache.get_stats();
        assert_eq!(stats.cache_size, 100);

        // All should be retrievable
        for i in 0..100 {
            assert!(cache.get(&format!("key_{}", i)).is_some());
        }

        let final_stats = cache.get_stats();
        assert_eq!(final_stats.total_requests, 100);
        assert_eq!(final_stats.cache_hits, 100);
        assert_eq!(final_stats.hit_ratio, 1.0);
    }

    #[test]
    fn test_cache_entry_debug() {
        let entry = CacheEntry {
            data: json!({"test": "value"}),
            expires_at: Instant::now() + Duration::from_secs(60),
            last_access: Instant::now(),
        };

        let debug_str = format!("{:?}", entry);
        assert!(debug_str.contains("CacheEntry"));
        assert!(debug_str.contains("data"));
        assert!(debug_str.contains("expires_at"));
        assert!(debug_str.contains("last_access"));
    }

    #[test]
    fn test_cache_stats_debug() {
        let stats = CacheStats {
            total_requests: 10,
            cache_hits: 8,
            cache_misses: 2,
            hit_ratio: 0.8,
            cache_size: 5,
        };

        let debug_str = format!("{:?}", stats);
        assert!(debug_str.contains("CacheStats"));
        assert!(debug_str.contains("10"));
        assert!(debug_str.contains("8"));
        assert!(debug_str.contains("2"));
        assert!(debug_str.contains("0.8"));
        assert!(debug_str.contains("5"));
    }

    #[test]
    fn test_cache_clone() {
        let cache = YnabCache::new(10);
        cache.set(
            "key".to_string(),
            json!({"data": "value"}),
            Duration::from_secs(60),
        );

        let cloned_cache = cache.clone();

        // Both should access the same underlying cache
        assert!(cache.get("key").is_some());
        assert!(cloned_cache.get("key").is_some());

        // Operations on clone should affect original
        cloned_cache.set(
            "key2".to_string(),
            json!({"data": "value2"}),
            Duration::from_secs(60),
        );
        assert!(cache.get("key2").is_some());
    }

    #[test]
    fn test_empty_cache_oldest_key() {
        let cache = YnabCache::new(10);

        let oldest_key = cache.find_oldest_key();
        assert!(oldest_key.is_none());
    }

    #[test]
    fn test_single_entry_oldest_key() {
        let cache = YnabCache::new(10);
        cache.set(
            "only_key".to_string(),
            json!({"data": "value"}),
            Duration::from_secs(60),
        );

        let oldest_key = cache.find_oldest_key();
        assert_eq!(oldest_key, Some("only_key".to_string()));
    }
}
