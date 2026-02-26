# Load Test Analysis & Optimization Guide

Dokumen ini berisi panduan analisis hasil load test dan rekomendasi optimasi.

## Baseline Performance Targets

Berdasarkan requirements (NFR-P001, NFR-P002, NFR-P004, NFR-P005):

| Metric | Target | Requirement |
|--------|--------|-------------|
| Web page response time (p90) | ≤ 2 seconds | NFR-P001 |
| API response time (p95) | ≤ 500ms | NFR-P002 |
| Dashboard load time | ≤ 5 seconds | NFR-P004 |
| API throughput | ≥ 100 req/s | NFR-P005 |
| Concurrent users | ≥ 500 users | NFR-SC001 |

## Test Execution Checklist

### Pre-Test

- [ ] Backup database
- [ ] Verify all services running
- [ ] Check resource availability (CPU, memory, disk)
- [ ] Clear Redis cache (optional, for consistent baseline)
- [ ] Document current system state
- [ ] Set up monitoring (Prometheus/Grafana)

### During Test

- [ ] Monitor CPU usage: `kubectl top pods -n simpelv2-staging`
- [ ] Monitor memory usage
- [ ] Watch service logs: `kubectl logs -f <pod> -n simpelv2-staging`
- [ ] Monitor database connections
- [ ] Check Redis hit rate
- [ ] Monitor network I/O

### Post-Test

- [ ] Collect all logs
- [ ] Export metrics
- [ ] Analyze bottlenecks
- [ ] Document findings
- [ ] Create optimization plan

## Analysis Framework

### 1. Response Time Analysis

#### Dashboard Load Test

**Expected Results:**
```
Metric                          | Target    | Actual    | Status
--------------------------------|-----------|-----------|--------
dashboard_load_time (p95)       | ≤ 5000ms  | ?         | ?
api_response_time (p95)         | ≤ 500ms   | ?         | ?
error_rate                      | < 5%      | ?         | ?
http_req_duration (p90)         | ≤ 2000ms  | ?         | ?
```

**Bottleneck Indicators:**
- p95 > 5000ms → Database query optimization needed
- p99 > 10000ms → Severe performance issue
- Increasing trend → Memory leak or resource exhaustion

**Common Issues:**
1. **Slow database queries**
   - Check: `EXPLAIN ANALYZE` on slow queries
   - Fix: Add indexes, optimize joins

2. **Missing cache**
   - Check: Redis hit rate
   - Fix: Implement caching for dashboard metrics

3. **N+1 queries**
   - Check: Query count per request
   - Fix: Use joins or batch loading

#### API Load Test

**Expected Results:**
```
Metric                          | Target    | Actual    | Status
--------------------------------|-----------|-----------|--------
api_response_time (p95)         | ≤ 500ms   | ?         | ?
http_reqs (rate)                | ≥ 100/s   | ?         | ?
error_rate                      | < 1%      | ?         | ?
```

**Bottleneck Indicators:**
- Throughput < 100 req/s → Connection pool exhaustion
- p95 > 500ms → Slow queries or external API calls
- Error rate > 1% → Service overload

**Common Issues:**
1. **Connection pool exhaustion**
   - Check: Database pool size
   - Fix: Increase pool size (current: 10-50)

2. **Slow external API calls**
   - Check: Integration service response times
   - Fix: Implement timeout, circuit breaker

3. **CPU bottleneck**
   - Check: `kubectl top pods`
   - Fix: Increase CPU limits, optimize algorithms

#### Search Load Test

**Expected Results:**
```
Metric                          | Target    | Actual    | Status
--------------------------------|-----------|-----------|--------
search_response_time (p95)      | ≤ 500ms   | ?         | ?
search_response_time (p99)      | ≤ 2000ms  | ?         | ?
error_rate                      | < 2%      | ?         | ?
```

**Bottleneck Indicators:**
- p95 > 500ms → Missing full-text search indexes
- High variance → Inconsistent query performance
- Timeout errors → Query too complex

**Common Issues:**
1. **Missing indexes**
   - Check: `EXPLAIN` on search queries
   - Fix: Add GIN indexes for full-text search

2. **Large result sets**
   - Check: Result count per query
   - Fix: Implement pagination, limit results

3. **Complex filters**
   - Check: Query execution plan
   - Fix: Simplify filters, add composite indexes

### 2. Error Analysis

#### Error Rate Thresholds

| Error Rate | Severity | Action |
|------------|----------|--------|
| 0-1% | Normal | Monitor |
| 1-5% | Warning | Investigate |
| 5-10% | Critical | Fix immediately |
| >10% | Emergency | Stop test, rollback |

