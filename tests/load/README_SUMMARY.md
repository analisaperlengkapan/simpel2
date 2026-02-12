# Load Testing Suite - Summary

Dokumentasi lengkap untuk load testing SIMPEL menggunakan k6 di Kubernete
ml            # Kubernetes Job manifests
├── run-all-tests.sh                   # Script runner (local)
├── run-k8s-load-tests.sh             # Script runner (kubectl)
├── analyze-results.sh                 # Script analisis hasil
│
├── gitlab-ci-load-test.yml           # GitLab CI configuration
├── .env.example                       # Environment variables template
│
├── optimizations/
│   ├── OPTIMIZATION_GUIDE.md          # Panduan optimasi lengkap
│   ├── 001_add_performance_indexes.sql # SQL: Indexes
│   ├── 002_create_materialized_views.sql # SQL: Materialized views
│   └── apply-optimizations.sh         # Script apply optimasi
│
└── results/                           # Hasil load test (generated)
    ├── dashboard_YYYYMMDD_HHMMSS.log
    ├── api_YYYYMMDD_HHMMSS.log
    ├── search_YYYYMMDD_HHMMSS.log
    └── analysis_report_YYYYMMDD_HHMMSS.md
```

## 🚀 Quick Start

### Option 1: Kubernetes (Recommended)

```bash
# 1. Apply ConfigMap dan Jobs
kubectl apply -f tests/load/k8s-load-test-job.yaml -n simpelv2-staging

# 2. Run all tests
./tests/load/run-k8s-load-tests.sh simpelv2-staging all

# 3. Analyze results
./tests/load/analyze-results.sh
```

### Option 2: Local (Requires k6 installed)

```bash
# 1. Install k6
# macOS: brew install k6
# Linux: See README.md

# 2. Run tests
./tests/load/run-all-tests.sh http://localhost:3020

# 3. Analyze results
./tests/load/analyze-results.sh
```

## 📊 Performance Targets

| Metric | Target | Requirement |
|--------|--------|-------------|
| Web page response time (p90) | ≤ 2 seconds | NFR-P001 |
| API response time (p95) | ≤ 500ms | NFR-P002 |
| Dashboard load time | ≤ 5 seconds | NFR-P004 |
| API throughput | ≥ 100 req/s | NFR-P005 |
| Concurrent users | ≥ 500 users | NFR-SC001 |
| Error rate | < 5% | - |

## 🧪 Test Scenarios

### 1. Dashboard Load Test

**Purpose:** Test dashboard performance with 100 concurrent users

**Stages:**
- Ramp up: 1-3 minutes (20 → 100 users)
- Sustained: 5 minutes (100 users)
- Ramp down: 2 minutes (100 → 0 users)

**Endpoints Tested:**
- `/api/v1/dashboard/portal`
- `/api/v1/dashboard/perlengkapan`
- `/api/v1/dashboard/gap-analysis`
- `/api/v1/dashboard/workflow-metrics`

**Success Criteria:**
- ✅ p95 response time ≤ 5 seconds
- ✅ Error rate < 5%

### 2. API Load Test

**Purpose:** Test API throughput at 100 requests/second

**Scenarios:**
- Constant rate: 100 req/s for 5 minutes
- Ramping VUs: 0 → 200 users over 12 minutes

**Endpoints Tested:**
- `/api/v1/kebutuhan` (30% weight)
- `/api/v1/pakaian-dinas` (15% weight)
- `/api/v1/roadmap` (10% weight)
- `/api/v1/master/kode-barang` (10% weight)
- `/api/v1/workflow/status` (10% weight)
- Others (25% weight)

**Success Criteria:**
- ✅ p95 response time ≤ 500ms
- ✅ Throughput ≥ 100 req/s
- ✅ Error rate < 1%

### 3. Search Load Test

**Purpose:** Test search functionality performance

**Stages:**
- Warm up: 1 minute (10 users)
- Ramp up: 5 minutes (10 → 50 users)
- Sustained: 5 minutes (50 users)
- Spike: 2 minutes (50 → 100 users)
- Recovery: 2 minutes (100 → 50 users)
- Ramp down: 1 minute (50 → 0 users)

**Search Types:**
- Basic search
- Search with filters
- Search with pagination
- Autocomplete search

**Success Criteria:**
- ✅ p95 response time ≤ 500ms
- ✅ p99 response time ≤ 2 seconds
- ✅ Error rate < 2%

## 🔧 Optimization Workflow

### 1. Baseline Testing

```bash
# Run initial load tests
./tests/load/run-k8s-load-tests.sh simpelv2-staging all

