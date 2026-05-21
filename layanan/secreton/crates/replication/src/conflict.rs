//! Replication conflict detection and resolution
//!
//! This module implements conflict detection and resolution for replication scenarios.
//! When a failover occurs, there may be conflicting writes between the old primary
//! and the new primary. This module detects such conflicts and resolves them using
//! a last-write-wins strategy based on timestamps.
//!
//! **Validates: Requirements 2.2.6, 2.2.8**

use crate::{Result, error::ReplicationError, operation::ReplicationOperation};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// Represents a conflict between two operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conflict {
    /// The path or identifier where the conflict occurred
    pub path: String,

    /// The operation from the old primary
    pub old_operation: ReplicationOperation,

    /// The operation from the new primary
    pub new_operation: ReplicationOperation,

    /// The operation that won the conflict resolution
    pub winner: ReplicationOperation,

    /// Timestamp when the conflict was detected
    pub detected_at: DateTime<Utc>,

    /// Reason for the resolution decision
    pub resolution_reason: String,
}

/// Strategy for resolving conflicts
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResolutionStrategy {
    /// Last write wins based on timestamp
    LastWriteWins,

    /// Highest sequence number wins
    HighestSequence,

    /// Manual resolution required
    Manual,
}

/// Conflict resolver that detects and resolves replication conflicts
pub struct ConflictResolver {
    /// Resolution strategy to use
    strategy: ResolutionStrategy,

    /// History of resolved conflicts for audit
    conflict_history: Arc<RwLock<Vec<Conflict>>>,

    /// Maximum number of conflicts to keep in history
    max_history_size: usize,
}

impl ConflictResolver {
    /// Create a new conflict resolver with the specified strategy
    ///
    /// # Arguments
    ///
    /// * `strategy` - The resolution strategy to use
    /// * `max_history_size` - Maximum number of conflicts to keep in history (default: 1000)
    ///
    /// # Example
    ///
    /// ```
    /// use secreton_replication::conflict::{ConflictResolver, ResolutionStrategy};
    ///
    /// let resolver = ConflictResolver::new(ResolutionStrategy::LastWriteWins, 1000);
    /// ```
    pub fn new(strategy: ResolutionStrategy, max_history_size: usize) -> Self {
        Self {
            strategy,
            conflict_history: Arc::new(RwLock::new(Vec::new())),
            max_history_size,
        }
    }

    /// Detect if two operations conflict
    ///
    /// Operations conflict if they:
    /// 1. Target the same path/resource
    /// 2. Have different sequence numbers
    /// 3. Have different timestamps
    ///
    /// # Arguments
    ///
    /// * `op1` - First operation
    /// * `op2` - Second operation
    ///
    /// # Returns
    ///
    /// `true` if the operations conflict, `false` otherwise
    pub fn detect_conflict(&self, op1: &ReplicationOperation, op2: &ReplicationOperation) -> bool {
        // Operations must target the same path
        let path1 = self.extract_path(op1);
        let path2 = self.extract_path(op2);

        if path1 != path2 {
            return false;
        }

        // Operations must have different IDs
        if op1.id == op2.id {
            return false;
        }

        // Operations must have different sequence numbers or timestamps
        if op1.sequence != op2.sequence || op1.timestamp != op2.timestamp {
            debug!(
                "Conflict detected at path '{}': op1(seq={}, ts={}) vs op2(seq={}, ts={})",
                path1, op1.sequence, op1.timestamp, op2.sequence, op2.timestamp
            );
            return true;
        }

        false
    }

