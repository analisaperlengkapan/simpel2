# Load Testing Execution Guide

Panduan step-by-step untuk menjalankan load testing SIMPEL.

## Prerequisites

✅ kubectl installed and configured
✅ Access to simpelv2-staging namespace
✅ Service perlengkapan running
✅ Database accessible

## Step-by-Step Execution

### Step 1: Preparation (5 minutes)

#### 1.1 Verify Services

```bash
# Check if services are running
kubectl get pods -n simpelv2-staging | grep perlengkapan

# Expected output:
# layanan-perlengkapan-xxxxx   1/1   Running   0   7d
```

#### 1.2 Check Service Health

```bash
# Port forward to service
kubectl port-forward -n simpelv2-staging \
  svc/layanan-perlengkapan 3020:3020 &

# Test health endpoint
curl http://localhost:3020/health

# Expected: {"status":"ok"}

# Stop port forward
kill %1
```

#### 1.3 Backup Database (Recommended)

```bash
# Get database credentials from secret
DB_PASSWORD=$(kubectl get secret postgres-credentials -n simpelv2-staging \
  -o jsonpath='{.data.password}' | base64 -d)

# Create backup
kubectl exec -n simpelv2-staging postgres-0 -- \
  pg_dump -U simpelv2 perlengkapan > backup_$(date +%Y%m%d_%H%M%S).sql
```

### Step 2: Run Load Tests (15-30 minutes)

#### 2.1 Apply k6 ConfigMap

```bash
# Apply ConfigMap with test scripts
kubectl apply -f tests/load/k8s-load-test-job.yaml -n simpelv2-staging

# Verify ConfigMap created
kubectl get configmap k6-load-tests -n simpelv2-staging
```

#### 2.2 Create Test Credentials Secret

```bash
# Create secret for test user password
kubectl create secret generic load-test-credentials \
  --from-literal=password='test_password' \
  -n simpelv2-staging \
  --dry-run=client -o yaml | kubectl apply -f -
```

#### 2.3 Run All Tests

```bash
# Run all load tests
./tests/load/run-k8s-load-tests.sh simpelv2-staging all
```

**Expected Duration:**
- Dashboard test: ~13 minutes
- API test: ~14 minutes
- Search test: ~16 minutes
- **Total: ~45 minutes**

#### 2.4 Monitor Test Execution

Open new terminal and monitor:

```bash
# Terminal 1: Watch pods
watch kubectl get pods -n simpelv2-staging -l app=k6-load-test

# Terminal 2: Monitor resource usage
watch kubectl top pods -n simpelv2-staging

# Terminal 3: Watch service logs
kubectl logs -f -l app.kubernetes.io/name=layanan-perlengkapan \
  -n simpelv2-staging
```

### Step 3: Analyze Results (10 minutes)

#### 3.1 Collect Test Logs

```bash
# Logs are automatically saved to tests/load/results/
ls -lh tests/load/results/

# Expected files:
# dashboard_YYYYMMDD_HHMMSS.log
# api_YYYYMMDD_HHMMSS.log
# search_YYYYMMDD_HHMMSS.log
```

#### 3.2 Run Analysis Script

```bash
# Analyze results
./tests/load/analyze-results.sh

# View report
cat tests/load/results/analysis_report_*.md
```

#### 3.3 Review Key Metrics

Check for:

✅ **Response Times**
- Dashboard p95 ≤ 5000ms
- API p95 ≤ 500ms
- Search p95 ≤ 500ms

✅ **Throughput**
- API ≥ 100 req/s

✅ **Error Rate**
- < 5% for dashboard
- < 1% for API
- < 2% for search

### Step 4: Apply Optimizations (15 minutes)

#### 4.1 Review Optimization Plan

```bash
# Read optimization guide
cat tests/load/optimizations/OPTIMIZATION_GUIDE.md
```

#### 4.2 Apply Database Optimizations

```bash
# Get database URL
DB_URL="postgres://simpelv2:${DB_PASSWORD}@postgres.simpelv2-staging:5432/perlengkapan"

# Apply optimizations
./tests/load/optimizations/apply-optimizations.sh "$DB_URL"
```

**This will:**
1. Add performance indexes (~5 minutes)
2. Create materialized views (~5 minutes)
3. Verify optimizations (~2 minutes)

#### 4.3 Verify Optimizations

```bash
# Check indexes
kubectl exec -n simpelv2-staging postgres-0 -- \
  psql -U simpelv2 -d perlengkapan -c "
  SELECT tablename, COUNT(*) as index_count
  FROM pg_indexes
  WHERE schemaname = 'perlengkapan'
  GROUP BY tablename
  ORDER BY index_count DESC;
  "

# Check materialized views
kubectl exec -n simpelv2-staging postgres-0 -- \
  psql -U simpelv2 -d perlengkapan -c "
  SELECT matviewname, last_refresh
  FROM pg_matviews
  WHERE schemaname = 'perlengkapan';
  "
```

### Step 5: Re-test & Compare (30 minutes)

#### 5.1 Run Load Tests Again

```bash
# Run tests after optimization
./tests/load/run-k8s-load-tests.sh simpelv2-staging all
```

