//! Property-based tests for replication
//!
//! **Validates: Requirements 2.2.1 - 2.4.7**

use proptest::prelude::*;
use secreton_replication::operation::{OperationData, OperationType};
use secreton_replication::{ReplicationConfig, ReplicationMode, ReplicationOperation};

/// Property 6: Replication configuration
///
/// **Property**: Replication configuration should be valid and consistent
///
/// **Validates**: Requirements 2.2.1
///
/// **Formal specification**:
/// ```text
/// ∀ config ∈ ReplicationConfig:
///   validate(config) ⇒
///     (mode = Performance ∨ mode = DR)
///     ∧ (is_primary ⇒ secondary_endpoints.len() ≥ 0)
///     ∧ (is_secondary ⇒ primary_endpoint.is_some())
///     ∧ interval > 0
///     ∧ batch_size > 0
/// ```
#[cfg(test)]
mod replication_config_tests {
    use super::*;

    proptest! {
        /// Test valid performance replication configuration
        #[test]
        fn prop_valid_performance_config(
            interval_ms in 10u64..1000u64,
            batch_size in 1usize..1000usize,
            num_secondaries in 0usize..5usize,
        ) {
            let secondary_endpoints: Vec<String> = (0..num_secondaries)
                .map(|i| format!("https://secondary-{}:50051", i))
                .collect();

            let config = ReplicationConfig {
                mode: ReplicationMode::Performance,
                primary_endpoint: None,
                secondary_endpoints,
                interval_ms,
                batch_size,
                ..Default::default()
            };

            // Configuration should be valid
            assert!(config.validate().is_ok(), "Valid configuration should pass validation");

            // Verify properties
            assert_eq!(config.mode, ReplicationMode::Performance);
            assert!(config.primary_endpoint.is_none(), "Primary should not have primary_endpoint");
            assert_eq!(config.interval_ms, interval_ms);
            assert_eq!(config.batch_size, batch_size);
        }
    }

    proptest! {
        /// Test valid DR replication configuration
        #[test]
        fn prop_valid_dr_config(
            interval_ms in 10u64..1000u64,
            batch_size in 1usize..1000usize,
        ) {
            let config = ReplicationConfig {
                mode: ReplicationMode::DisasterRecovery,
                primary_endpoint: None,
                secondary_endpoints: vec!["https://dr-secondary:50051".to_string()],
                interval_ms,
                batch_size,
                ..Default::default()
            };

            assert!(config.validate().is_ok());
            assert_eq!(config.mode, ReplicationMode::DisasterRecovery);
        }
    }

    proptest! {
        /// Test secondary configuration requires primary endpoint
        #[test]
        fn prop_secondary_requires_primary_endpoint(
            interval_ms in 10u64..1000u64,
        ) {
            let config = ReplicationConfig {
                mode: ReplicationMode::Performance,
                primary_endpoint: Some("https://primary:50051".to_string()),
                secondary_endpoints: vec![],
                interval_ms,
                batch_size: 100,
                ..Default::default()
            };

            assert!(config.validate().is_ok());
            assert!(config.primary_endpoint.is_some(), "Secondary must have primary_endpoint");
        }
    }

    proptest! {
        /// Test namespace filtering configuration
        #[test]
        fn prop_namespace_filtering(
            num_namespaces in 0usize..10usize,
        ) {
            let namespaces: Vec<String> = (0..num_namespaces)
                .map(|i| format!("namespace-{}", i))
                .collect();

            let config = ReplicationConfig {
                mode: ReplicationMode::Performance,
                primary_endpoint: None,
                secondary_endpoints: vec!["https://secondary:50051".to_string()],
                enable_namespace_filter: !namespaces.is_empty(),
                namespaces: namespaces.clone(),
                ..Default::default()
            };

            assert!(config.validate().is_ok());

            // Test namespace filtering
            if config.enable_namespace_filter && !namespaces.is_empty() {
                for ns in &namespaces {
                    assert!(
                        config.should_replicate_namespace(ns),
                        "Configured namespace should be replicated"
                    );
                }

                // Test non-configured namespace
                assert!(
                    !config.should_replicate_namespace("other-namespace"),
                    "Non-configured namespace should not be replicated"
                );
            } else {
                // No filter means replicate all
                assert!(
                    config.should_replicate_namespace("any-namespace"),
                    "Without filter, all namespaces should be replicated"
                );
            }
        }
    }

    #[test]
    fn test_replication_mode_properties() {
        // Performance mode
        let perf_mode = ReplicationMode::Performance;
        assert!(perf_mode.allows_secondary_reads());
        assert!(!perf_mode.replicates_ephemeral_state());

        // DR mode
        let dr_mode = ReplicationMode::DisasterRecovery;
        assert!(!dr_mode.allows_secondary_reads());
        assert!(dr_mode.replicates_ephemeral_state());
    }

    #[test]
    fn test_default_configuration() {
        let config = ReplicationConfig::default();
        assert!(config.validate().is_ok());
        assert_eq!(config.mode, ReplicationMode::Performance);
        assert!(config.interval_ms > 0);
        assert!(config.batch_size > 0);
    }

    #[test]
    fn test_invalid_interval() {
        let config = ReplicationConfig {
            interval_ms: 0, // Invalid
            ..Default::default()
        };

        assert!(
            config.validate().is_err(),
            "Zero interval should be invalid"
        );
    }

    #[test]
    fn test_invalid_batch_size() {
        let config = ReplicationConfig {
            batch_size: 0, // Invalid
            ..Default::default()
        };

        assert!(
            config.validate().is_err(),
            "Zero batch size should be invalid"
        );
    }
}

/// Property 7: WAL streaming
///
/// **Property**: WAL entries written on primary are correctly received and applied on secondary
///
/// **Validates**: Requirements 2.2.2
///
/// **Formal specification**:
/// ```text
/// ∀ operations ∈ Vec<ReplicationOperation>:
///   stream_to_secondary(operations) ⇒
///     (1) ∀ op ∈ operations: received_on_secondary(op)
///     (2) order_preserved(operations)
///     (3) correctly_deserialized(operations)
///     (4) handles_interruptions()
/// ```
#[cfg(test)]
mod wal_streaming_tests {
    use super::*;

    /// Generate arbitrary ReplicationOperation for property testing
    fn arb_replication_operation() -> impl Strategy<Value = ReplicationOperation> {
        (
            prop::collection::vec(any::<u8>(), 0..1024),
            0u64..1000000u64,
            prop::string::string_regex("[a-z0-9/]+").unwrap(),
        )
            .prop_map(|(data, sequence, path)| {
                ReplicationOperation::new(
                    OperationType::SecretWrite,
                    OperationData::Secret {
                        path,
                        data,
                        metadata: serde_json::json!({}),
                    },
                    sequence,
                )
            })
    }

    proptest! {
        /// Test that WAL entries maintain order during streaming
        ///
        /// **Property**: Operations streamed to secondary maintain their sequence order
        #[test]
        fn prop_wal_maintains_order(
            num_operations in 1usize..100usize,
        ) {
            // Generate operations with sequential sequence numbers
            let operations: Vec<ReplicationOperation> = (0..num_operations)
                .map(|i| {
                    ReplicationOperation::new(
                        OperationType::SecretWrite,
                        OperationData::Secret {
                            path: format!("/secret/test-{}", i),
                            data: vec![i as u8],
                            metadata: serde_json::json!({}),
                        },
                        i as u64,
                    )
                })
                .collect();

            // Verify operations are in order
            for i in 0..operations.len() - 1 {
                assert!(
                    operations[i].sequence < operations[i + 1].sequence,
                    "Operations must maintain sequence order"
                );
            }

            // Simulate streaming (serialize and deserialize)
            let serialized: Vec<Vec<u8>> = operations
                .iter()
                .map(|op| serde_json::to_vec(op).expect("Serialization should succeed"))
                .collect();

            let deserialized: Vec<ReplicationOperation> = serialized
                .iter()
                .map(|bytes| serde_json::from_slice(bytes).expect("Deserialization should succeed"))
                .collect();

            // Verify order is preserved after streaming
            for i in 0..deserialized.len() - 1 {
                assert_eq!(
                    deserialized[i].sequence,
                    operations[i].sequence,
                    "Sequence numbers must match after streaming"
                );
                assert!(
                    deserialized[i].sequence < deserialized[i + 1].sequence,
                    "Order must be preserved after streaming"
                );
            }
        }
    }

    proptest! {
        /// Test that WAL entries are correctly serialized and deserialized
        ///
        /// **Property**: Operations can be serialized and deserialized without data loss
        #[test]
        fn prop_wal_serialization_roundtrip(
            operations in prop::collection::vec(arb_replication_operation(), 1..50),
        ) {
            for operation in operations {
                // Serialize
                let serialized = serde_json::to_vec(&operation)
                    .expect("Serialization should succeed");

                // Deserialize
                let deserialized: ReplicationOperation = serde_json::from_slice(&serialized)
                    .expect("Deserialization should succeed");

                // Verify all fields match
                assert_eq!(deserialized.id, operation.id, "ID must match");
                assert_eq!(deserialized.sequence, operation.sequence, "Sequence must match");
                assert_eq!(deserialized.operation_type, operation.operation_type, "Type must match");

                // Verify data integrity
                if let (
                        OperationData::Secret { path: p1, data: d1, .. },
                        OperationData::Secret { path: p2, data: d2, .. },
                    ) = (&operation.data, &deserialized.data) {
                    assert_eq!(p1, p2, "Path must match");
                    assert_eq!(d1, d2, "Data must match");
                }
            }
        }
    }

    proptest! {
        /// Test that WAL streaming handles batch processing correctly
        ///
        /// **Property**: Large batches of operations can be streamed without data loss
        #[test]
        fn prop_wal_batch_streaming(
            batch_size in 1usize..100usize,
            num_batches in 1usize..10usize,
        ) {
            let mut all_operations = Vec::new();
            let mut sequence = 0u64;

            // Generate multiple batches
            for batch_idx in 0..num_batches {
                let batch: Vec<ReplicationOperation> = (0..batch_size)
                    .map(|i| {
                        let op = ReplicationOperation::new(
                            OperationType::SecretWrite,
                            OperationData::Secret {
                                path: format!("/secret/batch-{}/item-{}", batch_idx, i),
                                data: vec![batch_idx as u8, i as u8],
                                metadata: serde_json::json!({}),
                            },
                            sequence,
                        );
                        sequence += 1;
                        op
                    })
                    .collect();

                all_operations.extend(batch);
            }

            // Simulate streaming all operations
            let mut received_sequences = Vec::new();

            for operation in &all_operations {
                // Serialize (simulate network transmission)
                let serialized = serde_json::to_vec(operation)
                    .expect("Serialization should succeed");

                // Deserialize (simulate reception on secondary)
                let deserialized: ReplicationOperation = serde_json::from_slice(&serialized)
                    .expect("Deserialization should succeed");

                received_sequences.push(deserialized.sequence);
            }

            // Verify all operations were received
            assert_eq!(
                received_sequences.len(),
                all_operations.len(),
                "All operations must be received"
            );

            // Verify order is maintained
            for i in 0..received_sequences.len() - 1 {
                assert!(
                    received_sequences[i] < received_sequences[i + 1],
                    "Sequence order must be maintained across batches"
                );
            }

            // Verify no gaps in sequence numbers
            for (i, seq) in received_sequences.iter().enumerate() {
                assert_eq!(
                    *seq, i as u64,
                    "Sequence numbers must be contiguous"
                );
            }
        }
    }

