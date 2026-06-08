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
   infra/helm/simpel/values-staging.yaml -n simpelv2-staging` (or
   `infra/helm/deploy.sh staging`). Pin the `-rcN` tag.
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
and render-diff (`infra/helm/deploy.sh <env> template`) before upgrading.

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

## Gotchas

- `release.yml` / `promote.yml` run on ARC ephemeral runners; heavy image+docker
  builds — don't trigger many releases in parallel (saturates runners).
- The `imagePullPolicy` is `IfNotPresent` in both envs (safe with immutable tags).
- If `helm lint` rejects the tag, you used a mutable/empty tag — fix to SemVer.
