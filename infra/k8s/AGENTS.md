# 🚀 AGENTS.md - Kubernetes Infrastructure Guide for SIMPelv2

> **For AI Agents**: This file is the primary reference for understanding and managing the Kubernetes infrastructure of SIMPelv2.

## 📋 Overview

This directory contains **Kustomize-based** Kubernetes manifests for deploying SIMPelv2 across multiple environments. The structure follows GitOps best practices with base/overlay pattern.

## 🏗️ Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                    MicroK8s Cluster                             │
│  ┌──────────────────┐  ┌──────────────────┐                     │
│  │ simpel.kejaksaan │  │    simple02      │                     │
│  │    .go.id        │  │    (worker)      │                     │
│  │  172.15.10.254   │  │  172.15.10.252   │                     │
│  └──────────────────┘  └──────────────────┘                     │
├─────────────────────────────────────────────────────────────────┤
│                    Infrastructure Layer                         │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐ ┌───────────┐ │
│  │   Istio     │ │   MetalLB   │ │  Longhorn   │ │ Registry  │ │
│  │  (mesh)     │ │ (172.15.10. │ │  (storage)  │ │ :32000    │ │
│  │             │ │  200-220)   │ │             │ │           │ │
│  └─────────────┘ └─────────────┘ └─────────────┘ └───────────┘ │
├─────────────────────────────────────────────────────────────────┤
│                    Application Namespaces                       │
│  ┌─────────────────────┐  ┌─────────────────────┐              │
│  │ simpelv2-staging    │  │ simpelv2-production │              │
│  │ • 1 replica         │  │ • 3 replicas (HA)   │              │
│  │ • PERMISSIVE mTLS   │  │ • STRICT mTLS       │              │
│  │ • Debug logging     │  │ • Info logging      │              │
│  └─────────────────────┘  └─────────────────────┘              │
└─────────────────────────────────────────────────────────────────┘
```

## 📁 Directory Structure

```
infra/k8s/
├── base/                           # Shared base resources
│   ├── backend/                    # Backend microservices (Axum)
│   │   ├── layanan-daskrimti-portal.yaml
│   │   ├── layanan-pembinaan-perlengkapan.yaml
│   │   └── kustomization.yaml
│   ├── frontend/                   # Frontend microfrontends (Leptos WASM)
│   │   ├── portal.yaml
│   │   ├── daskrimti-portal.yaml
│   │   ├── pembinaan-perlengkapan.yaml
│   │   └── kustomization.yaml
│   ├── infrastructure/             # Core infrastructure
│   │   ├── postgres.yaml           # PostgreSQL with Patroni HA
│   │   ├── redis.yaml              # Redis cache
│   │   ├── authenc.yaml            # Identity Provider (gRPC)
│   │   ├── secreton.yaml           # Secrets Vault (gRPC)
│   │   └── kustomization.yaml
│   ├── configmaps/                 # Configuration
│   │   ├── simpelv2-config.yaml
│   │   ├── backend-config.yaml
│   │   └── kustomization.yaml
│   ├── secrets/                    # Secrets (base64 encoded)
│   │   ├── secrets.yaml
│   │   └── kustomization.yaml
│   ├── istio/                      # Istio service mesh
│   │   ├── gateway.yaml            # Ingress Gateway
│   │   ├── virtual-service.yaml    # Route definitions
│   │   ├── destination-rules.yaml  # mTLS settings
│   │   ├── peer-authentication.yaml
│   │   └── kustomization.yaml
│   ├── network-policies/           # Network security
│   │   ├── network-policies.yaml
│   │   └── kustomization.yaml
│   ├── metallb/                    # MetalLB (apply separately)
│   │   ├── ip-address-pool.yaml    # prod-addresspool, staging-addresspool
│   │   ├── l2-advertisement.yaml   # prod/staging L2 advertisements
│   │   └── kustomization.yaml
│   ├── monitoring/                 # Optional: Prometheus ServiceMonitors
│   │   ├── service-monitors.yaml
│   │   └── kustomization.yaml
│   └── kustomization.yaml          # Main base config
│
├── overlays/
│   ├── staging/                    # Staging environment
│   │   ├── namespace.yaml          # simpelv2-staging
│   │   ├── hpa.yaml                # Min 1, Max 2 replicas
│   │   ├── pdb.yaml                # maxUnavailable: 1
│   │   ├── mtls.yaml               # STRICT mTLS for staging
│   │   └── kustomization.yaml
│   └── production/                 # Production environment
│       ├── namespace.yaml          # simpelv2-production
│       ├── hpa.yaml                # Min 3, Max 10 replicas
│       ├── pdb.yaml                # minAvailable: 2
│       ├── resource-quota.yaml     # Resource limits
│       ├── security-context-patch.yaml  # PodSecurity compliance
│       └── kustomization.yaml
│
├── deploy.sh                       # Deployment automation script
├── README.md                       # User documentation
└── AGENTS.md                       # This file (AI agent guide)
```

## 🔧 Cluster Components

### Enabled MicroK8s Addons

| Addon | Purpose | Status |
|-------|---------|--------|
| `dns` | CoreDNS for service discovery | ✅ Enabled |
| `storage` | Default storage class | ✅ Enabled |
| `registry` | Container registry at :32000 | ✅ Enabled |
| `metallb` | LoadBalancer (172.15.10.200-250) | ✅ Enabled |
| `istio` | Service mesh with mTLS | ✅ Enabled |
| `longhorn` | Distributed block storage | ✅ Enabled |

### Storage Classes

| Name | Provisioner | Default | Use Case |
|------|-------------|---------|----------|
| `longhorn` | driver.longhorn.io | ✅ Yes | Production PVCs |
| `longhorn-static` | driver.longhorn.io | No | Static provisioning |
| `microk8s-hostpath` | microk8s.io/hostpath | No | Development only |

### MetalLB IP Pools

| Pool | Range | Purpose |
|------|-------|---------|
| `prod-addresspool` | 172.15.10.200-230 | Production LoadBalancer (31 IPs) |
| `staging-addresspool` | 172.15.10.231-250, 10.1.7.121/32 | Staging/Testing (21 IPs) |

> **Note**: MetalLB is for LoadBalancer services only. Pod networking uses Calico CNI (10.1.x.x subnet).

## 📊 Environment Comparison

| Feature | Staging | Production |
|---------|---------|------------|
| **Namespace** | `simpelv2-staging` | `simpelv2-production` |
| **MetalLB Pool** | `staging-addresspool` | `prod-addresspool` |
| **Replicas** | 1 | 3 (HA) |
| **HPA Min/Max** | 1/2 | 3/10 |
| **mTLS Mode** | STRICT | STRICT |
| **PodSecurity** | - | restricted |
| **ResourceQuota** | - | CPU: 20/40, Mem: 40Gi/80Gi |
| **PDB** | maxUnavailable: 1 | minAvailable: 2 |
| **Logging** | debug | info |
| **Image Tags** | :staging, :stag | :v0.1.0 |

## 🚀 Deployment Commands

### Using deploy.sh (Recommended)

```bash
cd /home/anbud02/simpel2/infra/k8s