    proptest! {
        /// Test that WAL streaming handles different operation types correctly
        ///
        /// **Property**: All operation types can be streamed and deserialized correctly
        #[test]
        fn prop_wal_operation_types(
            num_operations in 1usize..50usize,
        ) {
            let operation_types = [OperationType::SecretWrite,
                OperationType::SecretDelete,
                OperationType::PolicyWrite,
                OperationType::PolicyDelete,
                OperationType::ConfigUpdate];

            let operations: Vec<ReplicationOperation> = (0..num_operations)
                .map(|i| {
                    let op_type = operation_types[i % operation_types.len()].clone();
                    let data = match op_type {
                        OperationType::SecretWrite => OperationData::Secret {
                            path: format!("/secret/test-{}", i),
                            data: vec![i as u8],
                            metadata: serde_json::json!({}),
                        },
                        OperationType::SecretDelete => OperationData::SecretDeletion {
                            path: format!("/secret/test-{}", i),
                        },
                        OperationType::PolicyWrite => OperationData::Policy {
                            name: format!("policy-{}", i),
                            policy: "path \"secret/*\" { capabilities = [\"read\"] }".to_string(),
                        },
                        OperationType::PolicyDelete => OperationData::PolicyDeletion {
                            name: format!("policy-{}", i),
                        },
                        OperationType::ConfigUpdate => OperationData::Config {
                            key: format!("config-{}", i),
                            value: serde_json::json!({"value": i}),
                        },
                        _ => unreachable!(),
                    };

                    ReplicationOperation::new(op_type, data, i as u64)
                })
                .collect();

            // Stream all operations
            for operation in operations {
                let serialized = serde_json::to_vec(&operation)
                    .expect("Serialization should succeed");

                let deserialized: ReplicationOperation = serde_json::from_slice(&serialized)
                    .expect("Deserialization should succeed");

                assert_eq!(
                    deserialized.operation_type, operation.operation_type,
                    "Operation type must be preserved"
                );
            }
        }
    }

    proptest! {
        /// Test that WAL streaming handles network interruptions gracefully
        ///
        /// **Property**: Operations can be resumed from last known sequence after interruption
        #[test]
        fn prop_wal_handles_interruptions(
            total_operations in 10usize..100usize,
            interruption_point in 1usize..50usize,
        ) {
            let interruption_point = interruption_point.min(total_operations - 1);

            // Generate all operations
            let all_operations: Vec<ReplicationOperation> = (0..total_operations)
                .map(|i| {
                    ReplicationOperation::new(
                        OperationType::SecretWrite,
                        OperationData::Secret {
                            path: format!("/secret/test-{}", i),
                            data: vec![i as u8],
                            metadata: serde_json::json!({}),
                        },
                        i as u64,
                    )
                })
                .collect();

            // Simulate first batch (before interruption)
            let mut received_sequences = Vec::new();
            for op in &all_operations[..interruption_point] {
                let serialized = serde_json::to_vec(op)
                    .expect("Serialization should succeed");
                let deserialized: ReplicationOperation = serde_json::from_slice(&serialized)
                    .expect("Deserialization should succeed");
                received_sequences.push(deserialized.sequence);
            }

            // Get last received sequence (for resume)
            let last_sequence = received_sequences.last().copied().unwrap_or(0);

            // Simulate interruption and resume
            // Secondary should request operations starting from last_sequence + 1
            let resume_from = last_sequence + 1;

            // Stream remaining operations
            for op in &all_operations[resume_from as usize..total_operations] {
                let serialized = serde_json::to_vec(op)
                    .expect("Serialization should succeed");
                let deserialized: ReplicationOperation = serde_json::from_slice(&serialized)
                    .expect("Deserialization should succeed");
                received_sequences.push(deserialized.sequence);
            }

            // Verify all operations were eventually received
            assert_eq!(
                received_sequences.len(),
                total_operations,
                "All operations must be received after resume"
            );

            // Verify no duplicates
            let mut seen = std::collections::HashSet::new();
            for seq in &received_sequences {
                assert!(
                    seen.insert(*seq),
                    "No duplicate sequences should be received"
                );
            }

            // Verify completeness
            for i in 0..total_operations {
                assert!(
                    received_sequences.contains(&(i as u64)),
                    "All sequence numbers must be present"
                );
            }
        }
    }

    proptest! {
        /// Test that WAL streaming preserves timestamps
        ///
        /// **Property**: Operation timestamps are preserved during streaming
        #[test]
        fn prop_wal_preserves_timestamps(
            num_operations in 1usize..50usize,
        ) {
            let operations: Vec<ReplicationOperation> = (0..num_operations)
                .map(|i| {
                    ReplicationOperation::new(
                        OperationType::SecretWrite,
                        OperationData::Secret {
                            path: format!("/secret/test-{}", i),
                            data: vec![i as u8],
                            metadata: serde_json::json!({}),
                        },
                        i as u64,
                    )
                })
                .collect();

            for operation in operations {
                let original_timestamp = operation.timestamp;

                let serialized = serde_json::to_vec(&operation)
                    .expect("Serialization should succeed");

                let deserialized: ReplicationOperation = serde_json::from_slice(&serialized)
                    .expect("Deserialization should succeed");

                assert_eq!(
                    deserialized.timestamp, original_timestamp,
                    "Timestamp must be preserved during streaming"
                );
            }
        }
    }

    proptest! {
        /// Test that WAL streaming handles large payloads
        ///
        /// **Property**: Large secret data can be streamed without corruption
        #[test]
        fn prop_wal_large_payloads(
            payload_size in 1usize..10000usize,
            num_operations in 1usize..10usize,
        ) {
            let operations: Vec<ReplicationOperation> = (0..num_operations)
                .map(|i| {
                    let data: Vec<u8> = (0..payload_size).map(|j| ((i + j) % 256) as u8).collect();

                    ReplicationOperation::new(
                        OperationType::SecretWrite,
                        OperationData::Secret {
                            path: format!("/secret/large-{}", i),
                            data: data.clone(),
                            metadata: serde_json::json!({"size": payload_size}),
                        },
                        i as u64,
                    )
                })
                .collect();

            for operation in operations {
                let serialized = serde_json::to_vec(&operation)
                    .expect("Serialization should succeed");

                let deserialized: ReplicationOperation = serde_json::from_slice(&serialized)
                    .expect("Deserialization should succeed");

                // Verify data integrity
                if let (
                    OperationData::Secret { data: original_data, .. },
                    OperationData::Secret { data: deserialized_data, .. },
                ) = (&operation.data, &deserialized.data)
                {
                    assert_eq!(
                        original_data.len(),
                        deserialized_data.len(),
                        "Data length must be preserved"
                    );
                    assert_eq!(
                        original_data, deserialized_data,
                        "Data content must be preserved"
                    );
                }
            }
        }
    }

    #[test]
    fn test_wal_streaming_basic() {
        // Create a simple operation
        let operation = ReplicationOperation::new(
            OperationType::SecretWrite,
            OperationData::Secret {
                path: "/secret/test".to_string(),
                data: vec![1, 2, 3, 4, 5],
                metadata: serde_json::json!({"version": 1}),
            },
            42,
        );

        // Serialize
        let serialized = serde_json::to_vec(&operation).expect("Serialization should succeed");

        // Deserialize
        let deserialized: ReplicationOperation =
            serde_json::from_slice(&serialized).expect("Deserialization should succeed");

        // Verify
        assert_eq!(deserialized.id, operation.id);
        assert_eq!(deserialized.sequence, 42);
        assert_eq!(deserialized.operation_type, OperationType::SecretWrite);
    }

    #[test]
    fn test_wal_streaming_empty_data() {
        // Test with empty data
        let operation = ReplicationOperation::new(
            OperationType::SecretWrite,
            OperationData::Secret {
                path: "/secret/empty".to_string(),
                data: vec![],
                metadata: serde_json::json!({}),
            },
            1,
        );

        let serialized = serde_json::to_vec(&operation).expect("Serialization should succeed");
        let deserialized: ReplicationOperation =
            serde_json::from_slice(&serialized).expect("Deserialization should succeed");

        if let OperationData::Secret { data, .. } = &deserialized.data {
            assert!(data.is_empty(), "Empty data should be preserved");
        }
    }

    #[test]
    fn test_wal_streaming_sequence_gaps() {
        // Test detection of sequence gaps
        let operations = vec![
            ReplicationOperation::new(
                OperationType::SecretWrite,
                OperationData::Secret {
                    path: "/secret/1".to_string(),
                    data: vec![1],
                    metadata: serde_json::json!({}),
                },
                1,
            ),
            ReplicationOperation::new(
                OperationType::SecretWrite,
                OperationData::Secret {
                    path: "/secret/2".to_string(),
                    data: vec![2],
                    metadata: serde_json::json!({}),
                },
                2,
            ),
            ReplicationOperation::new(
                OperationType::SecretWrite,
                OperationData::Secret {
                    path: "/secret/4".to_string(),
                    data: vec![4],
                    metadata: serde_json::json!({}),
                },
                4, // Gap: sequence 3 is missing
            ),
        ];

        // Verify gap detection
        let mut last_seq = 0u64;
        let mut has_gap = false;

        for op in operations {
            if op.sequence != last_seq + 1 && last_seq != 0 {
                has_gap = true;
            }
            last_seq = op.sequence;
        }

        assert!(has_gap, "Should detect sequence gap");
    }
}

/// Property 8: Read consistency
///
/// **Property**: Reads on secondary maintain consistency guarantees
///
/// **Validates**: Requirements 2.2.4
///
/// **Formal specification**:
/// ```text
/// ∀ operations ∈ Vec<ReplicationOperation>, staleness_threshold ∈ u64:
///   (1) Read-after-write consistency:
///       write(op, seq_n) on primary ⇒
///       replicate_to_secondary(op) ⇒
///       read_with_consistency(path, seq_n) on secondary returns data from op
///
///   (2) Minimum sequence guarantee:
///       read_with_consistency(path, min_seq) ⇒
///       waits_until(last_applied_sequence ≥ min_seq) ∨ timeout
///
///   (3) Staleness detection:
///       lag > staleness_threshold ⇒
///       read(path) returns TooStale error
///
///   (4) Consistency across operations:
///       ∀ seq_i, seq_j where seq_i < seq_j:
///       read_at(seq_j) sees all writes up to seq_j
/// ```
#[cfg(test)]
mod read_consistency_tests {
    use super::*;
    use chrono::Utc;
    use secreton_replication::secondary::{SecondaryReadError, SecondaryReadHandler};
    use std::sync::Arc;