#### Common Error Types

**HTTP 500 (Internal Server Error)**
- Database connection timeout
- Unhandled exception
- Memory exhaustion

**HTTP 503 (Service Unavailable)**
- Service overload
- Circuit breaker open
- Pod crash/restart

**HTTP 401/403 (Auth Error)**
- Token expiration
- Invalid credentials
- Rate limiting

**Connection Errors**
- Network timeout
- DNS resolution failure
- Service not reachable

### 3. Resource Utilization

#### CPU Usage

```bash
# Monitor during test
kubectl top pods -n simpelv2-staging --watch

# Expected:
# - Normal: 30-50% CPU
# - Peak: 70-80% CPU
# - Critical: >90% CPU (throttling)
```

**Optimization:**
- If CPU > 80%: Increase CPU limits
- If CPU < 30%: Reduce CPU requests (save resources)

#### Memory Usage

```bash
# Monitor memory
kubectl top pods -n simpelv2-staging

# Expected:
# - Normal: 40-60% memory
# - Peak: 70-80% memory
# - Critical: >90% memory (OOM risk)
```

**Optimization:**
- If memory > 80%: Check for memory leaks
- If memory growing: Implement connection pooling
- If OOM kills: Increase memory limits

#### Database Connections

```sql
-- Check active connections
SELECT count(*) FROM pg_stat_activity WHERE state = 'active';

-- Check connection pool usage
SELECT
  datname,
  count(*) as connections,
  max_conn
FROM pg_stat_activity
GROUP BY datname, max_conn;
```

**Optimization:**
- If connections > 80% of pool: Increase pool size
- If many idle connections: Reduce connection timeout
- If connection errors: Check pool configuration

### 4. Throughput Analysis

#### Request Rate

```
Target: ≥ 100 requests/second
Actual: ? requests/second

Calculation:
- Total requests: ?
- Test duration: ? seconds
- Throughput: Total / Duration = ? req/s
```

**Bottleneck Indicators:**
- Throughput declining over time → Resource exhaustion
- Throughput < 100 req/s → Performance bottleneck
- High latency at low throughput → Inefficient code

## Optimization Strategies

### Phase 1: Quick Wins (Hours)

1. **Add Missing Indexes**
   ```sql
   -- Foreign key indexes
   CREATE INDEX idx_kebutuhan_satker ON kebutuhan_bmn(satker_id);
   CREATE INDEX idx_kebutuhan_tahun ON kebutuhan_bmn(tahun_anggaran);

   -- Full-text search indexes
   CREATE INDEX idx_kebutuhan_search ON kebutuhan_bmn
   USING GIN(to_tsvector('indonesian', nama_barang));
   ```

2. **Implement Caching**
   ```rust
   // Cache dashboard metrics (5 minutes TTL)
   let cache_key = format!("dashboard:metrics:{}", tahun);
   if let Some(cached) = redis.get(&cache_key).await? {
       return Ok(cached);
   }

   let metrics = calculate_metrics().await?;
   redis.setex(&cache_key, 300, &metrics).await?;
   ```

3. **Optimize Connection Pool**
   ```rust
   // Increase pool size
   DatabaseConfig {
       pool_min: 10,
       pool_max: 50,  // Increase from 20
       connection_timeout: Duration::from_secs(30),
   }
   ```

### Phase 2: Medium-term (Days)

1. **Implement Query Optimization**
   ```sql
   -- Before: N+1 queries
   SELECT * FROM kebutuhan_bmn WHERE satker_id = ?;
   -- For each: SELECT * FROM satkers WHERE id = ?;

   -- After: Single query with join
   SELECT k.*, s.nama as satker_nama
   FROM kebutuhan_bmn k
   JOIN satkers s ON k.satker_id = s.id
   WHERE k.satker_id = ?;
   ```

2. **Add Materialized Views**
   ```sql
   CREATE MATERIALIZED VIEW mv_dashboard_metrics AS
   SELECT
     tahun_anggaran,
     COUNT(*) as total_kebutuhan,
     SUM(jumlah_kebutuhan) as total_jumlah,
     COUNT(DISTINCT satker_id) as total_satker
   FROM kebutuhan_bmn
   GROUP BY tahun_anggaran;

   -- Refresh every 5 minutes
   CREATE OR REPLACE FUNCTION refresh_dashboard_metrics()
   RETURNS void AS $$
   BEGIN
     REFRESH MATERIALIZED VIEW CONCURRENTLY mv_dashboard_metrics;
   END;
   $$ LANGUAGE plpgsql;
   ```