# Preview changes (dry-run)
./deploy.sh staging diff
./deploy.sh production diff

# Deploy
./deploy.sh staging apply
./deploy.sh production apply

# Delete environment
./deploy.sh staging delete
./deploy.sh production delete
```

### Using kubectl directly

```bash
# Validate kustomization
kubectl kustomize overlays/staging
kubectl kustomize overlays/production

# Apply
kubectl apply -k overlays/staging
kubectl apply -k overlays/production

# Delete
kubectl delete -k overlays/staging
kubectl delete -k overlays/production
```

### MetalLB (apply separately to metallb-system)

```bash
kubectl apply -k base/metallb/
```

## 🔍 Verification Commands

```bash
# Check all namespaces
kubectl get namespaces | grep simpelv2

# Check resources in staging
kubectl get all -n simpelv2-staging

# Check resources in production
kubectl get all -n simpelv2-production

# Check HPA status
kubectl get hpa -A | grep simpelv2

# Check PDB status
kubectl get pdb -A | grep simpelv2

# Check Istio resources
kubectl get gateway,virtualservice,destinationrule,peerauthentication -A

# Check storage
kubectl get pvc -A | grep simpelv2

# Check MetalLB
kubectl get ipaddresspool,l2advertisement -n metallb-system

# Check Longhorn
kubectl get pods -n longhorn-system
```

## 🏷️ Labeling Convention

All resources use Kubernetes recommended labels:

```yaml
labels:
  app.kubernetes.io/name: <component-name>
  app.kubernetes.io/component: frontend|backend|infrastructure|database
  app.kubernetes.io/part-of: simpelv2
  app.kubernetes.io/managed-by: kustomize
  app.kubernetes.io/version: "0.1.0"
  app.kubernetes.io/environment: staging|production  # Added by overlay