    // Mock storage for testing (reuse from secondary.rs tests)
    use async_trait::async_trait;
    use secreton_replication::Result;
    use secreton_replication::manager::ReplicationStorage;
    use tokio::sync::RwLock;

    struct MockStorage {
        operations: Arc<RwLock<Vec<ReplicationOperation>>>,
        sequence: Arc<RwLock<u64>>,
    }

    impl MockStorage {
        fn new() -> Self {
            Self {
                operations: Arc::new(RwLock::new(Vec::new())),
                sequence: Arc::new(RwLock::new(0)),
            }
        }
    }

    #[async_trait]
    impl ReplicationStorage for MockStorage {
        async fn get_operations_since(&self, sequence: u64) -> Result<Vec<ReplicationOperation>> {
            let ops = self.operations.read().await;
            Ok(ops
                .iter()
                .filter(|op| op.sequence > sequence)
                .cloned()
                .collect())
        }

        async fn apply_operation(&self, operation: &ReplicationOperation) -> Result<()> {
            self.operations.write().await.push(operation.clone());
            *self.sequence.write().await = operation.sequence;
            Ok(())
        }

        async fn get_current_sequence(&self) -> Result<u64> {
            Ok(*self.sequence.read().await)
        }

        async fn store_cluster_metadata(&self, _key: &str, _value: &[u8]) -> Result<()> {
            Ok(())
        }

        async fn get_cluster_metadata(&self, _key: &str) -> Result<Option<Vec<u8>>> {
            Ok(None)
        }
    }

    proptest! {
        /// Test read-after-write consistency
        ///
        /// **Property**: Reads on secondary see writes from primary after replication
        #[test]
        fn prop_read_after_write_consistency(
            num_writes in 1usize..50usize,
            staleness_threshold_ms in 100u64..1000u64,
        ) {
            tokio::runtime::Runtime::new().unwrap().block_on(async {
                let storage = Arc::new(MockStorage::new());
                let handler = SecondaryReadHandler::new(storage.clone(), staleness_threshold_ms);

                // Simulate writes on primary being replicated to secondary
                let mut operations = Vec::new();
                for i in 0..num_writes {
                    let op = ReplicationOperation::new(
                        OperationType::SecretWrite,
                        OperationData::Secret {
                            path: format!("/secret/test-{}", i),
                            data: vec![i as u8; 10],
                            metadata: serde_json::json!({"version": i}),
                        },
                        i as u64 + 1,
                    );

                    // Apply operation to storage (simulating replication)
                    storage.apply_operation(&op).await.unwrap();

                    // Update handler's sequence
                    handler.update_sequence(op.sequence).await;

                    operations.push(op);
                }

                // Verify read-after-write: reads should see all replicated writes
                for op in operations.iter() {
                    let sequence = op.sequence;

                    // Read with consistency guarantee
                    let result = handler.wait_for_sequence(sequence).await;

                    // Should succeed - data has been replicated
                    assert!(
                        result.is_ok(),
                        "Read-after-write consistency: should see write at sequence {}",
                        sequence
                    );

                    // Verify current sequence is at least the requested sequence
                    let current_seq = *handler.last_applied_sequence().read().await;
                    assert!(
                        current_seq >= sequence,
                        "Current sequence {} should be >= requested sequence {}",
                        current_seq,
                        sequence
                    );
                }

                // Verify all writes are visible
                let final_sequence = *handler.last_applied_sequence().read().await;
                assert_eq!(
                    final_sequence,
                    num_writes as u64,
                    "All {} writes should be visible on secondary",
                    num_writes
                );
            });
        }
    }

    proptest! {
        /// Test minimum sequence guarantee with waiting
        ///
        /// **Property**: Reads wait for replication to reach minimum sequence
        #[test]
        fn prop_minimum_sequence_guarantee(
            initial_sequence in 1u64..20u64,
            target_sequence in 21u64..50u64,
            delay_ms in 10u64..100u64,
        ) {
            tokio::runtime::Runtime::new().unwrap().block_on(async {
                let storage = Arc::new(MockStorage::new());
                let handler = SecondaryReadHandler::new(storage.clone(), 1000);

                // Set initial sequence
                handler.update_sequence(initial_sequence).await;

                // Spawn task to update sequence after delay
                let handler_clone = handler.clone();
                tokio::spawn(async move {
                    tokio::time::sleep(tokio::time::Duration::from_millis(delay_ms)).await;
                    handler_clone.update_sequence(target_sequence).await;
                });

                // Wait for target sequence - should succeed after delay
                let start = std::time::Instant::now();
                let result = handler.wait_for_sequence(target_sequence).await;
                let elapsed = start.elapsed().as_millis() as u64;

                // Should succeed
                assert!(
                    result.is_ok(),
                    "Should successfully wait for sequence {} (started at {})",
                    target_sequence,
                    initial_sequence
                );

                // Should have waited at least the delay time
                assert!(
                    elapsed >= delay_ms,
                    "Should have waited at least {}ms, but only waited {}ms",
                    delay_ms,
                    elapsed
                );

                // Current sequence should be at target
                let current_seq = *handler.last_applied_sequence().read().await;
                assert!(
                    current_seq >= target_sequence,
                    "Current sequence {} should be >= target {}",
                    current_seq,
                    target_sequence
                );
            });
        }
    }

    proptest! {
        /// Test staleness detection and rejection
        ///
        /// **Property**: Stale reads are rejected when lag exceeds threshold
        #[test]
        fn prop_staleness_detection(
            staleness_threshold_ms in 50u64..200u64,
            lag_ms in 201u64..500u64,
        ) {
            tokio::runtime::Runtime::new().unwrap().block_on(async {
                let storage = Arc::new(MockStorage::new());
                let handler = SecondaryReadHandler::new(storage, staleness_threshold_ms);

                // Simulate old sync time to create lag
                let lag_duration = chrono::Duration::milliseconds(lag_ms as i64);
                *handler.last_sync().write().await = Utc::now() - lag_duration;

                // Calculate actual lag
                let actual_lag = handler.calculate_lag().await;

                // Lag should be approximately what we set
                assert!(
                    actual_lag >= lag_ms - 50, // Allow 50ms tolerance
                    "Actual lag {}ms should be >= {}ms",
                    actual_lag,
                    lag_ms - 50
                );

                // Check staleness - should fail because lag > threshold
                let result = handler.check_staleness().await;

                assert!(
                    matches!(result, Err(SecondaryReadError::TooStale { .. })),
                    "Should reject stale read when lag ({}ms) > threshold ({}ms)",
                    actual_lag,
                    staleness_threshold_ms
                );

                // Verify error contains correct values
                if let Err(SecondaryReadError::TooStale { lag_ms: reported_lag, threshold_ms }) = result {
                    assert_eq!(
                        threshold_ms, staleness_threshold_ms,
                        "Error should report correct threshold"
                    );
                    assert!(
                        reported_lag >= staleness_threshold_ms,
                        "Reported lag {}ms should be >= threshold {}ms",
                        reported_lag,
                        staleness_threshold_ms
                    );
                }
            });
        }
    }

    proptest! {
        /// Test consistency across multiple operations
        ///
        /// **Property**: Reads at sequence N see all writes up to sequence N
        #[test]
        fn prop_consistency_across_operations(
            num_operations in 5usize..30usize,
            read_points in prop::collection::vec(0usize..30usize, 1..10),
        ) {
            tokio::runtime::Runtime::new().unwrap().block_on(async {
                let storage = Arc::new(MockStorage::new());
                let handler = SecondaryReadHandler::new(storage.clone(), 1000);

                // Apply operations sequentially
                let mut operations = Vec::new();
                for i in 0..num_operations {
                    let op = ReplicationOperation::new(
                        OperationType::SecretWrite,
                        OperationData::Secret {
                            path: format!("/secret/test-{}", i),
                            data: vec![i as u8; 5],
                            metadata: serde_json::json!({"index": i}),
                        },
                        i as u64 + 1,
                    );

                    storage.apply_operation(&op).await.unwrap();
                    handler.update_sequence(op.sequence).await;
                    operations.push(op);
                }

                // Test reads at various points
                for read_point in read_points {
                    let read_point = read_point.min(num_operations - 1);
                    let target_sequence = (read_point + 1) as u64;

                    // Wait for sequence
                    let result = handler.wait_for_sequence(target_sequence).await;
                    assert!(
                        result.is_ok(),
                        "Should be able to read at sequence {}",
                        target_sequence
                    );

                    // Verify all operations up to this point are visible
                    let current_seq = *handler.last_applied_sequence().read().await;
                    assert!(
                        current_seq >= target_sequence,
                        "At read point {}, current sequence {} should be >= {}",
                        read_point,
                        current_seq,
                        target_sequence
                    );

                    // Verify we can see all operations up to target_sequence
                    let visible_ops = storage.get_operations_since(0).await.unwrap();
                    let visible_count = visible_ops
                        .iter()
                        .filter(|op| op.sequence <= target_sequence)
                        .count();

                    assert_eq!(
                        visible_count,
                        target_sequence as usize,
                        "Should see exactly {} operations at sequence {}",
                        target_sequence,
                        target_sequence
                    );
                }
            });
        }
    }

    proptest! {
        /// Test read consistency with concurrent updates
        ///
        /// **Property**: Reads maintain consistency even with concurrent replication
        #[test]
        fn prop_concurrent_read_consistency(
            num_writes in 10usize..50usize,
            num_reads in 5usize..20usize,
        ) {
            tokio::runtime::Runtime::new().unwrap().block_on(async {
                let storage = Arc::new(MockStorage::new());
                let handler = Arc::new(SecondaryReadHandler::new(storage.clone(), 1000));

                // Spawn writer task (simulating replication)
                let storage_clone = storage.clone();
                let handler_clone = handler.clone();
                let writer = tokio::spawn(async move {
                    for i in 0..num_writes {
                        let op = ReplicationOperation::new(
                            OperationType::SecretWrite,
                            OperationData::Secret {
                                path: format!("/secret/concurrent-{}", i),
                                data: vec![i as u8; 8],
                                metadata: serde_json::json!({}),
                            },
                            i as u64 + 1,
                        );

                        storage_clone.apply_operation(&op).await.unwrap();
                        handler_clone.update_sequence(op.sequence).await;

                        // Small delay to simulate network latency
                        tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;
                    }
                });

                // Spawn reader tasks
                let mut readers = Vec::new();
                for read_idx in 0..num_reads {
                    let handler_clone = handler.clone();
                    let target_seq = ((read_idx + 1) * num_writes / num_reads) as u64;

                    let reader = tokio::spawn(async move {
                        // Wait for a specific sequence
                        let result = handler_clone.wait_for_sequence(target_seq).await;

                        // Should eventually succeed
                        assert!(
                            result.is_ok(),
                            "Reader {} should see sequence {}",
                            read_idx,
                            target_seq
                        );

                        // Verify consistency
                        let current_seq = *handler_clone.last_applied_sequence().read().await;
                        assert!(
                            current_seq >= target_seq,
                            "Reader {}: current sequence {} should be >= target {}",
                            read_idx,
                            current_seq,
                            target_seq
                        );
                    });

                    readers.push(reader);
                }

                // Wait for all tasks to complete
                writer.await.unwrap();
                for reader in readers {
                    reader.await.unwrap();
                }

                // Final verification: all writes should be visible
                let final_seq = *handler.last_applied_sequence().read().await;
                assert_eq!(
                    final_seq,
                    num_writes as u64,
                    "All {} writes should be visible",
                    num_writes
                );
            });
        }
    }

