# SIMPelv2 MicroK8s Deployment

Konfigurasi lengkap untuk deployment SIMPelv2 di MicroK8s dengan arsitektur microfrontend dan microservices yang menggunakan **gRPC communication**, **Istio service mesh**, **mTLS**, **Envoy Gateway**, dan **SealedSecrets**.

## 📋 Daftar Isi

- [Arsitektur](#arsitektur)
- [Prasyarat](#prasyarat)
- [Struktur File](#struktur-file)
- [Deployment](#deployment)
- [Monitoring](#monitoring)
- [Maintenance](#maintenance)
- [Troubleshooting](#troubleshooting)

## 🏗️ Arsitektur

### Komponen Utama

**Frontend Microfrontends (11 total):**
- Portal - Gateway utama
- Badiklat, Datun, Intel, Pemulihan Aset, Pengawasan - Divisi khusus
- PIDMIL, PIDSUS, PIDUM - Divisi pidana
- Keuangan, Perencanaan, Perlengkapan - Divisi pembinaan

**Backend Microservices (gRPC Only):**
- Gerbang - API Gateway (HTTP:8080, gRPC:9080)
- Layanan Shared: AI, Bantuan, Dasbor, Dokumen, Integrasi, Konfigurasi, Laporan, Notifikasi (HTTP:3000, gRPC:9000)

**Infrastructure:**
- PostgreSQL - Database per service/schema dengan deadpool-postgres
- Redis - Cache dan session
- Envoy Gateway - Advanced API Gateway dengan TLS termination
- Authenc - Identity & Access Management (HTTP:8088, gRPC:9090)
- Secreton - Secret management (HTTP:8200, gRPC:9200)

**Service Mesh:**
- Istio - Full service mesh dengan sidecar proxy di semua pod
- mTLS - Mutual TLS antar semua microservice
- Envoy Proxy - Sidecar proxy untuk traffic management

**Security:**
- SealedSecrets - Encrypted secret management untuk Kubernetes
- TLS/HTTPS - Wajib untuk semua komunikasi
- mTLS - Mutual authentication antar service

**Monitoring Stack:**
- Prometheus - Metrics collection
- Grafana - Visualization
- Loki - Log aggregation
- Promtail - Log shipping
- AlertManager - Alert management

### Namespace Organization

```
simpelv2/                    # Main namespace
simpelv2-frontend/           # Frontend microfrontends
simpelv2-backend/            # Backend microservices
simpelv2-infra/              # Infrastructure services
simpelv2-monitoring/         # Monitoring stack
```

## 📋 Prasyarat

### MicroK8s Setup

```bash
# Install MicroK8s
sudo snap install microk8s --classic

# Add user to microk8s group
sudo usermod -a -G microk8s $USER
sudo chown -f -R $USER ~/.kube
newgrp microk8s

# Enable required addons
microk8s enable dns storage ingress metrics-server prometheus

# Install Istio
curl -L https://istio.io/downloadIstio | sh -
cd istio-*
export PATH=$PWD/bin:$PATH
istioctl install --set values.defaultRevision=default -y

# Install Envoy Gateway
kubectl apply -f https://github.com/envoyproxy/gateway/releases/download/v0.6.0/install.yaml

# Install SealedSecrets Controller
kubectl apply -f https://github.com/bitnami-labs/sealed-secrets/releases/download/v0.24.0/controller.yaml

# Verify installation
microk8s status --wait-ready
kubectl get pods -n istio-system
kubectl get pods -n envoy-gateway-system
kubectl get pods -n sealed-secrets
```

### System Requirements

- **CPU:** Minimum 8 cores (16 cores recommended)
- **Memory:** Minimum 16GB RAM (32GB recommended)
- **Storage:** Minimum 100GB available space
- **Network:** Stable internet connection for image pulls

### Domain Configuration

Pastikan domain berikut mengarah ke cluster:
- `simpel.kejaksaan.go.id` - **Single domain untuk semua akses**

### URL Structure (Path-based Routing)

```
simpel.kejaksaan.go.id/                    → Portal (main dashboard)
simpel.kejaksaan.go.id/badiklat            → Badiklat microfrontend
simpel.kejaksaan.go.id/datun               → Datun microfrontend
simpel.kejaksaan.go.id/intel               → Intel microfrontend
simpel.kejaksaan.go.id/pengawasan          → Pengawasan microfrontend
simpel.kejaksaan.go.id/pemulihan-aset      → Pemulihan Aset microfrontend
simpel.kejaksaan.go.id/pidum               → Pidum microfrontend
simpel.kejaksaan.go.id/pidsus              → Pidsus microfrontend
simpel.kejaksaan.go.id/pidmil              → Pidmil microfrontend
simpel.kejaksaan.go.id/pembinaan/keuangan  → Keuangan microfrontend
simpel.kejaksaan.go.id/pembinaan/perencanaan → Perencanaan microfrontend
simpel.kejaksaan.go.id/pembinaan/perlengkapan → Perlengkapan microfrontend
simpel.kejaksaan.go.id/api/                → API Gateway
simpel.kejaksaan.go.id/auth/               → Authentication
simpel.kejaksaan.go.id/monitoring/grafana  → Grafana (admin only)
simpel.kejaksaan.go.id/monitoring/prometheus → Prometheus (admin only)
```

## 📁 Struktur File

```
infra/k8s/
├── 00-istio-setup.yaml                  # Istio service mesh setup
├── 00-namespace.yaml                    # Namespace definitions
├── 01-configmap.yaml                    # Configuration data
├── 01-sealed-secrets.yaml               # SealedSecrets controller & secrets
├── 02-secrets.yaml                      # Legacy secrets (use SealedSecrets instead)
├── 03-persistent-volumes.yaml           # Storage configuration
├── 04-infrastructure-deployments.yaml  # Core infrastructure
├── 05-backend-deployments.yaml         # Backend services (legacy)
├── 05-backend-deployments-grpc.yaml    # Backend services with gRPC
├── 06-frontend-deployments.yaml        # Frontend services
├── 07-services.yaml                     # Service discovery (legacy)
├── 07-services-grpc.yaml               # Service discovery with gRPC
├── 08-monitoring-deployments.yaml      # Monitoring stack
├── 09-monitoring-services.yaml         # Monitoring services
├── 10-monitoring-config.yaml           # Monitoring configuration
├── 11-ingress.yaml                      # Legacy ingress (use Istio instead)
├── 12-network-policies.yaml            # Security policies
├── 13-hpa.yaml                          # Auto-scaling
├── 14-service-monitors.yaml            # Prometheus targets
├── 15-pod-disruption-budgets.yaml      # High availability
├── 16-deployment-script.yaml           # Deployment automation
├── 17-istio-virtual-services.yaml      # Istio traffic routing (legacy)
├── 18-istio-destination-rules.yaml     # Istio traffic policies
├── 19-envoy-gateway.yaml               # Envoy Gateway configuration (legacy)
├── 20-single-domain-routing.yaml       # Single domain path-based routing
├── 21-frontend-module-federation.yaml  # Microfrontend configuration
└── README.md                            # This file
```

## 🚀 Deployment

### Prerequisites Setup

```bash
# Install kubeseal CLI for SealedSecrets
wget https://github.com/bitnami-labs/sealed-secrets/releases/download/v0.24.0/kubeseal-0.24.0-linux-amd64.tar.gz
tar -xvzf kubeseal-0.24.0-linux-amd64.tar.gz
sudo install -m 755 kubeseal /usr/local/bin/kubeseal

# Create sealed secrets from your actual secrets
echo -n mypassword | kubectl create secret generic mysecret --dry-run=client --from-file=password=/dev/stdin -o yaml | kubeseal -o yaml > mysealedsecret.yaml
```

### Quick Start

```bash
# Clone repository
git clone <repository-url>
cd simpelv2/infra/k8s

# Setup Istio and dependencies first
kubectl apply -f 00-istio-setup.yaml
kubectl apply -f 01-sealed-secrets.yaml

# Wait for Istio to be ready
kubectl wait --for=condition=Ready pods --all -n istio-system --timeout=300s

# Deploy everything with gRPC support
./deploy-grpc.sh
```

### Manual Deployment with gRPC & Service Mesh

```bash
# 1. Setup Istio Service Mesh
microk8s kubectl apply -f 00-istio-setup.yaml
microk8s kubectl apply -f 00-namespace.yaml

# 2. Setup SealedSecrets
microk8s kubectl apply -f 01-sealed-secrets.yaml

# Wait for SealedSecrets controller
microk8s kubectl wait --for=condition=Ready pods --all -n sealed-secrets --timeout=300s

# 3. Apply configurations
microk8s kubectl apply -f 01-configmap.yaml
microk8s kubectl apply -f 03-persistent-volumes.yaml

# Wait for storage to be ready
microk8s kubectl wait --for=condition=Bound pvc --all --timeout=300s

# 4. Deploy infrastructure with gRPC support
microk8s kubectl apply -f 04-infrastructure-deployments.yaml
microk8s kubectl apply -f 07-services-grpc.yaml

# Wait for infrastructure to be ready
microk8s kubectl wait --for=condition=Ready pods --all -n simpelv2-infra --timeout=600s

# 5. Deploy backend services with gRPC
microk8s kubectl apply -f 05-backend-deployments-grpc.yaml

# Wait for backend to be ready
microk8s kubectl wait --for=condition=Ready pods --all -n simpelv2-backend --timeout=300s

# 6. Deploy frontend services
microk8s kubectl apply -f 06-frontend-deployments.yaml

# 7. Deploy monitoring
microk8s kubectl apply -f 08-monitoring-deployments.yaml
microk8s kubectl apply -f 09-monitoring-services.yaml
microk8s kubectl apply -f 10-monitoring-config.yaml

# 8. Configure Istio traffic management
microk8s kubectl apply -f 17-istio-virtual-services.yaml
microk8s kubectl apply -f 18-istio-destination-rules.yaml

# 9. Setup Envoy Gateway
microk8s kubectl apply -f 19-envoy-gateway.yaml

# 10. Configure security and policies
microk8s kubectl apply -f 12-network-policies.yaml

# 11. Enable auto-scaling and high availability
microk8s kubectl apply -f 13-hpa.yaml
microk8s kubectl apply -f 14-service-monitors.yaml
microk8s kubectl apply -f 15-pod-disruption-budgets.yaml
```

### Verification

```bash
# Check all pods are running
microk8s kubectl get pods --all-namespaces

# Check services
microk8s kubectl get svc --all-namespaces

# Check ingress
microk8s kubectl get ingress --all-namespaces

# Check HPA status
microk8s kubectl get hpa --all-namespaces
```

## 📊 Monitoring

### Access Monitoring Dashboard

```bash
# Get Grafana admin password
microk8s kubectl get secret monitoring-secrets -n simpelv2-monitoring -o jsonpath='{.data.GRAFANA_ADMIN_PASSWORD}' | base64 -d

# Access via ingress
https://monitoring.simpelv2.kejaksaan.go.id/grafana
```

### Key Metrics

- **Application Performance:** Response times, error rates
- **Infrastructure Health:** CPU, memory, disk usage
- **Database Performance:** Connection pools, query performance
- **Network Traffic:** Request volume, bandwidth usage

### Alerts

Configured alerts for:
- High CPU/memory usage (>80%)
- Service downtime
- Database connection issues
- High error rates (>10%)
- Disk space usage (>85%)

## 🔧 Maintenance

### Updates

```bash
# Update specific component
./update.sh frontend    # Update all frontend services
./update.sh backend     # Update all backend services
./update.sh monitoring  # Update monitoring stack
./update.sh infra       # Update infrastructure

# Update all components
./update.sh all
```

### Scaling

```bash
# Manual scaling
microk8s kubectl scale deployment portal --replicas=5 -n simpelv2-frontend

# Check HPA status
microk8s kubectl get hpa -n simpelv2-frontend
```

### Backup

```bash
# Backup persistent data
microk8s kubectl exec -n simpelv2-infra postgres-xxx -- pg_dump -U simpelv2_user simpelv2_prod > backup.sql

# Backup secrets and configs
microk8s kubectl get secrets,configmaps --all-namespaces -o yaml > configs-backup.yaml
```

### Rollback

```bash
# Complete rollback (removes everything)
./rollback.sh

# Rollback specific deployment
microk8s kubectl rollout undo deployment/portal -n simpelv2-frontend
```

## 🔍 Troubleshooting

### Common Issues

**Pods stuck in Pending:**
```bash
# Check node resources
microk8s kubectl describe nodes

# Check PVC status
microk8s kubectl get pvc --all-namespaces

# Check events
microk8s kubectl get events --sort-by=.metadata.creationTimestamp
```

**Service not accessible:**
```bash
# Check service endpoints
microk8s kubectl get endpoints -n <namespace>

# Check ingress status
microk8s kubectl describe ingress -n simpelv2-infra

# Check network policies
microk8s kubectl get networkpolicy --all-namespaces
```

**Database connection issues:**
```bash
# Check PostgreSQL logs
microk8s kubectl logs -n simpelv2-infra deployment/postgres

# Test database connectivity
microk8s kubectl exec -n simpelv2-infra deployment/postgres -- psql -U simpelv2_user -d simpelv2_prod -c "SELECT 1"
```

**High resource usage:**
```bash
# Check resource usage
microk8s kubectl top nodes
microk8s kubectl top pods --all-namespaces

# Check HPA status
microk8s kubectl get hpa --all-namespaces
```

### Logs

```bash
# Application logs
microk8s kubectl logs -n simpelv2-frontend deployment/portal -f

# Infrastructure logs
microk8s kubectl logs -n simpelv2-infra deployment/postgres -f

# Monitoring logs
microk8s kubectl logs -n simpelv2-monitoring deployment/prometheus -f

# All logs for a namespace
microk8s kubectl logs -n simpelv2-backend --all-containers=true -f
```

### Debug Commands

```bash
# Get detailed pod information
microk8s kubectl describe pod <pod-name> -n <namespace>

# Execute commands in pod
microk8s kubectl exec -it <pod-name> -n <namespace> -- /bin/bash

# Port forward for local access
microk8s kubectl port-forward -n simpelv2-frontend svc/portal-service 8080:8080

# Check resource quotas
microk8s kubectl describe resourcequota -n <namespace>
```

## 🔐 Security

### Service Mesh Security (Istio)

- **mTLS Strict Mode**: Semua komunikasi antar service menggunakan mutual TLS
- **Sidecar Proxy**: Setiap pod memiliki Envoy proxy untuk traffic management
- **Zero Trust**: Default deny dengan explicit allow rules
- **Certificate Management**: Automatic certificate rotation oleh Istio

### gRPC Communication

- **gRPC Only**: Komunikasi antar backend service hanya menggunakan gRPC (port 9000-9200)
- **HTTP Fallback**: HTTP hanya untuk browser access ke frontend
- **Performance**: gRPC memberikan performa tinggi dengan binary protocol
- **Type Safety**: Strong typing dengan Protocol Buffers

### SealedSecrets

- **Encrypted Secrets**: Semua secret dienkripsi dengan SealedSecrets
- **GitOps Safe**: Secret terenkripsi aman disimpan di Git
- **Controller**: SealedSecrets controller mendekripsi secret di cluster
- **Key Rotation**: Automatic key rotation setiap 30 hari

### TLS/HTTPS Everywhere

- **Envoy Gateway**: TLS termination di edge dengan Envoy
- **Internal mTLS**: Mutual TLS untuk komunikasi internal
- **Certificate Management**: Automatic certificate provisioning
- **HSTS**: HTTP Strict Transport Security headers

### Database Security

- **Database per Service**: Setiap service memiliki database/schema terpisah
- **Connection Pooling**: deadpool-postgres untuk efficient connection management
- **Encrypted Connections**: TLS untuk database connections
- **Credential Isolation**: Separate database users per service

### Network Policies

- Default deny all ingress traffic
- Explicit allow rules untuk required communication
- Istio AuthorizationPolicy untuk fine-grained access control
- DNS resolution allowed untuk all pods

### RBAC

- Service accounts dengan minimal required permissions
- Cluster roles untuk cross-namespace access
- Istio RBAC untuk service-to-service authorization
- Regular audit of permissions

## 📞 Support

Untuk bantuan lebih lanjut:

1. **Dokumentasi:** Lihat file README di setiap service
2. **Monitoring:** Cek dashboard Grafana untuk metrics
3. **Logs:** Gunakan Loki untuk log aggregation
4. **Alerts:** AlertManager akan mengirim notifikasi untuk issues

## 🔄 Continuous Integration

File konfigurasi ini dirancang untuk:
- GitOps workflow dengan ArgoCD/Flux
- Automated testing dengan CI/CD pipeline
- Blue-green deployments
- Canary releases

Pastikan untuk mengintegrasikan dengan CI/CD pipeline organisasi untuk deployment otomatis.