```

## 🔐 Security Configuration

### Network Policies

| Policy | From | To | Purpose |
|--------|------|-----|---------|
| `default-deny-ingress` | * | * | Block all by default |
| `allow-frontend-to-backend` | frontend | backend | REST API calls |
| `allow-backend-to-authenc` | backend | authenc | gRPC auth |
| `allow-backend-to-secreton` | backend | secreton | gRPC secrets |
| `allow-backend-to-infra` | backend | postgres/redis | Data access |
| `allow-istio-sidecar` | istio-system | * | Mesh traffic |

### mTLS Configuration

| Environment | Mode | Note |
|-------------|------|------|
| Staging | STRICT | mTLS enforced for production parity |
| Production | STRICT | Enforces mutual TLS only |

### PodSecurity Standards (Production)

```yaml
pod-security.kubernetes.io/enforce: restricted
pod-security.kubernetes.io/audit: restricted
pod-security.kubernetes.io/warn: restricted
```

## 🖼️ Container Images

All images are stored in the local MicroK8s registry:

```
localhost:32000/simpelv2/<image-name>:<tag>
```

| Image | Port | Type | Purpose |
|-------|------|------|---------|
| `portal` | 8080 | Deployment | Main portal frontend |
| `portal` | 8080 | Deployment | Portal microfrontend |
| `pembinaan-perlengkapan` | 8080 | Deployment | Pembinaan microfrontend |
| `layanan-portal` | 3010 | Deployment | Portal backend API |
| `layanan-daskrimti-integrasi` | - | CronJob | External API integration (MonSAKTI/MySIMKARI/SIMAN) |
| `layanan-pembinaan-perlengkapan` | 3020 | Deployment | Perlengkapan backend API |
| `authenc` | 8088/9088/9090 | Deployment | Identity Provider |
| `secreton` | 8200/9000/9090 | StatefulSet | Secrets Vault |

## 📝 Common Tasks

### Adding a New Backend Service

1. Create `base/backend/<service-name>.yaml` (copy from existing)
2. Add to `base/backend/kustomization.yaml`
3. Add NetworkPolicy in `base/network-policies/network-policies.yaml`
4. Add route in `base/istio/virtual-service.yaml`
5. Add HPA in overlay if needed

### Adding a New Frontend Microfrontend

1. Create `base/frontend/<mf-name>.yaml`
2. Add to `base/frontend/kustomization.yaml`
3. Add route in `base/istio/virtual-service.yaml`
4. Add HPA in overlay if needed

### Scaling Resources

```bash
# Manual scale (temporary)
kubectl scale deployment/<name> --replicas=5 -n simpelv2-production

