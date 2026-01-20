# Authenc Performance Optimization Guide

## 🎯 Performance Goals

| Metric | Target | Current |
|--------|--------|---------|
| Throughput | 5,000+ req/sec | TBD (load test) |
| Latency (p50) | <50ms | TBD |
| Latency (p99) | <100ms | TBD |
| Memory | <500MB | ~300MB |
| CPU | 2-4 cores | Scalable |

## 1. Build-Time Optimizations

### Release Profile Configuration
Already optimized in `Cargo.toml`:
```toml
[profile.release]
opt-level = 3           # Maximum optimization
lto = "fat"             # Full Link-Time Optimization
codegen-units = 1       # Single codegen unit for best optimization
panic = "abort"         # Smaller binary, faster panics
strip = true            # Strip symbols for smaller binary
overflow-checks = false # Disable overflow checks
```

### Binary Size
- **Stripped Release Binary**: ~50-100MB
- **Optimizations Applied**: LTO, strip, panic=abort
- **Compile Time**: ~8-10 minutes (one-time)

## 2. Runtime Optimizations

### Connection Pooling

#### Database Connection Pool
```toml
[database]
max_connections = 50      # Peak connections
min_connections = 10      # Minimum idle connections
connection_timeout = 30   # Timeout in seconds
idle_timeout = 600        # Close idle after 10 min
max_lifetime = 1800       # Recycle after 30 min
```

**Tuning Formula**:
```
max_connections = (num_cpus * 2) + 4
min_connections = num_cpus
```

#### Redis Connection Pool
```toml
[redis]
pool_size = 32
connection_timeout = 5
```

### Request Handling

#### Worker Threads
```toml
[server]
workers = 8  # Set to number of CPU cores
```

#### Keep-Alive
```toml
[server]
keep_alive = 75              # Seconds
client_timeout = 30          # Request timeout
client_disconnect_timeout = 5 # Graceful shutdown
```

### Caching Strategy

#### Multi-Layer Cache
1. **L1 Cache**: In-memory (fast, limited size)
2. **L2 Cache**: Redis (distributed, larger)
3. **L3 Cache**: Database (persistent)

#### Cache Invalidation
- TTL-based expiration
- Event-driven invalidation
- Manual invalidation via API

#### Recommended TTLs
```
Session tokens: 3600s (1 hour)
User profiles: 300s (5 minutes)
RBAC policies: 600s (10 minutes)
MFA verification: 300s (5 minutes)
Public keys: 86400s (24 hours)
```

### Compression

#### Enable Compression
```toml
[features]
enable_compression = true
```

**Supported Formats**:
- gzip (default)
- brotli (if enabled)
- deflate

**Compression Thresholds**:
- Minimum size: 1KB
- Compression level: 6 (balanced)

## 3. Database Optimizations

### Query Optimization

#### Essential Indexes
```sql
-- User lookups
CREATE INDEX idx_users_email ON users(email);
CREATE INDEX idx_users_username ON users(username);

-- Session management
CREATE INDEX idx_sessions_user_id ON sessions(user_id);
CREATE INDEX idx_sessions_token ON sessions(token);
CREATE INDEX idx_sessions_expires_at ON sessions(expires_at);

-- Audit logging
CREATE INDEX idx_audit_logs_user_id ON audit_logs(user_id);
CREATE INDEX idx_audit_logs_timestamp ON audit_logs(created_at);

-- MFA
CREATE INDEX idx_mfa_devices_user_id ON mfa_devices(user_id);
CREATE INDEX idx_mfa_attempts_user_id ON mfa_attempts(user_id);

-- RBAC
CREATE INDEX idx_role_assignments_user_id ON role_assignments(user_id);
CREATE INDEX idx_permissions_role_id ON permissions(role_id);
```

#### Query Analysis
```sql
-- Analyze query performance
EXPLAIN ANALYZE SELECT * FROM users WHERE email = 'user@example.com';

-- Identify slow queries
SELECT query, calls, mean_time
FROM pg_stat_statements
ORDER BY mean_time DESC
LIMIT 10;
```

### Connection Pool Tuning

#### Monitor Connection Usage
```sql
-- Check active connections
SELECT count(*) FROM pg_stat_activity;

-- Check connection by database
SELECT datname, count(*)
FROM pg_stat_activity
GROUP BY datname;
```