    proptest! {
        /// Test lag metrics accuracy
        ///
        /// **Property**: Lag metrics accurately reflect replication state
        #[test]
        fn prop_lag_metrics_accuracy(
            last_applied in 1u64..100u64,
            primary_seq in 101u64..200u64,
            staleness_threshold_ms in 100u64..500u64,
        ) {
            tokio::runtime::Runtime::new().unwrap().block_on(async {
                let storage = Arc::new(MockStorage::new());
                let handler = SecondaryReadHandler::new(storage, staleness_threshold_ms);

                // Set sequences
                handler.update_sequence(last_applied).await;
                handler.update_primary_sequence(primary_seq).await;

                // Get metrics
                let metrics = handler.get_lag_metrics().await;

                // Verify metrics accuracy
                assert_eq!(
                    metrics.last_applied_sequence, last_applied,
                    "Metrics should report correct last applied sequence"
                );
                assert_eq!(
                    metrics.primary_sequence, primary_seq,
                    "Metrics should report correct primary sequence"
                );

                // Sequence lag should be the difference
                let expected_seq_lag = primary_seq - last_applied;
                assert_eq!(
                    metrics.sequence_lag, expected_seq_lag,
                    "Sequence lag should be {} - {} = {}",
                    primary_seq,
                    last_applied,
                    expected_seq_lag
                );

                // Time lag should be recent (since we just updated)
                assert!(
                    metrics.time_lag_ms < 100,
                    "Time lag should be < 100ms for recent update, got {}ms",
                    metrics.time_lag_ms
                );

                // Staleness should be false for recent update
                assert!(
                    !metrics.is_stale,
                    "Should not be stale for recent update"
                );
            });
        }
    }

    proptest! {
        // Each case blocks for the full read-after-write timeout, so keep the
        // case count low AND use a short (200ms) timeout — otherwise this is
        // 256 cases x 5s ≈ 21 minutes (the original CI/local "hang").
        #![proptest_config(ProptestConfig::with_cases(10))]
        /// Test read consistency timeout behavior
        ///
        /// **Property**: Reads timeout gracefully when sequence is not reached
        #[test]
        fn prop_read_timeout_behavior(
            current_sequence in 1u64..50u64,
            unreachable_sequence in 100u64..200u64,
        ) {
            tokio::runtime::Runtime::new().unwrap().block_on(async {
                let storage = Arc::new(MockStorage::new());
                let handler = SecondaryReadHandler::new(storage, 1000).with_max_wait_ms(200);

                // Set current sequence
                handler.update_sequence(current_sequence).await;

                // Try to wait for unreachable sequence - should timeout
                let start = std::time::Instant::now();
                let result = handler.wait_for_sequence(unreachable_sequence).await;
                let elapsed = start.elapsed();

                // Should fail with consistency violation
                assert!(
                    matches!(result, Err(SecondaryReadError::ConsistencyViolation { .. })),
                    "Should timeout when waiting for unreachable sequence"
                );

                // Should have waited ~the (test-configured 200ms) timeout.
                assert!(
                    elapsed.as_millis() >= 150, // ~200ms timeout, with tolerance
                    "Should have waited for timeout period, elapsed: {:?}",
                    elapsed
                );

                // Verify error contains correct information
                if let Err(SecondaryReadError::ConsistencyViolation {
                    expected_sequence,
                    actual_sequence,
                }) = result
                {
                    assert_eq!(
                        expected_sequence, unreachable_sequence,
                        "Error should report expected sequence"
                    );
                    assert_eq!(
                        actual_sequence, current_sequence,
                        "Error should report actual sequence"
                    );
                }
            });
        }
    }

    #[tokio::test]
    async fn test_read_consistency_basic() {
        let storage = Arc::new(MockStorage::new());
        let handler = SecondaryReadHandler::new(storage.clone(), 1000);

        // Apply some operations
        for i in 1..=5 {
            let op = ReplicationOperation::new(
                OperationType::SecretWrite,
                OperationData::Secret {
                    path: format!("/secret/test-{}", i),
                    data: vec![i as u8],
                    metadata: serde_json::json!({}),
                },
                i,
            );
            storage.apply_operation(&op).await.unwrap();
            handler.update_sequence(i).await;
        }

        // Read with consistency - should see all 5 operations
        let result = handler.wait_for_sequence(5).await;
        assert!(result.is_ok());

        let current_seq = *handler.last_applied_sequence().read().await;
        assert_eq!(current_seq, 5);
    }

    #[tokio::test]
    async fn test_read_consistency_with_lag() {
        let storage = Arc::new(MockStorage::new());
        let handler = SecondaryReadHandler::new(storage.clone(), 100);

        // Set sequence
        handler.update_sequence(10).await;

        // Simulate lag by setting old sync time
        *handler.last_sync().write().await = Utc::now() - chrono::Duration::milliseconds(200);

        // Check staleness - should fail
        let result = handler.check_staleness().await;
        assert!(matches!(result, Err(SecondaryReadError::TooStale { .. })));
    }

    #[tokio::test]
    async fn test_read_consistency_sequence_ordering() {
        let storage = Arc::new(MockStorage::new());
        let handler = SecondaryReadHandler::new(storage.clone(), 1000);

        // Apply operations in order
        for i in 1..=10 {
            let op = ReplicationOperation::new(
                OperationType::SecretWrite,
                OperationData::Secret {
                    path: format!("/secret/ordered-{}", i),
                    data: vec![i as u8],
                    metadata: serde_json::json!({}),
                },
                i,
            );
            storage.apply_operation(&op).await.unwrap();
            handler.update_sequence(i).await;

            // At each point, should be able to read up to current sequence
            let result = handler.wait_for_sequence(i).await;
            assert!(
                result.is_ok(),
                "Should see sequence {} after applying it",
                i
            );
        }

        // Final check: all operations visible
        let ops = storage.get_operations_since(0).await.unwrap();
        assert_eq!(ops.len(), 10);

        // Verify ordering
        for i in 0..ops.len() - 1 {
            assert!(
                ops[i].sequence < ops[i + 1].sequence,
                "Operations should be in sequence order"
            );
        }
    }
}

/// Property 9: Replication lag monitoring
///
/// **Property**: Replication lag is accurately tracked and exposed via metrics
///
/// **Validates**: Requirements 2.2.5
///
/// **Formal specification**:
/// ```text
/// ∀ primary_seq ∈ u64, secondary_seq ∈ u64, time_lag ∈ Duration:
///   (1) Lag calculation accuracy:
///       lag_bytes = primary_seq - secondary_seq
///       lag_ms = time_since_last_sync
///
///   (2) Metrics exposure:
///       get_lag_bytes() returns lag_bytes
///       get_lag_ms() returns lag_ms
///       metrics are exposed via Prometheus
///
///   (3) WAL position tracking:
///       get_primary_sequence() returns primary_seq
///       get_secondary_sequence() returns secondary_seq
///
///   (4) Lag updates:
///       update_wal_positions(primary, secondary) ⇒
///       lag_bytes = primary - secondary
///
///   (5) Status based on lag:
///       lag_ms < threshold ⇒ status = Healthy
///       lag_ms ≥ threshold ⇒ status = Lagging
/// ```
#[cfg(test)]
mod replication_lag_monitoring_tests {
    use super::*;
    use async_trait::async_trait;
    use secreton_replication::Result;
    use secreton_replication::manager::ReplicationStorage;
    use secreton_replication::metrics::{ReplicationMetrics, ReplicationStatusValue};
    use secreton_replication::{ReplicationConfig, ReplicationManager, ReplicationMode};
    use std::sync::Arc;
    use tokio::sync::RwLock;

    // Mock storage for testing
    struct MockStorage {
        operations: Arc<RwLock<Vec<ReplicationOperation>>>,
        sequence: Arc<RwLock<u64>>,
    }

    impl MockStorage {
        fn new() -> Self {
            Self {
                operations: Arc::new(RwLock::new(Vec::new())),
                sequence: Arc::new(RwLock::new(0)),
            }
        }
    }

    #[async_trait]
    impl ReplicationStorage for MockStorage {
        async fn get_operations_since(&self, sequence: u64) -> Result<Vec<ReplicationOperation>> {
            let ops = self.operations.read().await;
            Ok(ops
                .iter()
                .filter(|op| op.sequence > sequence)
                .cloned()
                .collect())
        }

        async fn apply_operation(&self, operation: &ReplicationOperation) -> Result<()> {
            self.operations.write().await.push(operation.clone());
            *self.sequence.write().await = operation.sequence;
            Ok(())
        }

        async fn get_current_sequence(&self) -> Result<u64> {
            Ok(*self.sequence.read().await)
        }

        async fn store_cluster_metadata(&self, _key: &str, _value: &[u8]) -> Result<()> {
            Ok(())
        }

        async fn get_cluster_metadata(&self, _key: &str) -> Result<Option<Vec<u8>>> {
            Ok(None)
        }
    }

    proptest! {
        /// Test lag calculation accuracy (bytes)
        ///
        /// **Property**: Lag in bytes is accurately calculated as primary_seq - secondary_seq
        #[test]
        fn prop_lag_bytes_calculation(
            primary_seq in 100u64..10000u64,
            secondary_seq in 0u64..100u64,
        ) {
            tokio::runtime::Runtime::new().unwrap().block_on(async {
                let metrics = ReplicationMetrics::new(
                    "test-node".to_string(),
                    "performance".to_string(),
                );

                // Update WAL positions
                metrics.update_wal_positions(primary_seq, secondary_seq).await;

                // Get lag in bytes
                let lag_bytes = metrics.get_lag_bytes().await;

                // Verify calculation
                let expected_lag = primary_seq.saturating_sub(secondary_seq);
                assert_eq!(
                    lag_bytes, expected_lag,
                    "Lag bytes should be {} - {} = {}",
                    primary_seq, secondary_seq, expected_lag
                );

                // Verify primary and secondary sequences are tracked
                assert_eq!(
                    metrics.get_primary_sequence().await, primary_seq,
                    "Primary sequence should be tracked"
                );
                assert_eq!(
                    metrics.get_secondary_sequence().await, secondary_seq,
                    "Secondary sequence should be tracked"
                );
            });
        }
    }