# Permanent: Update HPA in overlay
# overlays/production/hpa.yaml
```

### Updating Secrets

```bash
# Generate base64
echo -n "new-password" | base64

# Update base/secrets/secrets.yaml
# Then apply:
kubectl apply -k overlays/production
```

## ⚠️ Important Notes

1. **DO NOT** edit resources directly in cluster - use Kustomize
2. **ImagePullBackOff** is expected until images are pushed to registry
3. **simpelv2** namespace (old) still exists for development - migrate gradually
4. **MetalLB** config is separate - apply with `kubectl apply -k base/metallb/`
5. **Monitoring** requires prometheus-operator CRDs - uncomment in base when ready

## 🔗 Related Documentation

- [Main AGENTS.md](/home/anbud02/simpel2/AGENTS.md) - Project overview
- [Authenc AGENTS.md](/home/anbud02/simpel2/infra/authenc/AGENTS.md) - Identity Provider
- [Secreton AGENTS.md](/home/anbud02/simpel2/infra/secreton/AGENTS.md) - Secrets Vault
- [README.md](./README.md) - User documentation

## 📞 Troubleshooting

### 🔴 CRITICAL: Istio Routing Returns 404 "route_not_found"

**Symptoms:** Ingress gateway returns 404 with `route_not_found` in logs despite VirtualService existing.

**Root Cause:** VirtualService in application namespace (e.g., `simpelv2-production`) cannot attach to Gateway in `istio-system` namespace unless gateway reference is namespace-qualified.

**Solution:**
```bash
# Option 1: Create VirtualService in istio-system namespace (RECOMMENDED)
# Use FQDNs for destinations since services are in different namespace
kubectl apply -f infra/k8s/overlays/production/production-istio.yaml  # Pre-configured file (apply from overlays)

# Option 2: Use namespace-qualified gateway reference in VirtualService
spec:
  gateways:
    - istio-system/simpelv2-dev-gateway  # Must include namespace!
```

**Verification:**
```bash
# Check if VirtualService is attached to Gateway
INGRESS_POD=$(kubectl get pods -n istio-system -l istio=ingressgateway -o jsonpath='{.items[0].metadata.name}')
kubectl exec -n istio-system $INGRESS_POD -- curl -s localhost:15000/config_dump | jq '.configs[]? | select(.["@type"]=="type.googleapis.com/envoy.admin.v3.RoutesConfigDump") | .dynamic_route_configs[]? | select(.route_config.name=="http.8080") | .route_config.virtual_hosts[]?.name'

# Should show actual host names, NOT "blackhole:80"
```

### 🔴 Database Connection Failed (CrashLoopBackOff)

**Symptoms:** Pods crash with "connection refused" or DNS lookup failures when connecting to PostgreSQL.

**Root Cause:**
1. ClusterIP services may not route correctly due to endpoint selector mismatch
2. Service selectors may not match actual pod labels

**Solution:**
```bash
# Check service endpoints
kubectl get endpoints postgres-primary -n simpelv2-production

# If no endpoints, check pod labels vs service selector
kubectl get pods -n simpelv2-production -l app.kubernetes.io/name=postgres --show-labels
kubectl get svc postgres-primary -n simpelv2-production -o jsonpath='{.spec.selector}'

# Use headless service instead of ClusterIP
# In DATABASE_URL, use 'postgres' (headless) instead of 'postgres-primary' (ClusterIP)
DATABASE_URL=postgres://simpelv2:password@postgres:5432/simpelv2
```

### 🔴 ImagePullBackOff on Master Node

**Symptoms:** Pods on master node stuck in `ImagePullBackOff` while same images work on worker.

**Root Cause:** Master node containerd missing `unpigz` binary for parallel decompression.

**Solution:**
```bash
# Option 1: Install unpigz on master
ssh root@simpel.kejaksaan.go.id "apt-get update && apt-get install -y pigz"

