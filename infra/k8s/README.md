# SIMPEL Kubernetes Kustomize Structure

## Overview

This directory contains Kubernetes manifests organized using **Kustomize** following GitOps best practices with base/overlay pattern for multi-environment deployments.

## Directory Structure

```
infra/k8s/
├── base/                           # Shared base configuration
│   ├── kustomization.yaml          # Main base kustomization
│   ├── namespaces/                 # Namespace definitions
│   ├── configmaps/                 # ConfigMaps
│   ├── secrets/                    # Secret templates
│   ├── infrastructure/             # PostgreSQL, Redis, Authenc, Secreton
│   ├── backend/                    # Backend services (layanan-*)
│   ├── frontend/                   # Frontend deployments (portal, microfrontends)
│   ├── network-policies/           # Network security policies
│   └── istio/                      # Istio Gateway, VirtualService, DestinationRules
│
└── overlays/                       # Environment-specific overlays
    ├── staging/                    # Staging environment
    │   ├── kustomization.yaml      # Staging patches
    │   ├── namespace.yaml          # simpelv2-staging namespace
    │   ├── hpa.yaml                # Minimal HPA
    │   └── pdb.yaml                # Minimal PDB
    │
    └── production/                 # Production environment
        ├── kustomization.yaml      # Production patches
        ├── namespace.yaml          # simpelv2-production namespace
        ├── hpa.yaml                # Aggressive HPA
        ├── pdb.yaml                # High availability PDB
        └── resource-quota.yaml     # Resource limits
```

## Quick Start

### Prerequisites

- MicroK8s with addons: `dns`, `storage`, `istio`, `metallb`
- `kubectl` configured to access the cluster
- Kustomize (built into kubectl 1.14+)

### Deploy to Staging

```bash
# Preview what will be applied
kubectl kustomize overlays/staging

# Apply staging environment
kubectl apply -k overlays/staging

# Verify deployment
kubectl get all -n simpelv2-staging
```

### Deploy to Production

```bash
# Preview what will be applied
kubectl kustomize overlays/production

# Apply production environment
kubectl apply -k overlays/production

# Verify deployment
kubectl get all -n simpelv2-production
```

### Delete Environment

```bash
# Delete staging
kubectl delete -k overlays/staging

# Delete production (CAREFUL!)
kubectl delete -k overlays/production
```

## Environment Comparison

| Feature | Staging | Production |
|---------|---------|------------|
| **Namespace** | `simpelv2-staging` | `simpelv2-production` |
| **Replicas (Frontend)** | 1 | 3 |
| **Replicas (Backend)** | 1 | 3 |
| **Replicas (PostgreSQL)** | 1 | 3 (HA with Patroni) |
| **Replicas (Authenc)** | 1 | 3 |
| **Replicas (Secreton)** | 1 | 3 |
| **HPA Min** | 1 | 3 |
| **HPA Max** | 2 | 10 |
| **PDB Min Available** | N/A | 2 |
| **mTLS Mode** | PERMISSIVE | STRICT |
| **HTTPS Redirect** | No | Yes |
| **Image Tag** | `staging` | `v0.1.0` |
| **Log Level** | `debug` | `info` |
| **Resource Limits** | Minimal | Production-grade |
| **Resource Quota** | None | Enforced |

## Common Operations

### Scale Deployment

```bash
# Manual scale (temporary, HPA will restore)
kubectl scale deployment/portal -n simpelv2-staging --replicas=2

# Permanent: edit overlay kustomization.yaml
```

### Update Image Tag

```bash
# Edit overlays/production/kustomization.yaml
# Change image tag in 'images' section
images:
  - name: localhost:32000/simpelv2/portal
    newTag: v0.2.0

# Apply
kubectl apply -k overlays/production
```

### View Logs

```bash
# Staging
kubectl logs -f deployment/portal -n simpelv2-staging

# Production (follow specific pod)
kubectl logs -f portal-xxx-yyy -n simpelv2-production
```

### Check HPA Status

```bash
kubectl get hpa -n simpelv2-production
kubectl describe hpa portal-hpa -n simpelv2-production
```

### Check PDB Status

```bash
kubectl get pdb -n simpelv2-production
kubectl describe pdb postgres-pdb -n simpelv2-production
```

## Customization

### Add New Service

1. Create deployment YAML in `base/backend/` or `base/frontend/`
2. Add to `base/backend/kustomization.yaml` or `base/frontend/kustomization.yaml`
3. Add HPA/PDB in overlays if needed
4. Test with `kubectl kustomize overlays/staging`

### Modify ConfigMap

1. Edit `base/configmaps/simpelv2-config.yaml` for base values
2. Override in overlay `configMapGenerator` for environment-specific values

### Add Secret

```bash
# Create sealed secret (recommended for GitOps)
kubectl create secret generic my-secret \
  --from-literal=key=value \
  --dry-run=client -o yaml | \
  kubeseal -o yaml > overlays/production/my-secret-sealed.yaml
```

## Troubleshooting

### Kustomize Build Fails

```bash
# Validate kustomization
kubectl kustomize overlays/staging --enable-alpha-plugins

# Check for missing resources
kustomize build overlays/staging 2>&1 | grep -i error
```

### Pod Not Starting

```bash
# Check events
kubectl describe pod <pod-name> -n simpelv2-staging

# Check logs
kubectl logs <pod-name> -n simpelv2-staging --previous
```

### Network Policy Issues

```bash
# Temporarily disable for debugging
kubectl delete networkpolicy default-deny-ingress -n simpelv2-staging

# Check policy
kubectl get networkpolicies -n simpelv2-staging
```

## Best Practices

1. **Never edit base for environment-specific values** - use overlays
2. **Use configMapGenerator** for environment-specific ConfigMaps
3. **Use secretGenerator** or sealed-secrets for secrets
4. **Test with staging** before production
5. **Use version tags** (v0.1.0) in production, `staging`/`latest` in staging
6. **Apply PDB** to critical services
7. **Set resource requests/limits** on all containers
8. **Enable HPA** for auto-scaling

## Related Documentation

- [Kustomize Reference](https://kustomize.io/)
- [Kubernetes Best Practices](https://kubernetes.io/docs/concepts/configuration/overview/)
- [Istio Traffic Management](https://istio.io/latest/docs/concepts/traffic-management/)