#### Adjust Pool Size
```toml
[database]
# For high-concurrency workloads
max_connections = 100
min_connections = 20

# For low-concurrency workloads
max_connections = 20
min_connections = 5
```

### Vacuum & Analyze

#### Automatic Maintenance
```sql
-- Enable autovacuum
ALTER TABLE users SET (autovacuum_vacuum_scale_factor = 0.05);
ALTER TABLE sessions SET (autovacuum_vacuum_scale_factor = 0.01);

-- Analyze tables
ANALYZE users;
ANALYZE sessions;
ANALYZE audit_logs;
```

## 4. API Performance

### Response Caching

#### Cache Headers
```rust
// Automatically set by framework
// Cache-Control: public, max-age=300
// ETag: "hash-of-content"
// Last-Modified: timestamp
```

#### Conditional Requests
```bash
# Client sends If-None-Match
curl -H "If-None-Match: \"hash\"" https://api.example.com/users/me

# Server responds with 304 Not Modified
```

### Pagination

#### Efficient Pagination
```bash
# Use cursor-based pagination for large datasets
GET /api/v1/users?cursor=abc123&limit=50

# Avoid offset-based pagination
GET /api/v1/users?offset=10000&limit=50  # Slow!
```

### Batch Operations

#### Bulk Endpoints
```bash
# Batch user creation
POST /api/v1/users/batch
{
  "users": [
    {"email": "user1@example.com", ...},
    {"email": "user2@example.com", ...}
  ]
}

# Batch permission assignment
POST /api/v1/permissions/batch
{
  "assignments": [...]
}
```

## 5. Authentication Performance

### JWT Optimization

#### Token Caching
```toml
[features]
enable_caching = true
```

#### Token Validation
- Cache validation results (5 min TTL)
- Use symmetric signing (HS256) for speed
- Avoid expensive cryptographic operations in hot paths

### MFA Performance

#### MFA Caching
```toml
[mfa_rate_limit]
cache_ttl = 300  # 5 minutes
```

#### Async MFA Verification
- Non-blocking verification
- Parallel processing of multiple factors
- Early exit on failure

### Session Management

#### Session Caching
```toml
[redis]
mfa_cache_ttl = 300
```

#### Session Cleanup
```sql
-- Periodic cleanup of expired sessions
DELETE FROM sessions WHERE expires_at < NOW();

-- Schedule with cron
0 * * * * psql authenc -c "DELETE FROM sessions WHERE expires_at < NOW();"
```

## 6. Observability Performance

### Metrics Collection

#### Efficient Metrics
- Use histograms for latency (not gauges)
- Batch metric updates
- Avoid high-cardinality labels

#### Metrics Sampling
```toml
[observability]
# Sample 10% of requests
metrics_sample_rate = 0.1
```

### Logging Performance

#### Structured Logging
- JSON format for efficient parsing
- Async logging to avoid blocking
- Log level filtering at compile time

#### Log Rotation
```bash
# Rotate logs daily
0 0 * * * logrotate /etc/logrotate.d/authenc
```

## 7. Load Testing

### Load Test Scenarios

#### Scenario 1: Login Spike
```bash
# Simulate 1000 concurrent login attempts
ab -n 10000 -c 1000 -p login.json https://api.example.com/api/v1/auth/login
```

#### Scenario 2: Token Validation
```bash
# Simulate 5000 token validations/sec
wrk -t 8 -c 1000 -d 60s \
  -H "Authorization: Bearer token" \
  https://api.example.com/api/v1/auth/validate
```

#### Scenario 3: User Lookup
```bash
# Simulate 2000 user lookups/sec
wrk -t 8 -c 500 -d 60s \
  https://api.example.com/api/v1/users/me
```

### Load Test Tools

#### Apache Bench
```bash
ab -n 10000 -c 100 https://api.example.com/api/v1/health
```

#### Wrk
```bash
wrk -t 4 -c 100 -d 30s https://api.example.com/api/v1/health
```

