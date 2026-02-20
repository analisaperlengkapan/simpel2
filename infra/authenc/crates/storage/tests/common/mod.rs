//! Common test utilities for storage layer tests
//!
//! Provides mock database connections and test helpers.

use authenc_types::{AuthencError, Result};
use std::sync::Arc;
use tokio::sync::Mutex;

/// Mock database client for testing
pub struct MockDatabase {
    /// Simulated query results
    pub query_results: Arc<Mutex<Vec<Result<Vec<tokio_postgres::Row>>>>>,
    /// Track executed queries
    pub executed_queries: Arc<Mutex<Vec<String>>>,
}

impl MockDatabase {
    pub fn new() -> Self {
        Self {
            query_results: Arc::new(Mutex::new(Vec::new())),
            executed_queries: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub async fn add_query_result(&self, result: Result<Vec<tokio_postgres::Row>>) {
        self.query_results.lock().await.push(result);
    }

    pub async fn get_executed_queries(&self) -> Vec<String> {
        self.executed_queries.lock().await.clone()
    }

    pub async fn clear(&self) {
        self.query_results.lock().await.clear();
        self.executed_queries.lock().await.clear();
    }
}

impl Default for MockDatabase {
    fn default() -> Self {
        Self::new()
    }
}

/// Test helper to create a UUID from a simple integer
pub fn test_uuid(n: u32) -> uuid::Uuid {
    uuid::Uuid::from_u128(n as u128)
}

/// Test helper to create a timestamp
pub fn test_timestamp() -> chrono::DateTime<chrono::Utc> {
    chrono::Utc::now()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_uuid_helper() {
        let uuid1 = test_uuid(1);
        let uuid2 = test_uuid(2);
        assert_ne!(uuid1, uuid2);
    }

    #[test]
    fn test_timestamp_helper() {
        let ts = test_timestamp();
        assert!(ts <= chrono::Utc::now());
    }

    #[tokio::test]
    async fn test_mock_database_creation() {
        let mock = MockDatabase::new();
        let queries = mock.get_executed_queries().await;
        assert!(queries.is_empty());
    }

    #[tokio::test]
    async fn test_mock_database_clear() {
        let mock = MockDatabase::new();
        mock.executed_queries.lock().await.push("SELECT 1".to_string());
        mock.clear().await;
        let queries = mock.get_executed_queries().await;
        assert!(queries.is_empty());
    }
}