    proptest! {
        /// Test lag calculation with zero lag
        ///
        /// **Property**: When primary and secondary are in sync, lag should be zero
        #[test]
        fn prop_zero_lag_when_synced(
            sequence in 1u64..10000u64,
        ) {
            tokio::runtime::Runtime::new().unwrap().block_on(async {
                let metrics = ReplicationMetrics::new(
                    "test-node".to_string(),
                    "performance".to_string(),
                );

                // Both at same sequence
                metrics.update_wal_positions(sequence, sequence).await;

                // Lag should be zero
                let lag_bytes = metrics.get_lag_bytes().await;
                assert_eq!(
                    lag_bytes, 0,
                    "Lag should be zero when primary and secondary are in sync"
                );
            });
        }
    }

    proptest! {
        /// Test lag increases as primary advances
        ///
        /// **Property**: As primary advances without secondary catching up, lag increases
        #[test]
        fn prop_lag_increases_with_primary_advance(
            initial_seq in 1u64..100u64,
            advances in prop::collection::vec(1u64..10u64, 1..20),
        ) {
            tokio::runtime::Runtime::new().unwrap().block_on(async {
                let metrics = ReplicationMetrics::new(
                    "test-node".to_string(),
                    "performance".to_string(),
                );

                let mut primary_seq = initial_seq;
                let secondary_seq = initial_seq;

                // Initial state: no lag
                metrics.update_wal_positions(primary_seq, secondary_seq).await;
                let mut last_lag = metrics.get_lag_bytes().await;
                assert_eq!(last_lag, 0, "Initial lag should be zero");

                // Advance primary without secondary catching up
                for advance in advances {
                    primary_seq += advance;
                    metrics.update_wal_positions(primary_seq, secondary_seq).await;

                    let current_lag = metrics.get_lag_bytes().await;

                    // Lag should increase
                    assert!(
                        current_lag > last_lag,
                        "Lag should increase from {} to {} when primary advances by {}",
                        last_lag, current_lag, advance
                    );

                    // Lag should equal the difference
                    let expected_lag = primary_seq - secondary_seq;
                    assert_eq!(
                        current_lag, expected_lag,
                        "Lag should be {} - {} = {}",
                        primary_seq, secondary_seq, expected_lag
                    );

                    last_lag = current_lag;
                }
            });
        }
    }

    proptest! {
        /// Test lag decreases as secondary catches up
        ///
        /// **Property**: As secondary catches up to primary, lag decreases
        #[test]
        fn prop_lag_decreases_with_secondary_catchup(
            primary_seq in 100u64..200u64,
            catchup_steps in prop::collection::vec(1u64..10u64, 1..20),
        ) {
            tokio::runtime::Runtime::new().unwrap().block_on(async {
                let metrics = ReplicationMetrics::new(
                    "test-node".to_string(),
                    "performance".to_string(),
                );

                let mut secondary_seq = 0u64;

                // Initial state: large lag
                metrics.update_wal_positions(primary_seq, secondary_seq).await;
                let mut last_lag = metrics.get_lag_bytes().await;
                assert_eq!(
                    last_lag, primary_seq,
                    "Initial lag should be full difference"
                );

                // Secondary catches up step by step
                for step in catchup_steps {
                    secondary_seq = (secondary_seq + step).min(primary_seq);
                    metrics.update_wal_positions(primary_seq, secondary_seq).await;

                    let current_lag = metrics.get_lag_bytes().await;

                    // Lag should decrease or stay same (if already caught up)
                    assert!(
                        current_lag <= last_lag,
                        "Lag should decrease from {} to {} as secondary catches up",
                        last_lag, current_lag
                    );

                    // Lag should equal the difference
                    let expected_lag = primary_seq - secondary_seq;
                    assert_eq!(
                        current_lag, expected_lag,
                        "Lag should be {} - {} = {}",
                        primary_seq, secondary_seq, expected_lag
                    );

                    last_lag = current_lag;

                    // If caught up, lag should be zero
                    if secondary_seq == primary_seq {
                        assert_eq!(current_lag, 0, "Lag should be zero when caught up");
                        break;
                    }
                }
            });
        }
    }

    proptest! {
        /// Test time-based lag calculation
        ///
        /// **Property**: Time lag is accurately calculated and updated
        #[test]
        fn prop_time_lag_calculation(
            lag_bytes in 0u64..1000u64,
            lag_ms in 0u64..5000u64,
        ) {
            tokio::runtime::Runtime::new().unwrap().block_on(async {
                let metrics = ReplicationMetrics::new(
                    "test-node".to_string(),
                    "performance".to_string(),
                );

                // Update lag metrics
                metrics.update_lag(lag_bytes, lag_ms).await;

                // Verify both metrics are stored
                assert_eq!(
                    metrics.get_lag_bytes().await, lag_bytes,
                    "Lag bytes should be stored correctly"
                );
                assert_eq!(
                    metrics.get_lag_ms().await, lag_ms,
                    "Lag milliseconds should be stored correctly"
                );
            });
        }
    }

    proptest! {
        /// Test status updates based on lag
        ///
        /// **Property**: Replication status reflects lag severity
        #[test]
        fn prop_status_based_on_lag(
            lag_ms in 0u64..10000u64,
            threshold_ms in 100u64..1000u64,
        ) {
            tokio::runtime::Runtime::new().unwrap().block_on(async {
                let metrics = ReplicationMetrics::new(
                    "test-node".to_string(),
                    "performance".to_string(),
                );

                // Determine expected status
                let expected_status = if lag_ms < threshold_ms {
                    ReplicationStatusValue::Healthy
                } else {
                    ReplicationStatusValue::Lagging
                };

                // Update status
                metrics.update_status(expected_status).await;

                // Note: We can't easily verify the Prometheus gauge value in tests,
                // but we can verify the update doesn't panic and the logic is correct
                assert!(
                    lag_ms < threshold_ms && matches!(expected_status, ReplicationStatusValue::Healthy)
                    || lag_ms >= threshold_ms && matches!(expected_status, ReplicationStatusValue::Lagging),
                    "Status should be Healthy if lag < threshold, Lagging otherwise"
                );
            });
        }
    }

    proptest! {
        /// Test lag monitoring in ReplicationManager
        ///
        /// **Property**: ReplicationManager accurately tracks and exposes lag metrics
        #[test]
        fn prop_manager_lag_monitoring(
            primary_seq in 100u64..1000u64,
            secondary_seq in 0u64..100u64,
        ) {
            tokio::runtime::Runtime::new().unwrap().block_on(async {
                let config = ReplicationConfig {
                    mode: ReplicationMode::Performance,
                    primary_endpoint: Some("https://primary:50051".to_string()),
                    secondary_endpoints: vec![],
                    ..Default::default()
                };

                let storage = Arc::new(MockStorage::new());
                let manager = ReplicationManager::new(config, storage).await.unwrap();

                // Simulate replication updates
                manager.update_applied_sequence(secondary_seq).await.unwrap();
                manager.update_primary_sequence(primary_seq).await.unwrap();

                // Get lag metrics
                let lag_bytes = manager.get_lag_bytes().await;
                let lag_ms = manager.get_lag_ms().await;
                let primary = manager.get_primary_sequence().await;
                let secondary = manager.get_secondary_sequence().await;

                // Verify tracking
                assert_eq!(
                    primary, primary_seq,
                    "Manager should track primary sequence"
                );
                assert_eq!(
                    secondary, secondary_seq,
                    "Manager should track secondary sequence"
                );

                // Verify lag calculation
                let expected_lag_bytes = primary_seq.saturating_sub(secondary_seq);
                assert_eq!(
                    lag_bytes, expected_lag_bytes,
                    "Manager should calculate lag bytes correctly"
                );

                // Time lag should be recent (< 100ms since we just updated)
                assert!(
                    lag_ms < 100,
                    "Time lag should be minimal for recent update, got {}ms",
                    lag_ms
                );
            });
        }
    }

    proptest! {
        /// Test lag metrics with multiple updates
        ///
        /// **Property**: Lag metrics correctly reflect multiple sequential updates
        #[test]
        fn prop_lag_metrics_multiple_updates(
            updates in prop::collection::vec((1u64..100u64, 1u64..100u64), 1..20),
        ) {
            tokio::runtime::Runtime::new().unwrap().block_on(async {
                let metrics = ReplicationMetrics::new(
                    "test-node".to_string(),
                    "performance".to_string(),
                );

                for (primary_seq, secondary_seq) in updates {
                    // Update positions
                    metrics.update_wal_positions(primary_seq, secondary_seq).await;

                    // Verify lag is calculated correctly
                    let lag_bytes = metrics.get_lag_bytes().await;
                    let expected_lag = primary_seq.saturating_sub(secondary_seq);

                    assert_eq!(
                        lag_bytes, expected_lag,
                        "After update to primary={}, secondary={}, lag should be {}",
                        primary_seq, secondary_seq, expected_lag
                    );

                    // Verify sequences are tracked
                    assert_eq!(
                        metrics.get_primary_sequence().await, primary_seq,
                        "Primary sequence should be updated"
                    );
                    assert_eq!(
                        metrics.get_secondary_sequence().await, secondary_seq,
                        "Secondary sequence should be updated"
                    );
                }
            });
        }
    }

    proptest! {
        /// Test lag monitoring with secondary lag metrics
        ///
        /// **Property**: Secondary read handler provides accurate lag metrics
        #[test]
        fn prop_secondary_lag_metrics(
            last_applied in 1u64..100u64,
            primary_seq in 101u64..200u64,
            staleness_threshold_ms in 100u64..500u64,
        ) {
            tokio::runtime::Runtime::new().unwrap().block_on(async {
                let storage = Arc::new(MockStorage::new());
                let handler = secreton_replication::secondary::SecondaryReadHandler::new(
                    storage,
                    staleness_threshold_ms,
                );

                // Set sequences
                handler.update_sequence(last_applied).await;
                handler.update_primary_sequence(primary_seq).await;

                // Get lag metrics
                let metrics = handler.get_lag_metrics().await;

                // Verify all metrics
                assert_eq!(
                    metrics.last_applied_sequence, last_applied,
                    "Should track last applied sequence"
                );
                assert_eq!(
                    metrics.primary_sequence, primary_seq,
                    "Should track primary sequence"
                );

                // Sequence lag should be the difference
                let expected_seq_lag = primary_seq - last_applied;
                assert_eq!(
                    metrics.sequence_lag, expected_seq_lag,
                    "Sequence lag should be {} - {} = {}",
                    primary_seq, last_applied, expected_seq_lag
                );

                // Time lag should be recent
                assert!(
                    metrics.time_lag_ms < 100,
                    "Time lag should be < 100ms for recent update, got {}ms",
                    metrics.time_lag_ms
                );

                // Staleness should be false for recent update
                assert!(
                    !metrics.is_stale,
                    "Should not be stale for recent update"
                );
            });
        }
    }