3. **Implement Rate Limiting**
   ```rust
   // Already implemented in rate_limiting.rs
   // Verify configuration:
   RateLimitConfig {
       requests_per_minute: 100,
       burst_size: 200,
   }
   ```

### Phase 3: Long-term (Weeks)

1. **Database Partitioning**
   ```sql
   -- Partition by year
   CREATE TABLE kebutuhan_bmn_2026 PARTITION OF kebutuhan_bmn
   FOR VALUES FROM (2026) TO (2027);
   ```

2. **Read Replicas**
   - Set up PostgreSQL read replicas
   - Route read queries to replicas
   - Keep writes on primary

3. **Horizontal Scaling**
   ```yaml
   # Increase replicas
   spec:
     replicas: 3  # Increase from 1
   ```

4. **CDN for Static Assets**
   - Move frontend assets to CDN
   - Reduce server load

## Optimization Checklist

### Database

- [ ] Add missing indexes (foreign keys, search columns)
- [ ] Optimize slow queries (EXPLAIN ANALYZE)
- [ ] Implement connection pooling (min: 10, max: 50)
- [ ] Add materialized views for dashboards
- [ ] Enable query result caching
- [ ] Set up read replicas (if needed)
- [ ] Implement database partitioning (if needed)

### Caching

- [ ] Implement Redis caching
- [ ] Cache reference data (1 hour TTL)
- [ ] Cache dashboard metrics (5 minutes TTL)
- [ ] Cache gap analysis results (1 hour TTL)
- [ ] Implement cache invalidation on data changes
- [ ] Monitor cache hit rate (target: >80%)

### Application

- [ ] Optimize N+1 queries
- [ ] Implement batch loading
- [ ] Add rate limiting (100 req/min per user)
- [ ] Optimize JSON serialization
- [ ] Reduce payload size
- [ ] Implement compression (gzip)

### Infrastructure

- [ ] Increase CPU limits (if needed)
- [ ] Increase memory limits (if needed)
- [ ] Scale horizontally (add replicas)
- [ ] Set up autoscaling (HPA)
- [ ] Optimize network configuration
- [ ] Enable HTTP/2

## Monitoring & Alerting

### Key Metrics to Monitor

1. **Response Time**
   - p50, p95, p99 latency
   - Alert if p95 > 500ms

2. **Throughput**
   - Requests per second
   - Alert if < 100 req/s

3. **Error Rate**
   - HTTP 5xx errors
   - Alert if > 1%

4. **Resource Usage**
   - CPU, memory, disk I/O
   - Alert if > 80%

5. **Database**
   - Connection pool usage
   - Query duration
   - Alert if pool > 80%

### Prometheus Queries

```promql
# API response time (p95)
histogram_quantile(0.95,
  rate(http_request_duration_seconds_bucket[5m])
)

# Request rate
rate(http_requests_total[5m])

# Error rate
rate(http_requests_total{status=~"5.."}[5m])
/
rate(http_requests_total[5m])

# CPU usage
rate(container_cpu_usage_seconds_total[5m])

# Memory usage
container_memory_usage_bytes
```

## Re-testing After Optimization

After implementing optimizations:

1. **Run baseline test** (before optimization)
2. **Implement optimization**
3. **Run comparison test** (after optimization)
4. **Compare results**:
   ```
   Metric              | Before  | After   | Improvement
   --------------------|---------|---------|-------------
   p95 response time   | 800ms   | 350ms   | 56% faster
   Throughput          | 75 req/s| 120 req/s| 60% increase
   Error rate          | 3%      | 0.5%    | 83% reduction
   ```

5. **Document improvements**
6. **Update baseline**

## Success Criteria

Load test is considered successful if:

- ✅ All thresholds pass
- ✅ p95 response time ≤ 500ms
- ✅ Dashboard load time ≤ 5 seconds
- ✅ Throughput ≥ 100 req/s
- ✅ Error rate < 1%
- ✅ No service crashes
- ✅ Resource usage < 80%
- ✅ No memory leaks

## References

- [Performance Testing Best Practices](https://k6.io/docs/testing-guides/test-types/)
- [PostgreSQL Performance Tuning](https://wiki.postgresql.org/wiki/Performance_Optimization)
- [Redis Caching Strategies](https://redis.io/docs/manual/patterns/)
- [Kubernetes Resource Management](https://kubernetes.io/docs/concepts/configuration/manage-resources-containers/)
