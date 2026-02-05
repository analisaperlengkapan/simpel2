# Infrastructure Audit Report: SIMPelv2 Kubernetes

**Date:** 2026-05-21
**Scope:** `infra/k8s` directory (Base and Overlays)
**Auditor:** AI Agent

## 1. Executive Summary

The current Infrastructure as Code (IaC) setup for SIMPelv2 is **Effective** and **Structurally Sound**, utilizing modern GitOps patterns (Kustomize) and separating concerns clearly between environments. However, it is **Not Optimal** for a true Production environment due to specific configuration choices that introduce Single Points of Failure (SPoF) and fragility in deployment processes. Efficiency regarding resource allocation is generally good.

## 2. Detailed Assessment

### 2.1 Optimality (Areas for Improvement)

*   **Single Point of Failure (Critical):**
    *   **Finding:** The production overlay (`overlays/production/kustomization.yaml`) pins all critical workloads (PostgreSQL, Secreton, Backend, Frontend) to a specific node (`kubernetes.io/hostname: simple02`).
    *   **Impact:** This negates the high-availability benefits of running 3 replicas. If node `simple02` fails, the entire production environment goes offline.
    *   **Recommendation:** Remove the `nodeSelector` patching for production, or ensure multiple worker nodes are available and labeled correctly to allow the scheduler to distribute pods across failure domains.

*   **Image Pull Strategy (Fragility):**
    *   **Finding:** Production deployments are patched with `imagePullPolicy: Never`.
    *   **Impact:** This requires a manual, imperative process to load container images onto the node's local storage. This is error-prone and defeats the purpose of the integrated registry (`localhost:32000`).
    *   **Recommendation:** Use `imagePullPolicy: IfNotPresent` or `Always` and configure proper authentication/access to the local registry.

*   **PostgreSQL Probe Configuration (Reliability):**
    *   **Finding:** Liveness and Readiness probes use hardcoded user arguments (`pg_isready -U simpelv2`).
    *   **Impact:** If the `POSTGRES_USER` environment variable changes, the probes will fail even if the database is healthy.
    *   **Recommendation:** Use shell expansion: `/bin/sh -c "pg_isready -U $POSTGRES_USER"`.

### 2.2 Effectiveness (Strengths)

*   **Kustomize Structure:** The use of `base` and `overlays` is correctly implemented, allowing for clear separation between "Staging" and "Production" configurations without code duplication.
*   **Service Mesh:** Istio is integrated for traffic management and mTLS, which is a robust choice for microservices security.
*   **State Management:** StatefulSets are correctly used for PostgreSQL and Secreton, with PersistentVolumeClaims via Longhorn.

### 2.3 Efficiency (Resource Usage)

*   **Resource Management:**
    *   **Finding:** Production overlays implement `ResourceQuota` and `LimitRange`. Backend services have defined `requests` (250m CPU) and `limits` (1000m CPU).
    *   **Assessment:** This is an efficient setup that prevents "noisy neighbor" problems and ensures predictable scheduling.
    *   **HPA:** Horizontal Pod Autoscalers are configured to scale based on CPU/Memory utilization, ensuring resources are used only when needed.

### 2.4 Security (Hardening)

*   **Pod Security Standards:**
    *   **Finding:** Most pods run as non-root users (`runAsNonRoot: true`), which is excellent.
    *   **Gap:** `allowPrivilegeEscalation: false` is missing from most container security contexts. This is a requirement for the "Restricted" Pod Security Standard.
    *   **Gap:** Filesystems are not explicitly read-only (`readOnlyRootFilesystem: true`), though some services mount specific volumes for writable areas.

## 3. Action Plan

The following immediate remediations are being applied:

1.  **Reliability:** Fixing PostgreSQL probes to use environment variables.
2.  **Security:** Adding `allowPrivilegeEscalation: false` to all application containers.

**Future Recommendations (Requires User Decision):**
*   Review the single-node pinning strategy for Production.
*   Migrate to a registry-based deployment workflow (remove `imagePullPolicy: Never`).