    /// Resolve a conflict between two operations
    ///
    /// Uses the configured resolution strategy to determine which operation wins.
    ///
    /// # Arguments
    ///
    /// * `op1` - First operation (typically from old primary)
    /// * `op2` - Second operation (typically from new primary)
    ///
    /// # Returns
    ///
    /// The winning operation and a conflict record for audit
    pub async fn resolve_conflict(
        &self,
        op1: ReplicationOperation,
        op2: ReplicationOperation,
    ) -> Result<(ReplicationOperation, Conflict)> {
        let path = self.extract_path(&op1);

        info!(
            "Resolving conflict at path '{}' using strategy {:?}",
            path, self.strategy
        );

        let (winner, reason) = match self.strategy {
            ResolutionStrategy::LastWriteWins => self.resolve_last_write_wins(&op1, &op2),
            ResolutionStrategy::HighestSequence => self.resolve_highest_sequence(&op1, &op2),
            ResolutionStrategy::Manual => {
                return Err(ReplicationError::internal(
                    "Manual conflict resolution not yet implemented",
                ));
            }
        };

        let conflict = Conflict {
            path,
            old_operation: op1,
            new_operation: op2,
            winner: winner.clone(),
            detected_at: Utc::now(),
            resolution_reason: reason,
        };

        // Add to history
        self.add_to_history(conflict.clone()).await;

        Ok((winner, conflict))
    }

    /// Resolve conflict using last-write-wins strategy
    ///
    /// The operation with the most recent timestamp wins
    fn resolve_last_write_wins(
        &self,
        op1: &ReplicationOperation,
        op2: &ReplicationOperation,
    ) -> (ReplicationOperation, String) {
        if op2.timestamp > op1.timestamp {
            (
                op2.clone(),
                format!(
                    "Last-write-wins: op2 timestamp ({}) > op1 timestamp ({})",
                    op2.timestamp, op1.timestamp
                ),
            )
        } else if op1.timestamp > op2.timestamp {
            (
                op1.clone(),
                format!(
                    "Last-write-wins: op1 timestamp ({}) > op2 timestamp ({})",
                    op1.timestamp, op2.timestamp
                ),
            )
        } else {
            // Timestamps are equal, use sequence number as tiebreaker
            if op2.sequence > op1.sequence {
                (
                    op2.clone(),
                    format!(
                        "Last-write-wins (tiebreaker): op2 sequence ({}) > op1 sequence ({})",
                        op2.sequence, op1.sequence
                    ),
                )
            } else {
                (
                    op1.clone(),
                    format!(
                        "Last-write-wins (tiebreaker): op1 sequence ({}) >= op2 sequence ({})",
                        op1.sequence, op2.sequence
                    ),
                )
            }
        }
    }

    /// Resolve conflict using highest-sequence strategy
    ///
    /// The operation with the highest sequence number wins
    fn resolve_highest_sequence(
        &self,
        op1: &ReplicationOperation,
        op2: &ReplicationOperation,
    ) -> (ReplicationOperation, String) {
        if op2.sequence > op1.sequence {
            (
                op2.clone(),
                format!(
                    "Highest-sequence: op2 sequence ({}) > op1 sequence ({})",
                    op2.sequence, op1.sequence
                ),
            )
        } else {
            (
                op1.clone(),
                format!(
                    "Highest-sequence: op1 sequence ({}) >= op2 sequence ({})",
                    op1.sequence, op2.sequence
                ),
            )
        }
    }

    /// Extract the path from an operation
    fn extract_path(&self, op: &ReplicationOperation) -> String {
        use crate::operation::OperationData;

        match &op.data {
            OperationData::Secret { path, .. } => path.clone(),
            OperationData::SecretDeletion { path } => path.clone(),
            OperationData::Policy { name, .. } => format!("policy/{}", name),
            OperationData::PolicyDeletion { name } => format!("policy/{}", name),
            OperationData::Config { key, .. } => format!("config/{}", key),
            OperationData::Token { token_id, .. } => format!("token/{}", token_id),
            OperationData::TokenRevocation { token_id } => format!("token/{}", token_id),
            OperationData::Lease { lease_id, .. } => format!("lease/{}", lease_id),
            OperationData::LeaseRenewal { lease_id, .. } => format!("lease/{}", lease_id),
            OperationData::LeaseRevocation { lease_id } => format!("lease/{}", lease_id),
            OperationData::Audit { .. } => "audit".to_string(),
        }
    }

