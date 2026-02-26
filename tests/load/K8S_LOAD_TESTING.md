# Load Testing dengan Kubernetes (kubectl)

Panduan menjalankan k6 load tests di Kubernetes cluster menggunakan kubectl.

## Prerequisites

1. **kubectl** terinstall dan terkonfigurasi
2. **Akses ke cluster** (simpelv2-staging atau simpelv2-production)
3. **Service perlengkapan** sudah running di namespace

## Quick Start

### 1. Jalankan Semua Tests

```bash
# Di namespace staging
./tests/load/run-k8s-load-tests.sh simpelv2-staging all

# Di namespace production (hati-hati!)
./tests/load/run-k8s-load-tests.sh simpelv2-production all
```

### 2. Jalankan Test Spesifik

```bash
# Dashboard load test
./tests/load/run-k8s-load-tests.sh simpelv2-staging dashboard

# API load test
./tests/load/run-k8s-load-tests.sh simpelv2-staging api

# Search load test
./tests/load/run-k8s-load-tests.sh simpelv2-staging search
```

## Manual kubectl Commands

### 1. Apply ConfigMap dan Jobs

```bash
# Apply semua resources
kubectl apply -f tests/load/k8s-load-test-job.yaml -n simpelv2-staging

# Atau apply hanya ConfigMap
kubectl apply -f tests/load/k8s-load-test-job.yaml -n simpelv2-staging | grep ConfigMap
```

### 2. Create Secret untuk Test Credentials

```bash
# Create secret (jika belum ada)
kubectl create secret generic load-test-credentials \
  --from-literal=password='test_password' \
  -n simpelv2-staging
```

### 3. Run Specific Test Job

```bash
# Dashboard test
kubectl create -f - <<EOF
apiVersion: batch/v1
kind: Job
metadata:
  name: k6-dashboard-load-test
  namespace: simpelv2-staging
spec:
  template:
    spec:
      restartPolicy: Never
      containers:
      - name: k6
        image: grafana/k6:latest
        command: ["k6", "run", "/scripts/dashboard-load-test.js"]
        env:
        - name: BASE_URL
          value: "http://layanan-perlengkapan:3020"
        volumeMounts:
        - name: scripts
          mountPath: /scripts
      volumes:
      - name: scripts
        configMap:
          name: k6-load-tests
EOF
```

### 4. Monitor Test Execution

```bash
# List jobs
kubectl get jobs -n simpelv2-staging -l app=k6-load-test

# Get pods
kubectl get pods -n simpelv2-staging -l app=k6-load-test

# Stream logs
POD_NAME=$(kubectl get pods -n simpelv2-staging -l test-type=dashboard -o jsonpath='{.items[0].metadata.name}')
kubectl logs -f $POD_NAME -n simpelv2-staging

# Check job status
kubectl describe job k6-dashboard-load-test -n simpelv2-staging
```

### 5. Get Results

```bash
# Get logs from completed pod
kubectl logs k6-dashboard-load-test-xxxxx -n simpelv2-staging > dashboard-results.log

# Get all test results
for pod in $(kubectl get pods -n simpelv2-staging -l app=k6-load-test -o name); do
  kubectl logs $pod -n simpelv2-staging > $(basename $pod).log
done
```

### 6. Cleanup

```bash
# Delete all load test jobs
kubectl delete jobs -l app=k6-load-test -n simpelv2-staging

# Delete specific job
kubectl delete job k6-dashboard-load-test -n simpelv2-staging

# Delete ConfigMap (jika perlu update)
kubectl delete configmap k6-load-tests -n simpelv2-staging
```

## Configuration

### Environment Variables

Edit di `k8s-load-test-job.yaml`:

```yaml
env:
- name: BASE_URL
  value: "http://layanan-perlengkapan:3020"  # Service URL
- name: TEST_USERNAME
  value: "test_operator"
- name: TEST_PASSWORD
  valueFrom:
    secretKeyRef:
      name: load-test-credentials
      key: password
```

### Resource Limits

Sesuaikan resource requests/limits:

```yaml
resources:
  requests:
    memory: "256Mi"
    cpu: "500m"
  limits:
    memory: "512Mi"
    cpu: "1000m"
```

### Test Duration

Edit di ConfigMap script:

```javascript
export const options = {
  stages: [
    { duration: '1m', target: 20 },   // Ramp up
    { duration: '5m', target: 100 },  // Sustained load
    { duration: '1m', target: 0 },    // Ramp down
  ],
};
```

