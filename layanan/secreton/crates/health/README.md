# secreton-health

Comprehensive health check system for Secreton.

## Overview

This crate provides a flexible and extensible health check infrastructure that supports:

- **Multiple health check implementations** via the `HealthCheck` trait
- **Centralized health check registry** for managing all checks
- **Different health status levels**: Healthy, Degraded, Unhealthy
- **Detailed health check results** with timing and metadata
- **Critical vs non-critical checks** for nuanced status reporting
- **Timeout support** for individual health checks
- **Tag-based filtering** for selective health check execution

## Usage

### Basic Example

```rust
use secreton_health::{HealthCheck, HealthCheckRegistry, HealthCheckResult, HealthStatus};
use async_trait::async_trait;

// Define a custom health check
struct DatabaseHealthCheck;

#[async_trait]
impl HealthCheck for DatabaseHealthCheck {
    fn name(&self) -> &str {
        "database"
    }

    async fn check(&self) -> HealthCheckResult {
        // Perform database health check
        match check_database_connection().await {
            Ok(_) => HealthCheckResult::healthy_with_message("Database connected"),
            Err(e) => HealthCheckResult::unhealthy(format!("Database error: {}", e)),
        }
    }
}

#[tokio::main]
async fn main() {
    // Create registry and register checks
    let mut registry = HealthCheckRegistry::new();
    registry.register(Box::new(DatabaseHealthCheck)).await.unwrap();

    // Execute all health checks
    let results = registry.check_all().await;

    println!("Overall status: {:?}", results.overall_status());
    println!("Total checks: {}", results.check_count());

    // Check individual results
    for (name, result) in &results.checks {
        println!("{}: {:?} ({}ms)", name, result.status, result.response_time_ms);
    }
}
```

### Health Status Levels

The `HealthStatus` enum provides three levels:

- **Healthy**: Component is fully operational
- **Degraded**: Component is operational but with reduced functionality
- **Unhealthy**: Component is not operational

```rust
use secreton_health::HealthStatus;

let status = HealthStatus::Healthy;
assert!(status.is_healthy());
assert_eq!(status.severity(), 0);

// Compare statuses
let worst = HealthStatus::Healthy.worst(&HealthStatus::Degraded);
assert_eq!(worst, HealthStatus::Degraded);
```

### Critical vs Non-Critical Checks

Health checks can be marked as critical or non-critical:

```rust
use secreton_health::HealthCheck;
use async_trait::async_trait;

struct MetricsHealthCheck;

#[async_trait]
impl HealthCheck for MetricsHealthCheck {
    fn name(&self) -> &str {
        "metrics"
    }

    async fn check(&self) -> HealthCheckResult {
        // ... check implementation
    }

    // Mark as non-critical
    fn is_critical(&self) -> bool {
        false
    }
}
```

**Overall status calculation:**

- If any **critical** check is Unhealthy → overall is Unhealthy
- If any check is Degraded → overall is Degraded
- If a **non-critical** check is Unhealthy → overall is Degraded (not Unhealthy)
- Otherwise → overall is Healthy

### Timeout Support

Health checks have a default timeout of 5 seconds:

```rust
use secreton_health::HealthCheckRegistry;

// Create registry with custom timeout (2 seconds)
let mut registry = HealthCheckRegistry::with_timeout(2000);

// Or set timeout after creation
registry.set_default_timeout(3000);
```

### Tag-Based Filtering

Health checks can be tagged for selective execution:

```rust
use secreton_health::HealthCheck;
use async_trait::async_trait;

struct KubernetesHealthCheck;

#[async_trait]
impl HealthCheck for KubernetesHealthCheck {
    fn name(&self) -> &str {
        "kubernetes"
    }

    async fn check(&self) -> HealthCheckResult {
        // ... check implementation
    }

    fn tags(&self) -> Vec<String> {
        vec!["kubernetes".to_string(), "probe".to_string()]
    }
}

// Execute only checks with specific tags
let results = registry.check_by_tags(&vec!["kubernetes".to_string()]).await;
```

### Detailed Results

Health check results can include additional details:

```rust
use secreton_health::HealthCheckResult;
use std::collections::HashMap;

let result = HealthCheckResult::healthy()
    .with_response_time(15)
    .add_detail("connections", serde_json::json!(10))
    .add_detail("latency_ms", serde_json::json!(5))
    .add_detail("version", serde_json::json!("1.0.0"));
```

## Integration with Kubernetes

This health check system is designed to integrate with Kubernetes health probes:

### Liveness Probe

```rust
// Simple check - is the process alive?
pub async fn liveness_check() -> Result<Json<LivenessResponse>, ApiError> {
    Ok(Json(LivenessResponse {
        alive: true,
        uptime: get_uptime_seconds(),
        timestamp: Utc::now(),
    }))
}
```

### Readiness Probe

```rust
// Comprehensive check - is the service ready to accept traffic?
pub async fn readiness_check(registry: &HealthCheckRegistry) -> Result<Json<ReadinessResponse>, ApiError> {
    let results = registry.check_all().await;

    if !results.is_healthy() {
        return Err(ApiError::ServiceUnavailable {
            message: "Service not ready".to_string(),
        });
    }

    Ok(Json(ReadinessResponse {
        ready: true,
        version: env!("CARGO_PKG_VERSION").to_string(),
        checks: results.checks,
        timestamp: Utc::now(),
    }))
}
```

### Startup Probe

```rust
// Check if initialization is complete
pub async fn startup_check(registry: &HealthCheckRegistry) -> Result<Json<StartupResponse>, ApiError> {
    let results = registry.check_by_tags(&vec!["startup".to_string()]).await;

    Ok(Json(StartupResponse {
        started: results.is_healthy(),
        timestamp: Utc::now(),
    }))
}
```