    /// Add a conflict to the history
    async fn add_to_history(&self, conflict: Conflict) {
        let mut history = self.conflict_history.write().await;

        history.push(conflict);

        // Trim history if it exceeds max size
        if history.len() > self.max_history_size {
            let excess = history.len() - self.max_history_size;
            history.drain(0..excess);

            debug!("Trimmed {} old conflicts from history", excess);
        }
    }

    /// Get the conflict history
    ///
    /// Returns a copy of all conflicts in the history
    pub async fn get_history(&self) -> Vec<Conflict> {
        self.conflict_history.read().await.clone()
    }

    /// Get conflicts for a specific path
    ///
    /// # Arguments
    ///
    /// * `path` - The path to filter by
    ///
    /// # Returns
    ///
    /// All conflicts that occurred at the specified path
    pub async fn get_conflicts_for_path(&self, path: &str) -> Vec<Conflict> {
        self.conflict_history
            .read()
            .await
            .iter()
            .filter(|c| c.path == path)
            .cloned()
            .collect()
    }

    /// Get the total number of conflicts resolved
    pub async fn conflict_count(&self) -> usize {
        self.conflict_history.read().await.len()
    }

    /// Clear the conflict history
    ///
    /// This is useful for testing or after manual review
    pub async fn clear_history(&self) {
        self.conflict_history.write().await.clear();
        info!("Conflict history cleared");
    }

    /// Batch resolve conflicts for multiple operations
    ///
    /// This is useful when reconciling state after a failover
    ///
    /// # Arguments
    ///
    /// * `operations` - List of operations to check for conflicts
    ///
    /// # Returns
    ///
    /// A map of paths to the winning operations
    pub async fn batch_resolve(
        &self,
        operations: Vec<ReplicationOperation>,
    ) -> Result<HashMap<String, ReplicationOperation>> {
        let mut result = HashMap::new();
        let mut path_ops: HashMap<String, Vec<ReplicationOperation>> = HashMap::new();

        // Group operations by path
        for op in operations {
            let path = self.extract_path(&op);
            path_ops.entry(path).or_default().push(op);
        }

        // Resolve conflicts for each path
        for (path, ops) in path_ops {
            if ops.len() == 1 {
                // No conflict
                result.insert(path, ops.into_iter().next().unwrap());
            } else {
                // Multiple operations for same path - resolve conflicts
                let mut winner = ops[0].clone();

                for op in ops.iter().skip(1) {
                    if self.detect_conflict(&winner, op) {
                        let (new_winner, conflict) =
                            self.resolve_conflict(winner, op.clone()).await?;
                        winner = new_winner;

                        warn!(
                            "Conflict resolved at path '{}': {}",
                            conflict.path, conflict.resolution_reason
                        );
                    }
                }

                result.insert(path, winner);
            }
        }

        Ok(result)
    }
}