#### 5.2 Compare Results

```bash
# Analyze new results
./tests/load/analyze-results.sh

# Compare with baseline
diff tests/load/results/baseline/analysis_report_*.md \
     tests/load/results/analysis_report_*.md
```

#### 5.3 Document Improvements

Create comparison table:

| Metric | Before | After | Improvement |
|--------|--------|-------|-------------|
| Dashboard p95 | ? ms | ? ms | ?% |
| API p95 | ? ms | ? ms | ?% |
| Throughput | ? req/s | ? req/s | ?% |
| Error rate | ?% | ?% | ?% |

### Step 6: Cleanup (5 minutes)

#### 6.1 Delete Test Jobs

```bash
# Delete all load test jobs
kubectl delete jobs -l app=k6-load-test -n simpelv2-staging

# Verify deletion
kubectl get jobs -n simpelv2-staging -l app=k6-load-test
```

#### 6.2 Optional: Delete ConfigMap

```bash
# Only if you want to update test scripts
kubectl delete configmap k6-load-tests -n simpelv2-staging
```

#### 6.3 Archive Results

```bash
# Create archive
tar -czf load_test_results_$(date +%Y%m%d).tar.gz tests/load/results/

# Move to archive directory
mkdir -p tests/load/archives
mv load_test_results_*.tar.gz tests/load/archives/
```

## Troubleshooting

### Issue: Pod CrashLoopBackOff

**Symptoms:**
```bash
kubectl get pods -n simpelv2-staging -l app=k6-load-test
# NAME                          READY   STATUS             RESTARTS
# k6-dashboard-load-test-xxx    0/1     CrashLoopBackOff   3
```

**Solution:**
```bash
# Check pod logs
kubectl logs k6-dashboard-load-test-xxx -n simpelv2-staging

# Common causes:
# 1. ConfigMap not found → Apply ConfigMap first
# 2. Secret not found → Create load-test-credentials secret
# 3. Service not reachable → Check service endpoint
```

### Issue: High Error Rate

**Symptoms:**
- Error rate > 5%
- Many HTTP 500 errors

**Solution:**
```bash
# Check service logs
kubectl logs -l app.kubernetes.io/name=layanan-perlengkapan \
  -n simpelv2-staging --tail=100

# Check database connections
kubectl exec -n simpelv2-staging postgres-0 -- \
  psql -U simpelv2 -d perlengkapan -c "
  SELECT count(*) FROM pg_stat_activity WHERE state = 'active';
  "

# Check resource usage
kubectl top pods -n simpelv2-staging
```

### Issue: Slow Response Times

**Symptoms:**
- p95 > 500ms
- Dashboard > 5 seconds

**Solution:**
```bash
# Check for missing indexes
kubectl exec -n simpelv2-staging postgres-0 -- \
  psql -U simpelv2 -d perlengkapan -c "
  SELECT schemaname, tablename, indexname
  FROM pg_indexes
  WHERE schemaname = 'perlengkapan';
  "

# Check slow queries
kubectl exec -n simpelv2-staging postgres-0 -- \
  psql -U simpelv2 -d perlengkapan -c "
  SELECT query, calls, mean_exec_time
  FROM pg_stat_statements
  ORDER BY mean_exec_time DESC
  LIMIT 10;
  "
```

## Success Checklist

After completing all steps, verify:

- [ ] All load tests completed successfully
- [ ] Performance targets met
- [ ] Optimizations applied
- [ ] Improvements documented
- [ ] Results archived
- [ ] Cleanup completed

## Next Steps

1. **Set up Continuous Monitoring**
   - Configure Prometheus alerts
   - Create Grafana dashboards
   - Set up automated load testing

2. **Schedule Regular Testing**
   - Weekly load tests
   - Before major releases
   - After infrastructure changes

3. **Document Baseline**
   - Save current performance metrics
   - Update performance targets
   - Share with team

## Quick Reference

### Common Commands

```bash
# Run all tests
./tests/load/run-k8s-load-tests.sh simpelv2-staging all

# Run specific test
./tests/load/run-k8s-load-tests.sh simpelv2-staging dashboard

# Analyze results
./tests/load/analyze-results.sh

# Apply optimizations
./tests/load/optimizations/apply-optimizations.sh

# Check test status
kubectl get jobs -n simpelv2-staging -l app=k6-load-test

# View test logs
kubectl logs -f <pod-name> -n simpelv2-staging

# Cleanup
kubectl delete jobs -l app=k6-load-test -n simpelv2-staging
```

### Performance Targets

| Metric | Target |
|--------|--------|
| Dashboard p95 | ≤ 5000ms |
| API p95 | ≤ 500ms |
| Throughput | ≥ 100 req/s |
| Error rate | < 5% |

## Support

For help:
1. Check [K8S_LOAD_TESTING.md](./K8S_LOAD_TESTING.md)
2. Review [LOAD_TEST_ANALYSIS.md](./LOAD_TEST_ANALYSIS.md)
3. Contact DevOps team

---

**Estimated Total Time:** 2-3 hours
**Recommended Schedule:** Off-peak hours (after 18:00 WIB)
