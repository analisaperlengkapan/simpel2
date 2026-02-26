# Secreton Replication

This crate provides replication capabilities for Secreton, enabling multi-region deployments and disaster recovery.

## Features

- **Performance Replication**: Read-only replicas for geographic distribution
- **Disaster Recovery Replication**: Full cluster failover capability
- **Replication Lag Monitoring**: Track and expose replication metrics
- **Automatic Failover**: Promote secondary to primary on failure
- **Namespace Filtering**: Selective replication by namespace
- **mTLS Communication**: Secure replication traffic

## Replication Modes

### Performance Replication

Optimized for geographic distribution and load balancing:

- Secondary nodes serve read-only requests
- Write requests are forwarded to primary
- Replicates: secrets, policies, configuration
- Does NOT replicate: tokens, leases, ephemeral state
- Lower consistency guarantees (eventual consistency)

### Disaster Recovery (DR) Replication

Full disaster recovery capability:

- Secondary nodes remain sealed until promoted
- Replicates ALL data including ephemeral state
- Replicates: secrets, policies, tokens, leases, audit logs
- Can be promoted to primary in case of disaster
- Higher consistency guarantees (strong consistency)

## Usage

### Primary Node Configuration

```toml
[replication]
mode = "performance"
secondary_endpoints = [
    "https://secondary1.example.com:50051",
    "https://secondary2.example.com:50051"
]
max_lag_ms = 100
batch_size = 100
interval_ms = 100
auto_failover = true

[replication.tls]
ca_cert = "/path/to/ca.crt"
client_cert = "/path/to/client.crt"
client_key = "/path/to/client.key"
```

### Secondary Node Configuration

```toml
[replication]
mode = "performance"
primary_endpoint = "https://primary.example.com:50051"
max_lag_ms = 100

[replication.tls]
ca_cert = "/path/to/ca.crt"
client_cert = "/path/to/client.crt"
client_key = "/path/to/client.key"
```

### Code Example

```rust
use secreton_replication::{ReplicationManager, ReplicationConfig, ReplicationMode};
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load configuration
    let config = ReplicationConfig {
        mode: ReplicationMode::Performance,
        primary_endpoint: None,
        secondary_endpoints: vec![
            "https://secondary:50051".to_string()
        ],
        ..Default::default()
    };

    // Create storage backend
    let storage = Arc::new(MyStorageBackend::new());

    // Create replication manager
    let manager = ReplicationManager::new(config, storage).await?;

    // Start replication
    manager.start().await?;

    // Add a secondary node dynamically
    let node_id = manager.add_secondary(
        "https://new-secondary:50051".to_string()
    ).await?;

    // Check replication lag
    let lag = manager.get_lag().await;
    println!("Current replication lag: {:?}", lag);

    // Get list of secondaries
    let secondaries = manager.get_secondaries().await;
    for node in secondaries {
        println!("Secondary: {} (status: {})", node.id, node.status);
    }

    Ok(())
}
```

## Architecture

```
Primary Cluster                    Secondary Cluster
┌──────────────┐                  ┌──────────────┐
│ Write Ops    │  ─────gRPC────▶  │ Replicate    │
│              │   (mTLS)         │ & Apply      │
└──────┬───────┘                  └──────┬───────┘
       │                                 │
       ▼                                 ▼
┌──────────────┐                  ┌──────────────┐
│ PostgreSQL   │                  │ PostgreSQL   │
│ (Primary)    │                  │ (Secondary)  │
└──────────────┘                  └──────────────┘
```

## Replication Flow

1. **Write Operation**: Client writes to primary
2. **Raft Commit**: Operation committed to Raft log
3. **PostgreSQL Persist**: Operation persisted to PostgreSQL
4. **Replication Stream**: Operation sent to secondaries via gRPC
5. **Secondary Apply**: Secondary applies operation to local storage
6. **Read Requests**: Clients read from nearest secondary

## Monitoring

The replication manager exposes the following metrics:

- `secreton_replication_lag_ms`: Current replication lag in milliseconds
- `secreton_replication_operations_total`: Total operations replicated
- `secreton_replication_errors_total`: Total replication errors
- `secreton_replication_secondary_status`: Status of each secondary node

## Failover

### Automatic Failover

When `auto_failover = true`, the replication manager automatically promotes a secondary to primary if:

1. Primary becomes unreachable
2. Failover timeout expires (default: 30 seconds)
3. Secondary is healthy and up-to-date

### Manual Failover

```bash
# Promote secondary to primary
secreton replication promote

# Check replication status
secreton replication status
```

## Testing

```bash
# Run unit tests
cargo test -p secreton-replication

# Run integration tests
cargo test -p secreton-replication --test integration
```

## Requirements

- Requirement 2.2.1: Primary cluster can designate secondary clusters
- Requirement 2.2.2: Read operations served by local replicas
- Requirement 2.2.3: Write operations forwarded to primary
- Requirement 2.2.4: Replication lag monitoring
- Requirement 2.2.5: Replication pause/resume
- Requirement 2.2.6: Automatic failover
- Requirement 2.2.7: Conflict resolution (last-write-wins)
- Requirement 2.2.8: Namespace filtering
- Requirement 2.2.9: mTLS for replication traffic
- Requirement 2.2.10: CLI and API for replication management

## License

See the main Secreton LICENSE file.