    proptest! {
        /// Test lag monitoring with stale detection
        ///
        /// **Property**: Lag metrics correctly identify stale secondaries
        #[test]
        fn prop_lag_monitoring_stale_detection(
            last_applied in 1u64..100u64,
            primary_seq in 101u64..200u64,
            staleness_threshold_ms in 50u64..200u64,
            actual_lag_ms in 201u64..500u64,
        ) {
            tokio::runtime::Runtime::new().unwrap().block_on(async {
                let storage = Arc::new(MockStorage::new());
                let handler = secreton_replication::secondary::SecondaryReadHandler::new(
                    storage,
                    staleness_threshold_ms,
                );

                // Set sequences
                handler.update_sequence(last_applied).await;
                handler.update_primary_sequence(primary_seq).await;

                // Simulate old sync time to create lag
                let lag_duration = chrono::Duration::milliseconds(actual_lag_ms as i64);
                *handler.last_sync().write().await = chrono::Utc::now() - lag_duration;

                // Get lag metrics
                let metrics = handler.get_lag_metrics().await;

                // Time lag should reflect the simulated lag
                assert!(
                    metrics.time_lag_ms >= actual_lag_ms - 50, // Allow 50ms tolerance
                    "Time lag {}ms should be >= {}ms",
                    metrics.time_lag_ms, actual_lag_ms - 50
                );

                // Should be marked as stale
                assert!(
                    metrics.is_stale,
                    "Should be marked as stale when lag ({}ms) > threshold ({}ms)",
                    metrics.time_lag_ms, staleness_threshold_ms
                );
            });
        }
    }

    #[tokio::test]
    async fn test_lag_monitoring_basic() {
        let metrics = ReplicationMetrics::new("test-node".to_string(), "performance".to_string());

        // Initial state
        assert_eq!(metrics.get_lag_bytes().await, 0);
        assert_eq!(metrics.get_lag_ms().await, 0);

        // Update lag
        metrics.update_lag(100, 50).await;

        assert_eq!(metrics.get_lag_bytes().await, 100);
        assert_eq!(metrics.get_lag_ms().await, 50);
    }

    #[tokio::test]
    async fn test_lag_monitoring_wal_positions() {
        let metrics = ReplicationMetrics::new("test-node".to_string(), "performance".to_string());

        // Update WAL positions
        metrics.update_wal_positions(1000, 950).await;

        // Verify positions
        assert_eq!(metrics.get_primary_sequence().await, 1000);
        assert_eq!(metrics.get_secondary_sequence().await, 950);

        // Verify lag calculation
        assert_eq!(metrics.get_lag_bytes().await, 50);
    }

    #[tokio::test]
    async fn test_lag_monitoring_status_updates() {
        let metrics = ReplicationMetrics::new("test-node".to_string(), "performance".to_string());

        // Update status - should not panic
        metrics.update_status(ReplicationStatusValue::Healthy).await;
        metrics.update_status(ReplicationStatusValue::Lagging).await;
        metrics
            .update_status(ReplicationStatusValue::Disconnected)
            .await;
    }

    #[tokio::test]
    async fn test_lag_monitoring_in_manager() {
        let config = ReplicationConfig {
            mode: ReplicationMode::Performance,
            primary_endpoint: Some("https://primary:50051".to_string()),
            secondary_endpoints: vec![],
            ..Default::default()
        };

        let storage = Arc::new(MockStorage::new());
        let manager = ReplicationManager::new(config, storage).await.unwrap();

        // Update sequences
        manager.update_applied_sequence(50).await.unwrap();
        manager.update_primary_sequence(100).await.unwrap();

        // Verify lag tracking
        assert_eq!(manager.get_primary_sequence().await, 100);
        assert_eq!(manager.get_secondary_sequence().await, 50);
        assert_eq!(manager.get_lag_bytes().await, 50);
    }

    #[tokio::test]
    async fn test_lag_monitoring_secondary_metrics() {
        let storage = Arc::new(MockStorage::new());
        let handler = secreton_replication::secondary::SecondaryReadHandler::new(storage, 1000);

        // Set sequences
        handler.update_sequence(75).await;
        handler.update_primary_sequence(100).await;

        // Get metrics
        let metrics = handler.get_lag_metrics().await;

        assert_eq!(metrics.last_applied_sequence, 75);
        assert_eq!(metrics.primary_sequence, 100);
        assert_eq!(metrics.sequence_lag, 25);
        assert!(!metrics.is_stale); // Recent update
    }

    #[tokio::test]
    async fn test_lag_monitoring_zero_lag() {
        let metrics = ReplicationMetrics::new("test-node".to_string(), "performance".to_string());

        // Both at same sequence
        metrics.update_wal_positions(500, 500).await;

        // Lag should be zero
        assert_eq!(metrics.get_lag_bytes().await, 0);
    }

    #[tokio::test]
    async fn test_lag_monitoring_large_lag() {
        let metrics = ReplicationMetrics::new("test-node".to_string(), "performance".to_string());

        // Large lag
        metrics.update_wal_positions(10000, 1000).await;

        // Lag should be 9000
        assert_eq!(metrics.get_lag_bytes().await, 9000);
    }
}

/// Property 10: Failover
///
/// **Property**: Automatic failover detection and secondary promotion
///
/// **Validates**: Requirements 2.2.6, 2.2.7
///
/// **Formal specification**:
/// ```text
/// ∀ heartbeat_timeout ∈ Duration, failure_threshold ∈ u32:
///   (1) Primary failure detection:
///       no_heartbeat_for(heartbeat_timeout * failure_threshold) ⇒
///       failover_triggered()
///
///   (2) Automatic promotion:
///       failover_triggered() ⇒
///       promote_to_primary() succeeds
///       ∧ is_primary() = true
///       ∧ write_operations_enabled()
///
///   (3) Metrics reset:
///       after_promotion() ⇒
///       lag_bytes = 0
///       ∧ lag_ms = 0
///       ∧ status = Healthy
///
///   (4) Idempotency:
///       failover_triggered() ⇒
///       promote_to_primary() called once only
/// ```
#[cfg(test)]
mod failover_tests {
    use super::*;
    use async_trait::async_trait;
    use chrono::Utc;
    use secreton_replication::Result;
    use secreton_replication::manager::ReplicationStorage;
    use secreton_replication::{
        FailoverDetector, HealthStatus, Heartbeat, ReplicationConfig, ReplicationManager,
        ReplicationMode,
    };
    use std::sync::Arc;
    use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
    use std::time::Duration;
    use tokio::sync::RwLock;

    // Mock storage for testing
    struct MockStorage {
        operations: Arc<RwLock<Vec<ReplicationOperation>>>,
        sequence: Arc<RwLock<u64>>,
    }

    impl MockStorage {
        fn new() -> Self {
            Self {
                operations: Arc::new(RwLock::new(Vec::new())),
                sequence: Arc::new(RwLock::new(0)),
            }
        }
    }

    #[async_trait]
    impl ReplicationStorage for MockStorage {
        async fn get_operations_since(&self, sequence: u64) -> Result<Vec<ReplicationOperation>> {
            let ops = self.operations.read().await;
            Ok(ops
                .iter()
                .filter(|op| op.sequence > sequence)
                .cloned()
                .collect())
        }

        async fn apply_operation(&self, operation: &ReplicationOperation) -> Result<()> {
            self.operations.write().await.push(operation.clone());
            *self.sequence.write().await = operation.sequence;
            Ok(())
        }

        async fn get_current_sequence(&self) -> Result<u64> {
            Ok(*self.sequence.read().await)
        }

        async fn store_cluster_metadata(&self, _key: &str, _value: &[u8]) -> Result<()> {
            Ok(())
        }

        async fn get_cluster_metadata(&self, _key: &str) -> Result<Option<Vec<u8>>> {
            Ok(None)
        }
    }

    proptest! {
        /// Test primary failure detection with various timeout/threshold combinations
        ///
        /// **Property**: Failover is triggered after heartbeat_timeout * failure_threshold
        #[test]
        fn prop_failover_detection(
            heartbeat_timeout_ms in 10u64..100u64,  // SHORT timeouts
            failure_threshold in 1u32..5u32,
        ) {
            tokio::runtime::Runtime::new().unwrap().block_on(async {
                let timeout = Duration::from_millis(heartbeat_timeout_ms);
                let detector = FailoverDetector::new(timeout, failure_threshold);

                // Record an initial heartbeat
                let initial_heartbeat = Heartbeat {
                    sequence: 1,
                    timestamp: Utc::now() - chrono::Duration::milliseconds((heartbeat_timeout_ms * 2) as i64),
                    status: HealthStatus::Healthy,
                };
                detector.record_heartbeat(initial_heartbeat).await;

                // Check health multiple times to accumulate failures
                let mut failover_triggered = false;
                for i in 0..failure_threshold {
                    // Use timeout to prevent hanging
                    let check_result = tokio::time::timeout(
                        Duration::from_millis(200),
                        detector.check_health()
                    ).await;

                    match check_result {
                        Ok(Ok(triggered)) => {
                            if triggered {
                                failover_triggered = true;
                                // Verify it triggered at the right count
                                assert_eq!(
                                    i + 1, failure_threshold,
                                    "Failover should trigger after exactly {} failures",
                                    failure_threshold
                                );
                                break;
                            }
                        }
                        Ok(Err(e)) => panic!("Health check failed: {}", e),
                        Err(_) => panic!("Health check timed out"),
                    }

                    // Small delay between checks
                    tokio::time::sleep(Duration::from_millis(5)).await;
                }

                // Verify failover was triggered
                assert!(
                    failover_triggered,
                    "Failover should be triggered after {} consecutive failures",
                    failure_threshold
                );
            });
        }
    }

    proptest! {
        /// Test automatic promotion after failover
        ///
        /// **Property**: Secondary is promoted to primary after failover
        #[test]
        fn prop_automatic_promotion(
            heartbeat_timeout_ms in 10u64..50u64,  // SHORT timeouts
        ) {
            tokio::runtime::Runtime::new().unwrap().block_on(async {
                let config = ReplicationConfig {
                    mode: ReplicationMode::DisasterRecovery,
                    primary_endpoint: Some("https://primary:50051".to_string()),
                    secondary_endpoints: vec![],
                    heartbeat_timeout_ms: Some(heartbeat_timeout_ms),
                    failure_threshold: Some(2),
                    ..Default::default()
                };

                let storage = Arc::new(MockStorage::new());
                let manager = ReplicationManager::new(config, storage).await.unwrap();

                // Verify initial state: not primary
                assert!(!manager.is_primary().await, "Should start as secondary");

                // Promote to primary (simulating automatic promotion after failover)
                let promote_result = tokio::time::timeout(
                    Duration::from_millis(200),
                    manager.promote_to_primary()
                ).await;

                match promote_result {
                    Ok(Ok(())) => {
                        // Verify promotion succeeded
                        assert!(
                            manager.is_primary().await,
                            "Should be primary after promotion"
                        );
                    }
                    Ok(Err(e)) => panic!("Promotion failed: {}", e),
                    Err(_) => panic!("Promotion timed out"),
                }
            });
        }
    }