## Performance Targets

| Metric | Target | Requirement |
|--------|--------|-------------|
| Dashboard load time (p95) | ≤ 5 seconds | NFR-P004 |
| API response time (p95) | ≤ 500ms | NFR-P002 |
| API throughput | ≥ 100 req/s | NFR-P005 |
| Error rate | < 5% | - |
| Concurrent users | 100 users | NFR-SC001 |

## Interpreting Results

### Success Indicators

```
✓ checks.........................: 100.00% ✓ 5000      ✗ 0
✓ http_req_duration..............: avg=245ms   p(95)=450ms
✓ http_reqs......................: 5000 (100/s)
✓ error_rate.....................: 0.00%
```

### Warning Signs

```
⚠ http_req_duration..............: avg=850ms   p(95)=1200ms  # Slow!
⚠ error_rate.....................: 2.50%                      # Errors!
⚠ http_reqs......................: 3500 (70/s)               # Low throughput!
```

### Failure Indicators

```
✗ checks.........................: 85.00% ✓ 4250      ✗ 750   # Many failures!
✗ http_req_duration..............: avg=2500ms  p(95)=5000ms  # Very slow!
✗ error_rate.....................: 15.00%                     # High error rate!
```

## Troubleshooting

### Pod CrashLoopBackOff

```bash
# Check pod events
kubectl describe pod k6-dashboard-load-test-xxxxx -n simpelv2-staging

# Check logs
kubectl logs k6-dashboard-load-test-xxxxx -n simpelv2-staging

# Common issues:
# - ConfigMap not found: Apply ConfigMap first
# - Secret not found: Create load-test-credentials secret
# - Image pull error: Check network/registry access
```

### Job Not Starting

```bash
# Check job status
kubectl get job k6-dashboard-load-test -n simpelv2-staging -o yaml

# Check events
kubectl get events -n simpelv2-staging --sort-by='.lastTimestamp'

# Common issues:
# - Resource quota exceeded
# - Node selector mismatch
# - Image pull policy
```

### Connection Refused

```bash
# Check if service exists
kubectl get svc layanan-perlengkapan -n simpelv2-staging

# Check service endpoints
kubectl get endpoints layanan-perlengkapan -n simpelv2-staging

# Test connectivity from another pod
kubectl run test-curl --rm -it --restart=Never --image=curlimages/curl -- \
  curl http://layanan-perlengkapan:3020/health
```

### High Error Rates

1. **Check service logs**:
   ```bash
   kubectl logs -l app.kubernetes.io/name=layanan-perlengkapan -n simpelv2-staging
   ```

2. **Check database**:
   ```bash
   kubectl get pods -l app.kubernetes.io/name=postgres -n simpelv2-staging
   ```

3. **Check resource usage**:
   ```bash
   kubectl top pods -n simpelv2-staging
   kubectl top nodes
   ```

## Best Practices

1. **Test di staging dulu** sebelum production
2. **Monitor resource usage** selama test:
   ```bash
   watch kubectl top pods -n simpelv2-staging
   ```
3. **Backup database** sebelum load test besar
4. **Jalankan saat off-peak hours**
5. **Start dengan load kecil** dan tingkatkan bertahap
6. **Document baseline performance** untuk comparison
7. **Cleanup jobs** setelah selesai

## Integration dengan CI/CD

Tambahkan ke `.gitlab-ci.yml`:

```yaml
load-test:staging:
  stage: test
  image: bitnami/kubectl:latest
  script:
    - kubectl config use-context staging
    - ./tests/load/run-k8s-load-tests.sh simpelv2-staging all
  artifacts:
    paths:
      - tests/load/results/
    expire_in: 30 days
  only:
    - staging
  when: manual
```

## Monitoring Dashboard

Jika Prometheus/Grafana tersedia:

```bash
# Port forward Grafana
kubectl port-forward -n monitoring svc/grafana 3000:80

# Access: http://localhost:3000
# Import k6 dashboard: https://grafana.com/grafana/dashboards/2587
```

## References

- [k6 Documentation](https://k6.io/docs/)
- [k6 Kubernetes Guide](https://k6.io/docs/testing-guides/running-distributed-tests/)
- [Kubernetes Jobs](https://kubernetes.io/docs/concepts/workloads/controllers/job/)
