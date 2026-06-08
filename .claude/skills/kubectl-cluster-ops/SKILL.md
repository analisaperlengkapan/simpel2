---
name: kubectl-cluster-ops
description: Diagnose and troubleshoot the SIMPel Kubernetes cluster (MicroK8s on node simple02) as an infra expert — pods CrashLoop/Pending, Istio gateway/mTLS, Calico networking, ARC runners, MetalLB, Longhorn PVCs, ImagePull, certs, DNS. Read/diagnose + emergency ops only. Use for cluster incidents and inspection. NOT for routine changes — those go through Helm (see deploy-to-environment).
---

# kubectl cluster ops (SIMPel)

Expert read/diagnose + emergency ops for **this specific cluster**. Source of
truth: `infra/AGENTS.md` + `infra/helm/RUNBOOK.md` (troubleshooting) + memory
`project-ci-arc-migration`. Don't duplicate values — this is the playbook.

## ⛔ Guardrail (mandatory)
**Mutations go through Helm** (`infra/helm/`, `values-<env>.yaml` + `helm upgrade`)
— **never** `kubectl apply/edit/patch` on live resources. This Skill is for
**read/inspect/diagnose** and **emergency-only** ops (cordon, delete a wedged pod,
rollout restart). Any persistent change → edit the chart and `helm upgrade`.

## Cluster facts
- MicroK8s, single worker node **`simple02`** (API `172.15.10.254:16443`),
  kubeconfig context **`admin`** (host `simple02`, user runs sudo there).
- Namespaces: **`simpelv2-staging`**, **`simpelv2-production`**, **`istio-system`**,
  **`arc-systems`** (ARC controller) + **`arc-runners`** (ephemeral runners),
  `metallb-system`, `longhorn-system`.
- Networking: **Istio** gateway/VirtualService for external routing (chart-templated);
  **Calico** CNI (**MTU 1450** — mismatch ⇒ stalled TLS/large-payload hangs);
  **MetalLB** L2 IP pool; **Longhorn** PVCs. Internal mTLS via Istio
  PeerAuthentication (prod = STRICT, staging = PERMISSIVE).

## Quick inspect
```bash
kubectl get pods -n simpelv2-<env> -o wide
kubectl describe pod <pod> -n <ns>            # events: scheduling, pulls, probes
kubectl logs <pod> -n <ns> --previous          # last crash
kubectl get events -n <ns> --sort-by=.lastTimestamp | tail -30
kubectl -n istio-system get gateway,virtualservice -A
```

## Troubleshooting playbooks
- **CrashLoopBackOff** → `logs --previous`; check startupProbe window (perlengkapan/
  simpelv1 run migrations/secret-fetch at boot — liveness can kill them; F6-X adds
  startupProbe). Check Secreton reachable + creds.
- **Pending** → `describe pod` events: insufficient resources (prod ResourceQuota/
  LimitRange), node-pin (staging pods pinned to `simple02`), or PVC unbound
  (Longhorn). `kubectl get pvc -n <ns>`.
- **ImagePullBackOff** → image tag must be immutable SemVer in ghcr; pull secret
  `ghcr-pull` (docker-registry) present in the namespace? PAT scope `read:packages`.
- **502/no route / mTLS** → Istio gateway + VirtualService bound? cert
  `simpel-tls`/`simpelv2-tls-secret` in `istio-system`? prod STRICT mTLS means a
  non-sidecar/plaintext caller is rejected — check PeerAuthentication + sidecar
  injection (`kubectl get pod <p> -o jsonpath … istio-proxy`).
- **DNS / intermittent timeouts / large-payload hang** → suspect Calico **MTU
  1450** mismatch; CoreDNS pods healthy?
- **ARC runners: jobs queued, none running** → `kubectl get pods -n arc-runners`
  (scaling up? dind ready?); controller `kubectl logs -n arc-systems <listener>`;
  `maxRunners` cap (16) hit by parallel runs. Registry mirror `mirror.gcr.io` in dind.
- **PVC / data** → `kubectl -n longhorn-system get volumes`; never delete a bound
  PVC without confirming it's not production data.

## Emergency ops (allowed; still prefer Helm)
```bash
kubectl rollout restart deploy/<name> -n <ns>      # pick up rotated secret / unwedge
kubectl delete pod <wedged-pod> -n <ns>            # let the controller recreate
kubectl cordon simple02 / uncordon simple02        # incident only
helm history simpel -n simpelv2-<env>; helm rollback simpel <REV> -n <ns> --wait
```
After any emergency mutation, reconcile the chart so state isn't drifted.