impl Default for ConflictResolver {
    fn default() -> Self {
        Self::new(ResolutionStrategy::LastWriteWins, 1000)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::operation::{OperationData, OperationType};
    use chrono::Duration;

    fn create_test_operation(
        path: &str,
        sequence: u64,
        timestamp: DateTime<Utc>,
    ) -> ReplicationOperation {
        ReplicationOperation {
            id: uuid::Uuid::new_v4(),
            timestamp,
            operation_type: OperationType::SecretWrite,
            data: OperationData::Secret {
                path: path.to_string(),
                data: vec![1, 2, 3],
                metadata: serde_json::json!({}),
            },
            sequence,
        }
    }

    #[test]
    fn test_conflict_resolver_creation() {
        let resolver = ConflictResolver::new(ResolutionStrategy::LastWriteWins, 100);
        assert_eq!(resolver.strategy, ResolutionStrategy::LastWriteWins);
        assert_eq!(resolver.max_history_size, 100);
    }

    #[test]
    fn test_detect_conflict_same_path_different_sequence() {
        let resolver = ConflictResolver::default();

        let op1 = create_test_operation("/secret/test", 1, Utc::now());
        let op2 = create_test_operation("/secret/test", 2, Utc::now());

        assert!(resolver.detect_conflict(&op1, &op2));
    }

    #[test]
    fn test_detect_conflict_different_paths() {
        let resolver = ConflictResolver::default();

        let op1 = create_test_operation("/secret/test1", 1, Utc::now());
        let op2 = create_test_operation("/secret/test2", 2, Utc::now());

        assert!(!resolver.detect_conflict(&op1, &op2));
    }

    #[test]
    fn test_detect_conflict_same_operation() {
        let resolver = ConflictResolver::default();

        let op1 = create_test_operation("/secret/test", 1, Utc::now());
        let op2 = op1.clone();

        assert!(!resolver.detect_conflict(&op1, &op2));
    }

    #[tokio::test]
    async fn test_resolve_last_write_wins_newer_timestamp() {
        let resolver = ConflictResolver::new(ResolutionStrategy::LastWriteWins, 100);

        let now = Utc::now();
        let op1 = create_test_operation("/secret/test", 1, now);
        let op2 = create_test_operation("/secret/test", 2, now + Duration::seconds(1));

        let (winner, conflict) = resolver
            .resolve_conflict(op1.clone(), op2.clone())
            .await
            .unwrap();

        assert_eq!(winner.id, op2.id);
        assert_eq!(conflict.winner.id, op2.id);
        assert!(conflict.resolution_reason.contains("Last-write-wins"));
    }

    #[tokio::test]
    async fn test_resolve_last_write_wins_older_timestamp() {
        let resolver = ConflictResolver::new(ResolutionStrategy::LastWriteWins, 100);

        let now = Utc::now();
        let op1 = create_test_operation("/secret/test", 1, now + Duration::seconds(1));
        let op2 = create_test_operation("/secret/test", 2, now);

        let (winner, conflict) = resolver
            .resolve_conflict(op1.clone(), op2.clone())
            .await
            .unwrap();

        assert_eq!(winner.id, op1.id);
        assert_eq!(conflict.winner.id, op1.id);
    }

    #[tokio::test]
    async fn test_resolve_last_write_wins_equal_timestamp() {
        let resolver = ConflictResolver::new(ResolutionStrategy::LastWriteWins, 100);

        let now = Utc::now();
        let op1 = create_test_operation("/secret/test", 1, now);
        let op2 = create_test_operation("/secret/test", 2, now);

        let (winner, conflict) = resolver
            .resolve_conflict(op1.clone(), op2.clone())
            .await
            .unwrap();

        // Higher sequence should win as tiebreaker
        assert_eq!(winner.id, op2.id);
        assert!(conflict.resolution_reason.contains("tiebreaker"));
    }

    #[tokio::test]
    async fn test_resolve_highest_sequence() {
        let resolver = ConflictResolver::new(ResolutionStrategy::HighestSequence, 100);

        let now = Utc::now();
        let op1 = create_test_operation("/secret/test", 1, now + Duration::seconds(10));
        let op2 = create_test_operation("/secret/test", 2, now);

        let (winner, conflict) = resolver
            .resolve_conflict(op1.clone(), op2.clone())
            .await
            .unwrap();

        // Higher sequence should win even though timestamp is older
        assert_eq!(winner.id, op2.id);
        assert!(conflict.resolution_reason.contains("Highest-sequence"));
    }

    #[tokio::test]
    async fn test_conflict_history() {
        let resolver = ConflictResolver::new(ResolutionStrategy::LastWriteWins, 100);

        let now = Utc::now();
        let op1 = create_test_operation("/secret/test", 1, now);
        let op2 = create_test_operation("/secret/test", 2, now + Duration::seconds(1));

        resolver.resolve_conflict(op1, op2).await.unwrap();

        let history = resolver.get_history().await;
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].path, "/secret/test");
    }

    #[tokio::test]
    async fn test_get_conflicts_for_path() {
        let resolver = ConflictResolver::new(ResolutionStrategy::LastWriteWins, 100);

        let now = Utc::now();

        // Create conflicts for different paths
        let op1 = create_test_operation("/secret/test1", 1, now);
        let op2 = create_test_operation("/secret/test1", 2, now + Duration::seconds(1));
        resolver.resolve_conflict(op1, op2).await.unwrap();

        let op3 = create_test_operation("/secret/test2", 1, now);
        let op4 = create_test_operation("/secret/test2", 2, now + Duration::seconds(1));
        resolver.resolve_conflict(op3, op4).await.unwrap();

        let conflicts = resolver.get_conflicts_for_path("/secret/test1").await;
        assert_eq!(conflicts.len(), 1);
        assert_eq!(conflicts[0].path, "/secret/test1");
    }

    #[tokio::test]
    async fn test_history_size_limit() {
        let resolver = ConflictResolver::new(ResolutionStrategy::LastWriteWins, 5);

        let now = Utc::now();

        // Create more conflicts than the limit
        for i in 0..10 {
            let op1 = create_test_operation(&format!("/secret/test{}", i), 1, now);
            let op2 =
                create_test_operation(&format!("/secret/test{}", i), 2, now + Duration::seconds(1));
            resolver.resolve_conflict(op1, op2).await.unwrap();
        }

        let history = resolver.get_history().await;
        assert_eq!(history.len(), 5); // Should be trimmed to max size
    }

    #[tokio::test]
    async fn test_clear_history() {
        let resolver = ConflictResolver::new(ResolutionStrategy::LastWriteWins, 100);

        let now = Utc::now();
        let op1 = create_test_operation("/secret/test", 1, now);
        let op2 = create_test_operation("/secret/test", 2, now + Duration::seconds(1));

        resolver.resolve_conflict(op1, op2).await.unwrap();
        assert_eq!(resolver.conflict_count().await, 1);

        resolver.clear_history().await;
        assert_eq!(resolver.conflict_count().await, 0);
    }

    #[tokio::test]
    async fn test_batch_resolve_no_conflicts() {
        let resolver = ConflictResolver::new(ResolutionStrategy::LastWriteWins, 100);

        let now = Utc::now();
        let operations = vec![
            create_test_operation("/secret/test1", 1, now),
            create_test_operation("/secret/test2", 2, now),
            create_test_operation("/secret/test3", 3, now),
        ];

        let result = resolver.batch_resolve(operations).await.unwrap();

        assert_eq!(result.len(), 3);
        assert!(result.contains_key("/secret/test1"));
        assert!(result.contains_key("/secret/test2"));
        assert!(result.contains_key("/secret/test3"));
    }

    #[tokio::test]
    async fn test_batch_resolve_with_conflicts() {
        let resolver = ConflictResolver::new(ResolutionStrategy::LastWriteWins, 100);

        let now = Utc::now();
        let operations = vec![
            create_test_operation("/secret/test", 1, now),
            create_test_operation("/secret/test", 2, now + Duration::seconds(1)),
            create_test_operation("/secret/test", 3, now + Duration::seconds(2)),
        ];

        let result = resolver.batch_resolve(operations.clone()).await.unwrap();

        assert_eq!(result.len(), 1);
        assert!(result.contains_key("/secret/test"));

        // The operation with the latest timestamp should win
        let winner = result.get("/secret/test").unwrap();
        assert_eq!(winner.sequence, 3);

        // Should have 2 conflicts in history (3 ops = 2 conflicts)
        assert_eq!(resolver.conflict_count().await, 2);
    }

    #[test]
    fn test_extract_path_from_different_operations() {
        let resolver = ConflictResolver::default();

        // Test secret operation
        let secret_op = ReplicationOperation {
            id: uuid::Uuid::new_v4(),
            timestamp: Utc::now(),
            operation_type: OperationType::SecretWrite,
            data: OperationData::Secret {
                path: "/secret/test".to_string(),
                data: vec![],
                metadata: serde_json::json!({}),
            },
            sequence: 1,
        };
        assert_eq!(resolver.extract_path(&secret_op), "/secret/test");

        // Test policy operation
        let policy_op = ReplicationOperation {
            id: uuid::Uuid::new_v4(),
            timestamp: Utc::now(),
            operation_type: OperationType::PolicyWrite,
            data: OperationData::Policy {
                name: "admin".to_string(),
                policy: "{}".to_string(),
            },
            sequence: 2,
        };
        assert_eq!(resolver.extract_path(&policy_op), "policy/admin");
    }
}