#### k6
```javascript
import http from 'k6/http';
import { check } from 'k6';

export let options = {
  stages: [
    { duration: '30s', target: 100 },
    { duration: '1m', target: 500 },
    { duration: '30s', target: 0 },
  ],
};

export default function() {
  let response = http.get('https://api.example.com/api/v1/health');
  check(response, {
    'status is 200': (r) => r.status === 200,
    'response time < 100ms': (r) => r.timings.duration < 100,
  });
}
```

## 8. Scaling Strategies

### Horizontal Scaling

#### Load Balancer Configuration
```nginx
upstream authenc {
    least_conn;  # Use least connections algorithm
    server authenc-1:8088 max_fails=3 fail_timeout=30s;
    server authenc-2:8088 max_fails=3 fail_timeout=30s;
    server authenc-3:8088 max_fails=3 fail_timeout=30s;
}

server {
    listen 443 ssl;
    location / {
        proxy_pass http://authenc;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }
}
```

#### Session Affinity
```nginx
# Sticky sessions for MFA flows
upstream authenc {
    hash $cookie_sessionid consistent;
    server authenc-1:8088;
    server authenc-2:8088;
    server authenc-3:8088;
}
```

### Vertical Scaling

#### Resource Allocation
```yaml
# Kubernetes resource requests/limits
resources:
  requests:
    memory: "256Mi"
    cpu: "500m"
  limits:
    memory: "512Mi"
    cpu: "1000m"
```

### Database Scaling

#### Read Replicas
```toml
# Primary database
[database]
host = "primary.example.com"
port = 5432

# Read replica for analytics
[database.read_replica]
host = "replica.example.com"
port = 5432
```

#### Sharding (if needed)
- Shard by user_id
- Shard by tenant_id
- Consistent hashing for distribution

## 9. Monitoring & Profiling

### Continuous Profiling

#### CPU Profiling
```bash
# Generate CPU profile
cargo flamegraph --bin authenc

# Analyze with flamegraph
flamegraph.pl authenc.svg
```

#### Memory Profiling
```bash
# Use valgrind for memory analysis
valgrind --leak-check=full ./target/release/authenc
```

### Performance Metrics

#### Key Metrics to Track
```
authenc_http_requests_total{method, status, path}
authenc_http_request_duration_seconds{method, path}
authenc_db_query_duration_seconds{query}
authenc_cache_hit_ratio
authenc_mfa_verification_duration_seconds
authenc_jwt_validation_duration_seconds
```

#### Alerting Thresholds
```yaml
- alert: HighLatency
  expr: histogram_quantile(0.99, authenc_http_request_duration_seconds) > 0.1
  for: 5m

- alert: HighErrorRate
  expr: rate(authenc_http_requests_total{status=~"5.."}[5m]) > 0.01
  for: 5m

- alert: CacheHitRatioDegraded
  expr: authenc_cache_hit_ratio < 0.7
  for: 10m
```

## 10. Optimization Checklist

- [ ] Enable LTO in release profile
- [ ] Set workers to CPU core count
- [ ] Configure connection pools
- [ ] Enable Redis caching
- [ ] Create database indexes
- [ ] Enable compression
- [ ] Set appropriate TTLs
- [ ] Configure rate limiting
- [ ] Enable metrics collection
- [ ] Set up load balancer
- [ ] Configure health checks
- [ ] Enable log rotation
- [ ] Set up monitoring/alerting
- [ ] Run load tests
- [ ] Document scaling procedures
- [ ] Set up auto-scaling (if using K8s)

## 11. Performance Tuning Commands

### Check Current Performance
```bash
# Response time
curl -w "@curl-format.txt" -o /dev/null -s https://api.example.com/api/v1/health

# Database connections
psql authenc -c "SELECT count(*) FROM pg_stat_activity;"

# Cache hit rate
redis-cli INFO stats | grep hits

# System resources
top -b -n 1 | head -20
```

### Benchmark Specific Endpoints
```bash
# Login performance
ab -n 1000 -c 10 -p login.json https://api.example.com/api/v1/auth/login

# Token validation
ab -n 5000 -c 100 -H "Authorization: Bearer token" \
  https://api.example.com/api/v1/auth/validate

# User lookup
ab -n 2000 -c 50 https://api.example.com/api/v1/users/me
```

---

**Last Updated**: 2024-01-15
**Version**: 0.1.0