## Architecture

The health check system consists of:

1. **`HealthCheck` trait**: Interface for implementing custom health checks
2. **`HealthStatus` enum**: Three-level status (Healthy, Degraded, Unhealthy)
3. **`HealthCheckResult`**: Result of a single health check with timing and details
4. **`HealthCheckResults`**: Collection of results with overall status calculation
5. **`HealthCheckRegistry`**: Central registry for managing and executing checks
6. **`HealthCheckError`**: Error types for health check operations

## Best Practices

1. **Keep checks fast**: Health checks should complete quickly (< 1 second)
2. **Use appropriate status levels**: Reserve Unhealthy for truly broken components
3. **Mark checks as critical appropriately**: Only critical infrastructure should be marked critical
4. **Include useful details**: Add context to help diagnose issues
5. **Use tags for organization**: Group related checks with tags
6. **Set reasonable timeouts**: Default 5 seconds is usually sufficient

## License

This crate is part of the Secreton project.

## Built-in Health Checks

### Seal Status Health Check

The seal status health check monitors whether Secreton is sealed or unsealed:

- **Unhealthy** when Secreton is sealed (cannot serve requests)
- **Healthy** when Secreton is unsealed (operational)

This is a critical health check that affects Kubernetes readiness probes. When sealed, the pod should not receive traffic until it is unsealed.

**Tags**: `seal`, `security`, `kubernetes`, `readiness`

#### Usage Example

```rust
use secreton_health::{HealthCheckRegistry, checks::SealStatusHealthCheck};
use secreton_core::services::seal::SealService;
use std::sync::Arc;

#[tokio::main]
async fn main() {
    // Create seal service
    let seal_service = Arc::new(SealService::new(Default::default()));

    // Create seal status health check
    let seal_check = SealStatusHealthCheck::new(seal_service);

    // Register with health check registry
    let mut registry = HealthCheckRegistry::new();
    registry.register(Box::new(seal_check)).await.unwrap();

    // Execute all health checks
    let results = registry.check_all().await;

    if results.is_unhealthy() {
        println!("System is unhealthy - Secreton may be sealed");
    }
}
```

#### Kubernetes Integration

The seal status check is designed for Kubernetes readiness probes:

```yaml
apiVersion: v1
kind: Pod
metadata:
  name: secreton
spec:
  containers:
  - name: secreton
    image: secreton:latest
    readinessProbe:
      httpGet:
        path: /health/ready
        port: 8200
      initialDelaySeconds: 5
      periodSeconds: 5
      failureThreshold: 3
```

When Secreton is sealed, the readiness probe will fail, preventing traffic from being routed to the pod until it is unsealed.

#### Implementation Details

The seal status health check:

- Checks the seal status from Secreton's core state
- Returns detailed information including seal status in the response
- Is marked as a critical check (affects overall system health)
- Has minimal overhead (< 1ms response time)
- Supports state transitions (sealed ↔ unsealed)

**Validates: Requirements 2.6.2** - Health check system must report seal status

### Raft Cluster Health Check

The Raft cluster health check monitors the health of the Raft consensus cluster:

- **Unhealthy** when Raft is completely unavailable
- **Degraded** when no leader is elected (cluster cannot serve writes)
- **Degraded** when peers have high replication lag
- **Healthy** when cluster is operational with a leader

This is a critical health check that affects Kubernetes readiness probes. When degraded or unhealthy, the pod should not receive write traffic.

**Tags**: `raft`, `cluster`, `replication`, `kubernetes`, `readiness`

#### Usage Example

```rust
use secreton_health::{HealthCheckRegistry, checks::RaftClusterHealthCheck};
use secreton_storage::raft::RaftCluster;
use std::sync::Arc;

#[tokio::main]
async fn main() {
    // Create Raft cluster
    let raft_cluster = Arc::new(RaftCluster::new(Default::default()).await.unwrap());

    // Create Raft cluster health check
    let raft_check = RaftClusterHealthCheck::new(raft_cluster)
        .with_max_lag(1000); // Optional: set custom lag threshold

    // Register with health check registry
    let mut registry = HealthCheckRegistry::new();
    registry.register(Box::new(raft_check)).await.unwrap();

    // Execute all health checks
    let results = registry.check_all().await;

    if results.is_degraded() {
        println!("Raft cluster is degraded - may be electing leader or have high lag");
    }
}
```

#### Kubernetes Integration

The Raft cluster check is designed for Kubernetes readiness probes:

```yaml
apiVersion: v1
kind: Pod
metadata:
  name: secreton
spec:
  containers:
  - name: secreton
    image: secreton:latest
    readinessProbe:
      httpGet:
        path: /health/ready
        port: 8200
      initialDelaySeconds: 10
      periodSeconds: 5
      failureThreshold: 3
```

When the Raft cluster has no leader or high replication lag, the readiness probe will fail, preventing write traffic from being routed to the pod.

#### Health Status Conditions

The Raft cluster health check evaluates the following conditions:

1. **Raft Availability**: Can the Raft cluster be queried?
   - If NO → **Unhealthy**

2. **Leader Election**: Is there a current leader?
   - If NO → **Degraded** (leader election in progress)

3. **Replication Lag**: Are peers keeping up with the leader?
   - If any peer has lag > threshold → **Degraded**
   - Default threshold: 1000 log entries

4. **All checks pass** → **Healthy**

#### Implementation Details

The Raft cluster health check:

- Checks leader status from Raft metrics
- Monitors peer connectivity and replication lag
- Returns detailed information including leader ID, peer count, and lag metrics
- Is marked as a critical check (affects overall system health)
- Has minimal overhead (< 10ms response time)
- Supports configurable lag thresholds

**Validates: Requirements 2.6.3** - Health check system must report Raft cluster status
