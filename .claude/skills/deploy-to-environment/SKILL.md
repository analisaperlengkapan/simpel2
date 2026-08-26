---
name: deploy-to-environment
description: Release/deploy SIMPel to staging or production via the mandatory staging→promote→production flow (Helm + release.yml/promote.yml + git tags), including rollback. Use when cutting a release, deploying a chart change, promoting an RC, or rolling back. Enforces "build once, promote the same digest" and "never prod-only".
---

# Deploy to environment (SIMPel)

**Read first (source of truth, don't duplicate):** `infra/AGENTS.md` →
"Pemisahan Lingkungan" → "Alur deploy WAJIB", and `infra/helm/RUNBOOK.md` →
"Release Flow (rc → promote)" + "Rollback". This Skill is the procedure +
guardrails; the values/specifics live there.

## Hard rules (non-negotiable)

- **NEVER deploy prod-only.** `simpel.kejaksaan.go.id` is a government system —
  every change goes **staging → promote → production**. No `helm upgrade` to
  `simpelv2-production` that didn't first pass `simpelv2-staging`.
- **All cluster changes via Helm** (`values-<env>.yaml` + `helm upgrade`) — never
  `kubectl edit/patch/apply` on live resources.
- **Image tags = immutable SemVer** `vMAJOR.MINOR.PATCH` (RC: `-rcN`). NEVER
  `latest`/`stag`/`prod`/empty — `values.schema.json` rejects them at `helm lint`.
- **Build once, promote the artifact.** Promotion **re-tags the identical digest**
  (no rebuild); the image that passed staging QA is bit-for-bit what runs in prod.

## Flow

1. **Build & push** — push git tag `vX.Y.Z-rcN` → `release.yml` builds+pushes all
   images (immutable digest) + cosign + SBOM/provenance + Trivy.
2. **Deploy STAGING** — `helm upgrade --install simpel infra/helm/simpel -f
   infra/helm/simpel/values-staging.yaml -f <secrets-carry> -n simpelv2-staging
   --timeout 20m`. The tag lives in `values-staging.yaml`, not in a `--set`.
   `<secrets-carry>` is derived, never hand-written, and never committed:
   `helm get values simpel -n simpelv2-<env> -o json | jq '{secrets: .secrets}'`.
   It carries the 24 `secrets.*` values supplied at bootstrap that the repo
   values file does not hold. Not `--reuse-values` — that also preserves values
   you *deleted* from the repo file, which is the drift mechanism itself.
   `--timeout 20m` because helm's default 5m is shorter than the migration hook
   Job's `activeDeadlineSeconds: 1800` (staging revision 6 failed in that gap).
   See the header of `values-staging.yaml` for what was measured.
3. **Test STAGING** — smoke + Playwright e2e + `cargo test` against staging. Fail →
   fix, bump `-rc(N+1)`, back to (1). Destructive tests stay on staging.
4. **Promote** — `gh workflow run promote.yml` (re-tag `vX.Y.Z-rcN` → `vX.Y.Z`,
   same digest, no rebuild). Verify source↔target digest identik per image.
5. **Deploy PRODUCTION** — `helm upgrade --install simpel infra/helm/simpel -f
   infra/helm/simpel/values-production.yaml -n simpelv2-production` with the final
   tag (`infra/helm/deploy.sh production`). RC retag to final = the "lulus termasuk
   di prod" certification — only after staging AND prod tests pass.
6. **Test PRODUCTION** — smoke + non-destructive e2e. Regression → rollback (below).

Pre-flight: `helm lint infra/helm/simpel -f infra/helm/simpel/values-<env>.yaml`
and render-diff (`infra/helm/deploy.sh <env> template`) before upgrading. Render
BOTH ways — with and without the secrets overlay — and diff the objects: the
answer should be "0 objects differ", and if it is not, the overlay is load-bearing
and you have just learned which objects depend on it.

## Rollback

```bash
helm history simpel -n simpelv2-<env>
helm rollback simpel <REVISION> -n simpelv2-<env> --wait
```

Keep the previous final tag available so you can roll forward to a known-good digest.

## One-time prod prerequisites (don't skip on a fresh environment)

Bootstrap+unseal Secreton, DigiCert `simpelv2-tls-secret` in `istio-system`,
MetalLB IP pool, `secretonAuth.enabled=true`. From F2H (when activating): gRPC mTLS
cert (or `GRPC_ALLOW_INSECURE=true`), Secreton DB role for dynamic creds, Stakater
Reloader for simpelv1 secret rotation. See `infra/AGENTS.md` "Bootstrap".

## Fresh-environment bootstrap (first-ever install ≠ steady-state upgrade)

> Proven @staging 2026-06-17. A first install on an EMPTY namespace exposed bugs an
> upgrade never hits (the DBs/secrets/ns pre-existed). Root causes + detail: memory
> `project-helm-namespace-footgun-safety`. Required reading before F6-A prod bootstrap.

1. **Pre-flight:** `helm uninstall` of this chart **cascade-deletes the whole namespace
   and its data** (chart renders a Helm-managed `namespace.yaml`; now guarded by
   `resource-policy: keep` + PVC retention, but **lifecycle is upgrade-only** — never
   routine-uninstall a live env). Before ANY uninstall, check if the chart templates the
   namespace. Production uninstall is guarded: `SIMPEL_CONFIRM_DESTROY=yes`.
2. **Pre-create what the chart does NOT manage** (needed before image pulls / hooks):
   - `kubectl create ns simpelv2-<env>` + labels `istio-injection=enabled`,
     `pod-security.kubernetes.io/{enforce=privileged,audit=baseline,warn=baseline}`
     (then install with `--set namespace.create=false`), and
   - `ghcr-pull` image-pull Secret — copy from ns `arc-runners`
     (`kubectl get secret ghcr-pull -n arc-runners -o json | jq 'del(.metadata.namespace,
     .metadata.resourceVersion,.metadata.uid,.metadata.creationTimestamp,.metadata.ownerReferences,
     .metadata.managedFields)' | kubectl apply -n simpelv2-<env> -f -`).
3. **Secrets:** install with `--set secrets.bootstrap=true -f values-secrets.yaml`
   (gitignored, OUTSIDE the repo tree). Staging = fresh-generated postgres/simpelv1 +
   **MOCK** integration (`.invalid` URLs, sync off — real gov-API tokens are prod-only,
   see `layanan/integrasi/AGENTS.md`). Use a **URL-safe** pg password (`openssl rand -hex 24`).
4. **Install (no `--wait`):** `helm upgrade --install simpel infra/helm/simpel -f
   values-<env>.yaml -f values-secrets.yaml -n simpelv2-<env> --timeout 20m`.
   **No `--set`** — `global.imageTag` and `namespace.create` are recorded in
   `values-<env>.yaml`; passing them here is how the repo and the cluster came
   to describe different systems twice (rc24's tag, and `namespace.create`
   since revision 8). Bump the values file in the same commit you tag.
   `--timeout 20m` is required, not decorative: helm's default is 5m while the
   migration hook Job gets `activeDeadlineSeconds: 1800`, and staging revision 6
   failed in exactly that gap ("timed out waiting for the condition") while the
   Job was still running legitimately. Skip `--wait`:
   secreton-0 comes up **sealed** (no auto-unseal); with `secretonAuth.enabled=false`
   nothing depends on it so apps still boot from k8s secrets. `deploy.sh` forces
   `--wait 5m` → would time out; call `helm` directly here.
   - **post-install hooks run in order:** `db-create` (creates `dbsimpelv1`+`secreton` —
     the Patroni image SKIPS `/docker-entrypoint-initdb.d`, so only `dbsimpelv2` exists
     otherwise) → `integrasi-migrate` → `authenc-migrate` (each waits-for-postgres).
     perlengkapan self-migrates on boot AFTER authenc+integrasi schemas exist (may
     crashloop briefly — tolerated by its startupProbe).
5. **Secreton:** bootstrap + unseal fresh (skill `secreton-ops`; **new** Shamir keys —
   any prior offline keys are void once the PVC is gone), then `--set
   secretonAuth.enabled=true` and `helm upgrade`.
6. **Verify:** `dbsimpelv2` has `authenc`+`integrasi`+`perlengkapan` schemas; all pods
   Ready; login HTTP 200. Seed `integrasi.*` synthetic fixtures for staging.

## Gotchas

- `release.yml` / `promote.yml` run on ARC ephemeral runners; heavy image+docker
  builds — don't trigger many releases in parallel (saturates runners).
- The `imagePullPolicy` is `IfNotPresent` in both envs (safe with immutable tags).
- If `helm lint` rejects the tag, you used a mutable/empty tag — fix to SemVer.