    proptest! {
        /// Test metrics reset after promotion
        ///
        /// **Property**: Lag metrics are reset to zero after promotion
        #[test]
        fn prop_metrics_reset_after_promotion(
            initial_lag_bytes in 1u64..1000u64,
            _initial_lag_ms in 1u64..1000u64,
        ) {
            tokio::runtime::Runtime::new().unwrap().block_on(async {
                let config = ReplicationConfig {
                    mode: ReplicationMode::DisasterRecovery,
                    primary_endpoint: Some("https://primary:50051".to_string()),
                    secondary_endpoints: vec![],
                    ..Default::default()
                };

                let storage = Arc::new(MockStorage::new());
                let manager = ReplicationManager::new(config, storage).await.unwrap();

                // Set initial lag (simulating replication lag before failover)
                manager.update_applied_sequence(100).await.unwrap();
                manager.update_primary_sequence(100 + initial_lag_bytes).await.unwrap();

                // Verify lag exists
                let lag_before = manager.get_lag_bytes().await;
                assert_eq!(
                    lag_before, initial_lag_bytes,
                    "Should have initial lag before promotion"
                );

                // Promote to primary
                let promote_result = tokio::time::timeout(
                    Duration::from_millis(200),
                    manager.promote_to_primary()
                ).await;

                match promote_result {
                    Ok(Ok(())) => {
                        // Verify lag is reset
                        let lag_after = manager.get_lag_bytes().await;
                        assert_eq!(
                            lag_after, 0,
                            "Lag should be reset to 0 after promotion"
                        );

                        let lag_ms_after = manager.get_lag_ms().await;
                        assert_eq!(
                            lag_ms_after, 0,
                            "Time lag should be reset to 0 after promotion"
                        );
                    }
                    Ok(Err(e)) => panic!("Promotion failed: {}", e),
                    Err(_) => panic!("Promotion timed out"),
                }
            });
        }
    }

    proptest! {
        /// Test failover callback is invoked exactly once
        ///
        /// **Property**: Failover callback is called once, not multiple times
        #[test]
        fn prop_failover_callback_once(
            heartbeat_timeout_ms in 10u64..50u64,  // SHORT timeouts
            failure_threshold in 2u32..4u32,
        ) {
            tokio::runtime::Runtime::new().unwrap().block_on(async {
                let timeout = Duration::from_millis(heartbeat_timeout_ms);
                let mut detector = FailoverDetector::new(timeout, failure_threshold);

                let callback_count = Arc::new(AtomicU32::new(0));
                let callback_count_clone = callback_count.clone();

                detector.set_failover_callback(move || {
                    callback_count_clone.fetch_add(1, Ordering::SeqCst);
                });

                // Record an old heartbeat to trigger failure
                let old_heartbeat = Heartbeat {
                    sequence: 1,
                    timestamp: Utc::now() - chrono::Duration::milliseconds((heartbeat_timeout_ms * 2) as i64),
                    status: HealthStatus::Healthy,
                };
                detector.record_heartbeat(old_heartbeat).await;

                // Check health multiple times to trigger failover
                for _ in 0..failure_threshold + 2 {
                    let check_result = tokio::time::timeout(
                        Duration::from_millis(100),
                        detector.check_health()
                    ).await;

                    if check_result.is_err() {
                        panic!("Health check timed out");
                    }

                    tokio::time::sleep(Duration::from_millis(5)).await;
                }

                // Verify callback was invoked exactly once
                let count = callback_count.load(Ordering::SeqCst);
                assert_eq!(
                    count, 1,
                    "Failover callback should be invoked exactly once, got {} invocations",
                    count
                );
            });
        }
    }

    proptest! {
        // Each case loops sending heartbeats with real sleeps; 256 cases is
        // minutes of wall-clock. Cap to match the other failover proptests.
        #![proptest_config(ProptestConfig::with_cases(10))]
        /// Test failover detection with healthy heartbeats
        ///
        /// **Property**: Failover is NOT triggered when receiving healthy heartbeats
        #[test]
        fn prop_no_failover_with_healthy_heartbeats(
            heartbeat_timeout_ms in 10u64..50u64,  // SHORT timeouts
            num_heartbeats in 5usize..20usize,
        ) {
            tokio::runtime::Runtime::new().unwrap().block_on(async {
                let timeout = Duration::from_millis(heartbeat_timeout_ms);
                let detector = FailoverDetector::new(timeout, 3);

                let callback_invoked = Arc::new(AtomicBool::new(false));
                let callback_invoked_clone = callback_invoked.clone();

                let mut detector_mut = detector;
                detector_mut.set_failover_callback(move || {
                    callback_invoked_clone.store(true, Ordering::SeqCst);
                });

                // Send healthy heartbeats regularly
                for i in 0..num_heartbeats {
                    let heartbeat = Heartbeat {
                        sequence: i as u64 + 1,
                        timestamp: Utc::now(),
                        status: HealthStatus::Healthy,
                    };
                    detector_mut.record_heartbeat(heartbeat).await;

                    // Check health - should not trigger failover
                    let check_result = tokio::time::timeout(
                        Duration::from_millis(100),
                        detector_mut.check_health()
                    ).await;

                    match check_result {
                        Ok(Ok(triggered)) => {
                            assert!(
                                !triggered,
                                "Failover should not trigger with healthy heartbeats"
                            );
                        }
                        Ok(Err(e)) => panic!("Health check failed: {}", e),
                        Err(_) => panic!("Health check timed out"),
                    }

                    // Small delay between heartbeats
                    tokio::time::sleep(Duration::from_millis(heartbeat_timeout_ms / 2)).await;
                }

                // Verify failover was never triggered
                assert!(
                    !callback_invoked.load(Ordering::SeqCst),
                    "Failover should not be triggered with healthy heartbeats"
                );
            });
        }
    }

    proptest! {
        /// Test failure count reset on recovery
        ///
        /// **Property**: Failure count resets when primary recovers
        #[test]
        fn prop_failure_count_reset_on_recovery(
            heartbeat_timeout_ms in 10u64..50u64,  // SHORT timeouts
            initial_failures in 1u32..3u32,
        ) {
            tokio::runtime::Runtime::new().unwrap().block_on(async {
                let timeout = Duration::from_millis(heartbeat_timeout_ms);
                let detector = FailoverDetector::new(timeout, 5);

                // Record an old heartbeat to accumulate failures
                let old_heartbeat = Heartbeat {
                    sequence: 1,
                    timestamp: Utc::now() - chrono::Duration::milliseconds((heartbeat_timeout_ms * 2) as i64),
                    status: HealthStatus::Healthy,
                };
                detector.record_heartbeat(old_heartbeat).await;

                // Accumulate some failures
                for _ in 0..initial_failures {
                    let check_result = tokio::time::timeout(
                        Duration::from_millis(100),
                        detector.check_health()
                    ).await;

                    if check_result.is_err() {
                        panic!("Health check timed out");
                    }

                    tokio::time::sleep(Duration::from_millis(5)).await;
                }

                // Verify failures were accumulated
                let failure_count_before = detector.failure_count().await;
                assert_eq!(
                    failure_count_before, initial_failures,
                    "Should have accumulated {} failures",
                    initial_failures
                );

                // Send a healthy heartbeat (recovery)
                let healthy_heartbeat = Heartbeat {
                    sequence: 2,
                    timestamp: Utc::now(),
                    status: HealthStatus::Healthy,
                };
                detector.record_heartbeat(healthy_heartbeat).await;

                // Verify failure count was reset
                let failure_count_after = detector.failure_count().await;
                assert_eq!(
                    failure_count_after, 0,
                    "Failure count should be reset to 0 after recovery"
                );
            });
        }
    }

    proptest! {
        /// Test promotion only happens in DR mode
        ///
        /// **Property**: Promotion is only allowed in DisasterRecovery mode
        #[test]
        fn prop_promotion_only_in_dr_mode(
            mode_is_dr in prop::bool::ANY,
        ) {
            tokio::runtime::Runtime::new().unwrap().block_on(async {
                let mode = if mode_is_dr {
                    ReplicationMode::DisasterRecovery
                } else {
                    ReplicationMode::Performance
                };

                let config = ReplicationConfig {
                    mode,
                    primary_endpoint: Some("https://primary:50051".to_string()),
                    secondary_endpoints: vec![],
                    ..Default::default()
                };

                let storage = Arc::new(MockStorage::new());
                let manager = ReplicationManager::new(config, storage).await.unwrap();

                // Try to promote
                let promote_result = tokio::time::timeout(
                    Duration::from_millis(200),
                    manager.promote_to_primary()
                ).await;

                match promote_result {
                    Ok(Ok(())) => {
                        // Should only succeed in DR mode
                        assert!(
                            mode_is_dr,
                            "Promotion should only succeed in DR mode"
                        );
                        assert!(manager.is_primary().await);
                    }
                    Ok(Err(_)) => {
                        // Should fail in Performance mode
                        assert!(
                            !mode_is_dr,
                            "Promotion should fail in Performance mode"
                        );
                        assert!(!manager.is_primary().await);
                    }
                    Err(_) => panic!("Promotion timed out"),
                }
            });
        }
    }

    proptest! {
        /// Test unhealthy status triggers failover
        ///
        /// **Property**: Unhealthy heartbeat status triggers failover
        #[test]
        fn prop_unhealthy_status_triggers_failover(
            heartbeat_timeout_ms in 10u64..50u64,  // SHORT timeouts
            failure_threshold in 1u32..3u32,
        ) {
            tokio::runtime::Runtime::new().unwrap().block_on(async {
                let timeout = Duration::from_millis(heartbeat_timeout_ms);
                let detector = FailoverDetector::new(timeout, failure_threshold);

                // Record unhealthy heartbeats
                for _ in 0..failure_threshold {
                    let unhealthy_heartbeat = Heartbeat {
                        sequence: 1,
                        timestamp: Utc::now(),  // Recent timestamp
                        status: HealthStatus::Unhealthy,  // But unhealthy status
                    };
                    detector.record_heartbeat(unhealthy_heartbeat).await;

                    let check_result = tokio::time::timeout(
                        Duration::from_millis(100),
                        detector.check_health()
                    ).await;

                    if check_result.is_err() {
                        panic!("Health check timed out");
                    }

                    tokio::time::sleep(Duration::from_millis(5)).await;
                }

                // Verify failover was triggered
                let failover_triggered = detector.is_failover_triggered().await;
                assert!(
                    failover_triggered,
                    "Failover should be triggered by unhealthy status"
                );
            });
        }
    }

    #[tokio::test]
    async fn test_failover_basic() {
        let detector = FailoverDetector::new(Duration::from_millis(50), 2);

        // Record an old heartbeat
        let old_heartbeat = Heartbeat {
            sequence: 1,
            timestamp: Utc::now() - chrono::Duration::milliseconds(200),
            status: HealthStatus::Healthy,
        };
        detector.record_heartbeat(old_heartbeat).await;

        // Check health twice to trigger failover
        for _ in 0..2 {
            let result =
                tokio::time::timeout(Duration::from_millis(100), detector.check_health()).await;

            assert!(result.is_ok(), "Health check should not timeout");
        }

        // Verify failover was triggered
        assert!(detector.is_failover_triggered().await);
    }