# Save results as baseline
cp tests/load/results/*.log tests/load/results/baseline/
```

### 2. Apply Optimizations

```bash
# Apply database optimizations
./tests/load/optimizations/apply-optimizations.sh

# Verify optimizations
psql "$DATABASE_URL" -c "SELECT * FROM pg_indexes WHERE schemaname = 'perlengkapan';"
```

### 3. Re-test & Compare

```bash
# Run load tests again
./tests/load/run-k8s-load-tests.sh simpelv2-staging all

# Analyze and compare
./tests/load/analyze-results.sh
```

### 4. Iterate

- Identify remaining bottlenecks
- Implement additional optimizations
- Repeat testing cycle

## 📈 Implemented Optimizations

### Database Level

1. **Performance Indexes**
   - Foreign key indexes
   - Composite indexes
   - Full-text search indexes (pg_trgm)
   - Partial indexes

2. **Materialized Views**
   - Dashboard metrics
   - Gap analysis
   - Workflow metrics
   - Satker summary

3. **Query Optimization**
   - Eliminated N+1 queries
   - Optimized JOINs
   - Added EXPLAIN ANALYZE

### Application Level

1. **Connection Pooling**
   - Increased pool size (10-50)
   - Optimized timeouts

2. **Caching**
   - Redis caching (5-60 min TTL)
   - Cache invalidation strategy

3. **Rate Limiting**
   - 100 req/min per user
   - Burst size: 200

## 📋 Checklist

### Pre-Test

- [ ] Backup database
- [ ] Verify all services running
- [ ] Check resource availability
- [ ] Clear Redis cache (optional)
- [ ] Set up monitoring

### During Test

- [ ] Monitor CPU/memory usage
- [ ] Watch service logs
- [ ] Monitor database connections
- [ ] Check Redis hit rate

### Post-Test

- [ ] Collect all logs
- [ ] Export metrics
- [ ] Analyze bottlenecks
- [ ] Document findings
- [ ] Create optimization plan

### After Optimization

- [ ] Apply optimizations
- [ ] Verify optimizations
- [ ] Re-run load tests
- [ ] Compare with baseline
- [ ] Document improvements

## 🔍 Troubleshooting

### Common Issues

1. **High Error Rates**
   - Check service logs
   - Verify database connections
   - Check resource limits

2. **Slow Response Times**
   - Run EXPLAIN ANALYZE
   - Check for missing indexes
   - Monitor cache hit rate

3. **Connection Errors**
   - Verify service endpoints
   - Check network policies
   - Verify credentials

### Debug Commands

```bash
# Check service status
kubectl get pods -n simpelv2-staging

# View service logs
kubectl logs -f <pod-name> -n simpelv2-staging

# Check resource usage
kubectl top pods -n simpelv2-staging

# Test connectivity
kubectl run test-curl --rm -it --restart=Never --image=curlimages/curl -- \
  curl http://layanan-perlengkapan:3020/health
```

## 📚 Documentation

| Document | Purpose |
|----------|---------|
| [README.md](./README.md) | General load testing guide |
| [K8S_LOAD_TESTING.md](./K8S_LOAD_TESTING.md) | Kubernetes-specific guide |
| [LOAD_TEST_ANALYSIS.md](./LOAD_TEST_ANALYSIS.md) | Analysis framework |
| [OPTIMIZATION_GUIDE.md](./optimizations/OPTIMIZATION_GUIDE.md) | Optimization strategies |

## 🎯 Success Criteria

Load testing is successful when:

- ✅ All thresholds pass
- ✅ Performance targets met
- ✅ Error rate < 1%
- ✅ No service crashes
- ✅ Resource usage < 80%
- ✅ Improvements documented

## 📞 Support

For issues or questions:

1. Check troubleshooting section
2. Review documentation
3. Check service logs
4. Contact DevOps team

## 🔗 References

- [k6 Documentation](https://k6.io/docs/)
- [PostgreSQL Performance](https://wiki.postgresql.org/wiki/Performance_Optimization)
- [Kubernetes Best Practices](https://kubernetes.io/docs/concepts/configuration/manage-resources-containers/)
- [SIMPEL Requirements](../../.kiro/specs/simpel-completion/requirements.md)

---

**Last Updated:** February 10, 2026
**Maintained by:** SIMPelv2 DevOps Team