# Option 2 (RECOMMENDED): Schedule all pods on worker node
# Add nodeSelector to all deployments
spec:
  template:
    spec:
      nodeSelector:
        kubernetes.io/hostname: simple02

# Apply in kustomization.yaml as patch:
patches:
  - target:
      kind: Deployment
    patch: |-
      - op: add
        path: /spec/template/spec/nodeSelector
        value:
          kubernetes.io/hostname: simple02
```

### 🔴 PVC ReadWriteOnce Attachment Issues

**Symptoms:** Pod stuck in `ContainerCreating` with "volume is already attached to another node".

**Root Cause:** RWO volumes can only be attached to one node at a time.

**Solution:**
```bash
# Find pods using the PVC
kubectl get pods -A -o json | jq -r '.items[] | select(.spec.volumes[]?.persistentVolumeClaim.claimName == "redis-data") | .metadata.name + " in " + .metadata.namespace'

# Delete pods on wrong node
kubectl delete pod <pod-name> -n <namespace> --force --grace-period=0

# Wait for volume to detach, then new pod will schedule on correct node
```

### 🟡 Kustomize Namespace Transformation Overwrites istio-system

**Symptoms:** Gateway and VirtualService get placed in application namespace instead of istio-system.

**Root Cause:** Kustomize's `namespace:` field transforms ALL resource namespaces.

**Solution:**
```yaml
# DON'T include istio/ resources in base for overlays with different namespace
# Instead, manage Istio routing separately:

# 1. Create dedicated Istio config files
#    - overlays/production/production-istio.yaml  (for simpelv2-production)
#    - overlays/staging/staging-istio.yaml     (for simpelv2-staging)

# 2. Apply them separately (not via Kustomize)
kubectl apply -f infra/k8s/overlays/production/production-istio.yaml
kubectl apply -f infra/k8s/overlays/staging/overlays/staging/staging-istio.yaml
```

### 🟡 Multiple Gateways Competing for Same Port

**Symptoms:** Only one Gateway works, others show routes as "blackhole".

**Root Cause:** When multiple Gateways use `hosts: ["*"]` on same port, only one wins.

**Solution:**
```yaml
# Use specific hosts for each environment
# Production Gateway:
hosts:
  - "simpel.kejaksaan.go.id"

# Staging Gateway:
hosts:
  - "10.1.7.121"
  - "staging.simpel.internal"
```

### Pods stuck in ImagePullBackOff

```bash
# Check if image exists in registry
curl -s http://localhost:32000/v2/_catalog | jq

# Build and push image
docker build -t localhost:32000/simpelv2/<name>:staging .
docker push localhost:32000/simpelv2/<name>:staging
```

### Istio sidecar not injecting

```bash
# Check namespace label
kubectl get ns <namespace> --show-labels | grep istio-injection

# Add label if missing
kubectl label namespace <namespace> istio-injection=enabled
```

### PVC stuck in Pending

```bash
# Check Longhorn status
kubectl get pods -n longhorn-system

# Check storage class
kubectl get sc

# Check Longhorn UI
kubectl port-forward -n longhorn-system svc/longhorn-frontend 8080:80
```

### MetalLB not assigning IP

```bash
# Check speaker pods
kubectl get pods -n metallb-system

# Check IPAddressPool
kubectl get ipaddresspool -n metallb-system -o yaml

# Apply updated MetalLB config
kubectl apply -k base/metallb/
```

### Cross-pod connectivity issues

```bash
# Check Calico status
kubectl get pods -n kube-system -l k8s-app=calico-node

# Test connectivity from a pod
kubectl run test-net --rm -it --restart=Never --image=busybox -- nc -vz <target-svc> <port>

# Check NetworkPolicies
kubectl get networkpolicies -n <namespace>