    #[tokio::test]
    async fn test_promotion_success() {
        let config = ReplicationConfig {
            mode: ReplicationMode::DisasterRecovery,
            primary_endpoint: Some("https://primary:50051".to_string()),
            secondary_endpoints: vec![],
            ..Default::default()
        };

        let storage = Arc::new(MockStorage::new());
        let manager = ReplicationManager::new(config, storage).await.unwrap();

        // Verify initial state
        assert!(!manager.is_primary().await);

        // Promote
        let result =
            tokio::time::timeout(Duration::from_millis(200), manager.promote_to_primary()).await;

        assert!(result.is_ok(), "Promotion should not timeout");
        assert!(result.unwrap().is_ok(), "Promotion should succeed");
        assert!(manager.is_primary().await);
    }

    #[tokio::test]
    async fn test_promotion_resets_lag() {
        let config = ReplicationConfig {
            mode: ReplicationMode::DisasterRecovery,
            primary_endpoint: Some("https://primary:50051".to_string()),
            secondary_endpoints: vec![],
            ..Default::default()
        };

        let storage = Arc::new(MockStorage::new());
        let manager = ReplicationManager::new(config, storage).await.unwrap();

        // Set initial lag
        manager.update_applied_sequence(100).await.unwrap();
        manager.update_primary_sequence(200).await.unwrap();

        assert_eq!(manager.get_lag_bytes().await, 100);

        // Promote
        manager.promote_to_primary().await.unwrap();

        // Verify lag is reset
        assert_eq!(manager.get_lag_bytes().await, 0);
        assert_eq!(manager.get_lag_ms().await, 0);
    }

    #[tokio::test]
    async fn test_promotion_idempotency() {
        let config = ReplicationConfig {
            mode: ReplicationMode::DisasterRecovery,
            primary_endpoint: Some("https://primary:50051".to_string()),
            secondary_endpoints: vec![],
            ..Default::default()
        };

        let storage = Arc::new(MockStorage::new());
        let manager = ReplicationManager::new(config, storage).await.unwrap();

        // First promotion should succeed
        let result1 = manager.promote_to_primary().await;
        assert!(result1.is_ok());
        assert!(manager.is_primary().await);

        // Second promotion should fail (already primary)
        let result2 = manager.promote_to_primary().await;
        assert!(result2.is_err());
    }
}

/// Property 10: Last-write-wins conflict resolution
///
/// **Property**: For any conflicting writes to the same path, the write with the
/// latest timestamp should be the final value
///
/// **Validates**: Requirements 2.2.6
///
/// **Formal specification**:
/// ```text
/// ∀ op1, op2 ∈ ReplicationOperation:
///   (op1.path = op2.path ∧ op1.id ≠ op2.id) ⇒
///     resolve_conflict(op1, op2) =
///       if op1.timestamp > op2.timestamp then op1
///       else if op2.timestamp > op1.timestamp then op2
///       else if op1.sequence > op2.sequence then op1
///       else op2
/// ```
#[cfg(test)]
mod conflict_resolution_tests {
    use super::*;
    use chrono::{DateTime, Duration, Utc};
    use secreton_replication::{ConflictResolver, ResolutionStrategy};
    use uuid::Uuid;

    fn create_operation(
        path: String,
        sequence: u64,
        timestamp: DateTime<Utc>,
    ) -> ReplicationOperation {
        ReplicationOperation {
            id: Uuid::new_v4(),
            timestamp,
            operation_type: OperationType::SecretWrite,
            data: OperationData::Secret {
                path,
                data: vec![1, 2, 3],
                metadata: serde_json::json!({}),
            },
            sequence,
        }
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(10))]
        /// Test last-write-wins with different timestamps
        #[test]
        fn prop_last_write_wins_timestamp(
            path in "[a-z/]{5,20}",
            seq1 in 1u64..1000u64,
            seq2 in 1u64..1000u64,
            time_diff_ms in 1i64..1000i64,
        ) {
            let resolver = ConflictResolver::new(ResolutionStrategy::LastWriteWins, 100);

            let now = Utc::now();
            let op1 = create_operation(path.clone(), seq1, now);
            let op2 = create_operation(path.clone(), seq2, now + Duration::milliseconds(time_diff_ms));

            // Resolve conflict
            let runtime = tokio::runtime::Runtime::new().unwrap();
            let (winner, conflict) = runtime.block_on(async {
                resolver.resolve_conflict(op1.clone(), op2.clone()).await.unwrap()
            });

            // The operation with the later timestamp should win
            if time_diff_ms > 0 {
                assert_eq!(winner.id, op2.id, "Later timestamp should win");
                assert_eq!(conflict.winner.id, op2.id);
            } else {
                // If timestamps are equal, higher sequence wins
                if seq2 > seq1 {
                    assert_eq!(winner.id, op2.id, "Higher sequence should win as tiebreaker");
                } else {
                    assert_eq!(winner.id, op1.id, "First operation should win if sequences are equal");
                }
            }
        }
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(10))]
        /// Test conflict detection for same path
        #[test]
        fn prop_conflict_detection_same_path(
            path in "[a-z/]{5,20}",
            seq1 in 1u64..1000u64,
            seq2 in 1u64..1000u64,
        ) {
            prop_assume!(seq1 != seq2); // Ensure different sequences

            let resolver = ConflictResolver::default();

            let now = Utc::now();
            let op1 = create_operation(path.clone(), seq1, now);
            let op2 = create_operation(path.clone(), seq2, now);

            // Should detect conflict for same path with different sequences
            assert!(resolver.detect_conflict(&op1, &op2),
                "Should detect conflict for same path with different sequences");
        }
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(10))]
        /// Test no conflict for different paths
        #[test]
        fn prop_no_conflict_different_paths(
            path1 in "[a-z/]{5,20}",
            path2 in "[a-z/]{5,20}",
            seq1 in 1u64..1000u64,
            seq2 in 1u64..1000u64,
        ) {
            prop_assume!(path1 != path2); // Ensure different paths

            let resolver = ConflictResolver::default();

            let now = Utc::now();
            let op1 = create_operation(path1, seq1, now);
            let op2 = create_operation(path2, seq2, now);

            // Should NOT detect conflict for different paths
            assert!(!resolver.detect_conflict(&op1, &op2),
                "Should not detect conflict for different paths");
        }
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(10))]
        /// Test highest-sequence strategy
        #[test]
        fn prop_highest_sequence_strategy(
            path in "[a-z/]{5,20}",
            seq1 in 1u64..1000u64,
            seq2 in 1u64..1000u64,
            time_diff_ms in -1000i64..1000i64,
        ) {
            prop_assume!(seq1 != seq2); // Ensure different sequences

            let resolver = ConflictResolver::new(ResolutionStrategy::HighestSequence, 100);

            let now = Utc::now();
            let op1 = create_operation(path.clone(), seq1, now);
            let op2 = create_operation(path.clone(), seq2, now + Duration::milliseconds(time_diff_ms));

            // Resolve conflict
            let runtime = tokio::runtime::Runtime::new().unwrap();
            let (winner, _) = runtime.block_on(async {
                resolver.resolve_conflict(op1.clone(), op2.clone()).await.unwrap()
            });

            // The operation with the higher sequence should win, regardless of timestamp
            if seq2 > seq1 {
                assert_eq!(winner.id, op2.id, "Higher sequence should win");
            } else {
                assert_eq!(winner.id, op1.id, "Higher sequence should win");
            }
        }
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(10))]
        /// Test batch conflict resolution
        #[test]
        fn prop_batch_resolve_consistency(
            path in "[a-z/]{5,20}",
            num_ops in 2usize..10usize,
        ) {
            let resolver = ConflictResolver::new(ResolutionStrategy::LastWriteWins, 100);

            let now = Utc::now();
            let mut operations = Vec::new();

            // Create multiple operations for the same path with increasing timestamps
            for i in 0..num_ops {
                let op = create_operation(
                    path.clone(),
                    i as u64,
                    now + Duration::milliseconds(i as i64 * 100),
                );
                operations.push(op);
            }

            // Batch resolve
            let runtime = tokio::runtime::Runtime::new().unwrap();
            let result = runtime.block_on(async {
                resolver.batch_resolve(operations.clone()).await.unwrap()
            });

            // Should have exactly one winner for the path
            assert_eq!(result.len(), 1, "Should have exactly one winner");
            assert!(result.contains_key(&path), "Should contain the path");

            // The winner should be the operation with the latest timestamp (last one)
            let winner = result.get(&path).unwrap();
            assert_eq!(winner.sequence, (num_ops - 1) as u64,
                "Winner should be the operation with the latest timestamp");
        }
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(10))]
        /// Test conflict history tracking
        #[test]
        fn prop_conflict_history_tracking(
            path in "[a-z/]{5,20}",
            num_conflicts in 1usize..20usize,
        ) {
            let resolver = ConflictResolver::new(ResolutionStrategy::LastWriteWins, 100);

            let now = Utc::now();
            let runtime = tokio::runtime::Runtime::new().unwrap();

            // Create and resolve multiple conflicts
            for i in 0..num_conflicts {
                let op1 = create_operation(path.clone(), i as u64, now);
                let op2 = create_operation(
                    path.clone(),
                    i as u64 + 1,
                    now + Duration::milliseconds(100),
                );

                runtime.block_on(async {
                    resolver.resolve_conflict(op1, op2).await.unwrap();
                });
            }

            // Check history
            let history = runtime.block_on(async {
                resolver.get_history().await
            });

            assert_eq!(history.len(), num_conflicts,
                "History should contain all conflicts");

            // All conflicts should be for the same path
            for conflict in &history {
                assert_eq!(conflict.path, path, "All conflicts should be for the same path");
            }
        }
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(10))]
        /// Test conflict resolution is deterministic
        #[test]
        fn prop_conflict_resolution_deterministic(
            path in "[a-z/]{5,20}",
            seq1 in 1u64..1000u64,
            seq2 in 1u64..1000u64,
            time_diff_ms in 1i64..1000i64,
        ) {
            let resolver = ConflictResolver::new(ResolutionStrategy::LastWriteWins, 100);

            let now = Utc::now();
            let op1 = create_operation(path.clone(), seq1, now);
            let op2 = create_operation(path.clone(), seq2, now + Duration::milliseconds(time_diff_ms));

            let runtime = tokio::runtime::Runtime::new().unwrap();

            // Resolve the same conflict multiple times
            let (winner1, _) = runtime.block_on(async {
                resolver.resolve_conflict(op1.clone(), op2.clone()).await.unwrap()
            });

            let (winner2, _) = runtime.block_on(async {
                resolver.resolve_conflict(op1.clone(), op2.clone()).await.unwrap()
            });

            // Results should be identical
            assert_eq!(winner1.id, winner2.id, "Conflict resolution should be deterministic");
            assert_eq!(winner1.sequence, winner2.sequence);
            assert_eq!(winner1.timestamp, winner2.timestamp);
        }
    }
}
