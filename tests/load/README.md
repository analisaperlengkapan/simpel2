# Load Testing with k6

This directory contains k6 load test scripts for SIMPEL performance testing.

## Prerequisites

Install k6:

```bash
# macOS
brew install k6

# Linux (Debian/Ubuntu)
sudo gpg -k
sudo gpg --no-default-keyring --keyring /usr/share/keyrings/k6-archive-keyring.gpg --keyserver hkp://keyserver.ubuntu.com:80 --recv-keys C5AD17C747E3415A3642D57D77C6C491D6AC1D69
echo "deb [signed-by=/usr/share/keyrings/k6-archive-keyring.gpg] https://dl.k6.io/deb stable main" | sudo tee /etc/apt/sources.list.d/k6.list
sudo apt-get update
sudo apt-get install k6

# Windows (via Chocolatey)
choco install k6

# Or download from: https://k6.io/docs/get-started/installation/
```

## Test Scripts

### 1. Dashboard Load Test (`dashboard-load-test.js`)

Tests dashboard performance with 100 concurrent users.

**Performance Targets:**

- Dashboard load time: ≤ 5 seconds (NFR-P004)
- API response time (95th percentile): ≤ 500ms (NFR-P002)
- Error rate: < 5%

**Usage:**

```bash
# Basic run
k6 run tests/load/dashboard-load-test.js

# Custom configuration
k6 run --vus 100 --duration 5m tests/load/dashboard-load-test.js

# With environment variables
k6 run -e BASE_URL=http://staging.simpel.internal:3020 \
       -e TEST_USERNAME=operator \
       -e TEST_PASSWORD=password \
       tests/load/dashboard-load-test.js
```

### 2. API Load Test (`api-load-test.js`)

Tests API performance at 100 requests/second.

**Performance Targets:**

- API response time (95th percentile): ≤ 500ms (NFR-P002)
- API throughput: ≥ 100 requests/second (NFR-P005)
- Error rate: < 1%

**Usage:**

```bash
# Basic run
k6 run tests/load/api-load-test.js

# Custom configuration
k6 run --vus 50 --duration 5m tests/load/api-load-test.js

# With environment variables
k6 run -e BASE_URL=http://staging.simpel.internal:3020 \
       -e TEST_USERNAME=operator \
       -e TEST_PASSWORD=password \
       tests/load/api-load-test.js
```

### 3. Search Load Test (`search-load-test.js`)

Tests search functionality performance.

**Performance Targets:**

- Search response time (95th percentile): ≤ 500ms (NFR-P002)
- Web page response time (90th percentile): ≤ 2 seconds (NFR-P001)
- Error rate: < 2%

**Usage:**

```bash
# Basic run
k6 run tests/load/search-load-test.js

# Custom configuration
k6 run --vus 50 --duration 5m tests/load/search-load-test.js

# With environment variables
k6 run -e BASE_URL=http://staging.simpel.internal:3020 \
       -e TEST_USERNAME=operator \
       -e TEST_PASSWORD=password \
       tests/load/search-load-test.js
```

## Running All Tests

Use the provided shell script:

```bash
# Run all load tests
./tests/load/run-all-tests.sh

# Run with custom base URL
BASE_URL=http://staging.simpel.internal:3020 ./tests/load/run-all-tests.sh
```

## Output Formats

### Console Output (default)

```bash
k6 run tests/load/dashboard-load-test.js
```

### JSON Output

```bash
k6 run --out json=results.json tests/load/dashboard-load-test.js
```

### InfluxDB Output (for Grafana visualization)

```bash
k6 run --out influxdb=http://localhost:8086/k6 tests/load/dashboard-load-test.js
```

### Cloud Output (k6 Cloud)

```bash
k6 cloud tests/load/dashboard-load-test.js
```

## Analyzing Results

### Key Metrics

1. **http_req_duration**: Total request duration
   - p(90): 90th percentile
   - p(95): 95th percentile
   - p(99): 99th percentile

2. **http_reqs**: Total number of requests
   - rate: Requests per second (throughput)

3. **http_req_failed**: Failed requests
   - rate: Error rate

4. **Custom Metrics**:
   - `dashboard_load_time`: Dashboard load time
   - `api_response_time`: API response time
   - `search_response_time`: Search response time
   - `error_rate`: Custom error rate
   - `request_count`: Total request count

### Interpreting Results

**Success Criteria:**

- ✅ All thresholds pass
- ✅ Error rate < 5%
- ✅ p(95) response time meets targets
- ✅ Throughput meets targets

**Warning Signs:**

- ⚠️ Increasing response times over test duration
- ⚠️ Error rate > 1%
- ⚠️ Throughput declining under load

**Failure Indicators:**

- ❌ Thresholds fail
- ❌ Error rate > 5%
- ❌ Response times exceed targets
- ❌ System crashes or becomes unresponsive

## Performance Targets (from Requirements)

| Metric | Target | Requirement |
|--------|--------|-------------|
| Web page response time (90th percentile) | ≤ 2 seconds | NFR-P001 |
| API response time (95th percentile) | ≤ 500ms | NFR-P002 |
| Dashboard load time | ≤ 5 seconds | NFR-P004 |
| API throughput | ≥ 100 requests/second | NFR-P005 |
| Concurrent users | ≥ 500 users | NFR-SC001 |

## Troubleshooting

### High Error Rates

1. Check server logs for errors
2. Verify database connection pool size
3. Check Redis cache configuration
4. Monitor system resources (CPU, memory, disk I/O)

### Slow Response Times

1. Enable database query logging
2. Check for missing indexes
3. Review cache hit rates
4. Profile slow endpoints
5. Check network latency

### Connection Errors

1. Verify BASE_URL is correct
2. Check firewall rules
3. Verify service is running
4. Check connection pool exhaustion

## Best Practices

1. **Run tests in staging environment first**
2. **Start with small load and gradually increase**
3. **Monitor system resources during tests**
4. **Run tests during off-peak hours**
5. **Document baseline performance**
6. **Compare results over time**
7. **Test after major changes**

## CI/CD Integration

Add to `.gitlab-ci.yml`:

```yaml
load-test:
  stage: test
  image: grafana/k6:latest
  script:
    - k6 run --out json=results.json tests/load/api-load-test.js
  artifacts:
    reports:
      junit: results.json
  only:
    - staging
    - production
```

## References

- [k6 Documentation](https://k6.io/docs/)
- [k6 Best Practices](https://k6.io/docs/testing-guides/test-types/)
- [Performance Testing Guide](https://k6.io/docs/testing-guides/)