# Disable Istio sidecar for database pods if needed
# Add annotation: sidecar.istio.io/inject: "false"
```

---

## 📖 Related AGENTS.md Files

| Component | Location | Purpose |
|-----------|----------|---------|
| **Root Project** | [`/AGENTS.md`](../../AGENTS.md) | Main codebase conventions, Rust patterns |
| **Authenc** | [`/infra/authenc/AGENTS.md`](../authenc/AGENTS.md) | Identity Provider service |
| **Secreton** | [`/infra/secreton/AGENTS.md`](../secreton/AGENTS.md) | Secrets Management service |
| **Layanan Integrasi** | [`/layanan/daskrimti/integrasi/AGENTS.md`](../../layanan/daskrimti/integrasi/AGENTS.md) | Government API integration |

---

## 🚀 Staging to Production Promotion

### Environment Separation

| Aspect | Staging | Production |
|--------|---------|------------|
| **Namespace** | `simpelv2-staging` | `simpelv2-production` |
| **Host/Domain** | `10.1.7.121` | `simpel.kejaksaan.go.id` |
| **VirtualService** | `simpelv2-staging-routes` | `simpelv2-prod-routes` |
| **Replicas** | 1 | 3 (HA) |
| **Image Tags** | `:stag` | `:prod` |
| **Logging** | `debug` | `info` |
| **mTLS** | PERMISSIVE | STRICT |

### Promotion Workflow

```bash
# 1. Build and tag images for production
docker tag localhost:32000/simpelv2/<image>:stag localhost:32000/simpelv2/<image>:prod
docker push localhost:32000/simpelv2/<image>:prod

# 2. Export image and import to worker node (if not using registry)
docker save localhost:32000/simpelv2/<image>:prod | microk8s ctr --address /var/snap/microk8s/common/run/containerd.sock images import -

# 3. Apply production overlay
kubectl apply -k overlays/production

# 4. Apply Istio routing (separate from Kustomize)
kubectl apply -f infra/k8s/overlays/production/production-istio.yaml

# 5. Verify deployment
kubectl get pods -n simpelv2-production
curl -s http://172.15.10.200/ -H "Host: simpel.kejaksaan.go.id"
```

### Istio Routing Configuration

```yaml
# Routing is managed SEPARATELY from Kustomize (to avoid namespace transformation issues)
# Files:
#   - overlays/production/production-istio.yaml  -> Routes to simpelv2-production services
#   - overlays/staging/staging-istio.yaml       -> Routes to simpelv2-staging services

# Both use the same Gateway: simpelv2-dev-gateway (in istio-system)
# Differentiated by hosts:
#   - Production: ["*"]  (matches simpel.kejaksaan.go.id and all other hosts)
#   - Staging: ["10.1.7.121"]  (matches only staging IP)
```

### Quick Promotion Commands

```bash
# Promote a specific image from staging to production
IMAGE=layanan-daskrimti-portal

# Tag staging as production
docker pull localhost:32000/simpelv2/${IMAGE}:stag
docker tag localhost:32000/simpelv2/${IMAGE}:stag localhost:32000/simpelv2/${IMAGE}:prod
docker push localhost:32000/simpelv2/${IMAGE}:prod

# Import to containerd on worker node
docker save localhost:32000/simpelv2/${IMAGE}:prod | \
  ssh simple02 "microk8s ctr --address /var/snap/microk8s/common/run/containerd.sock images import -"

# Rolling restart in production
kubectl rollout restart deployment/${IMAGE} -n simpelv2-production
kubectl rollout status deployment/${IMAGE} -n simpelv2-production
```

### Rollback

```bash
# If production deployment fails, rollback
kubectl rollout undo deployment/<name> -n simpelv2-production

# Or redeploy staging image to production
kubectl set image deployment/<name> <container>=localhost:32000/simpelv2/<image>:stag -n simpelv2-production
```

---

*Last Updated: 2026-02-03*
*Maintained by: SIMPelv2 DevOps Team*
