# SIMPEL E2E Test Suite

## Overview

End-to-end coverage of the boundary between the Portal (Leptos/WASM) and SIMPEL
(the `perlengkapan` microfrontend + Rust backend). Tests are Playwright specs,
split across the two frontends' own `tests/e2e/` trees — there is no third
top-level suite.

The cross-app leg runs against the **single-origin** `cross-app-ingress`
(`infra/nginx/e2e-cross-app-ingress.conf`), which mirrors the production Istio
VirtualService: `/portal`, `/perlengkapan/simpel` and `/api/*` all on one
origin. One origin is not a test convenience — SSO depends on it. The Portal
writes `localStorage["auth_token"]` and SIMPEL reads the same-origin token, so a
second origin silently detaches them.

SIMPEL v1 (the legacy Laravel monolith) was removed from the repository on
2026-11-16, together with its SSO handoff and its e2e jobs.

## Where the tests live

| Concern | Spec | Project | Job |
|---|---|---|---|
| Portal login (captcha, token) | `antarmuka/portal/tests/e2e/portal-auth-e2e.spec.ts` | `portal-chromium` | `e2e-portal` |
| Portal core workflows, apps catalogue | `antarmuka/portal/tests/e2e/portal-core-workflow.spec.ts` | `portal-core` | `e2e-portal` |
| Portal → SIMPEL SSO | `antarmuka/portal/tests/e2e/auth-login-flow.spec.ts` | `portal-cross-app` | `e2e-portal-cross-app` |
| Integrasi ↔ Authenc (gRPC) | `antarmuka/portal/tests/e2e/integrasi-authenc-e2e.spec.ts` | `portal-integration` | `e2e-integrasi-authenc` |
| Permissions / RBAC | `antarmuka/perlengkapan/tests/e2e/rbac-scoping*.spec.ts` | `perlengkapan-rbac*` | `e2e-perlengkapan` |

## The apps catalogue

The Portal apps page lists **SIMPEL** as one card, named without a version. The
registry is `antarmuka/portal/src/features/microfrontends.rs`; the page is
`antarmuka/portal/src/features/apps/mod.rs`. The card opens in a new tab
(`target="_blank"`), so launching the app does not tear down the portal.

## Running the tests

### Prerequisites

The stack must be up on one origin. In CI this is the `e2e-portal-cross-app` job;
locally:

```bash
docker compose -f docker-compose.yml -f docker-compose.e2e.yml --profile cross-app up -d \
  postgres redis authenc layanan-integrasi layanan-perlengkapan perlengkapan portal \
  cross-app-ingress
```

Then seed the fixtures:

```bash
docker compose -f docker-compose.yml -f docker-compose.e2e.yml run -T --rm e2e-seed \
  -v ON_ERROR_STOP=1 -h postgres -U postgres -d dbsimpelv2 -f - \
  < tests/fixtures/e2e/seed-multisatker.sql
```

### Run

Playwright runs from inside the e2e image (the specs need a browser + the stack
network):

```bash
docker run --rm --network "$(docker network ls --format '{{.Name}}' | grep -m1 simpel-network)" \
  -e BASE_URL=http://cross-app-ingress:8080 \
  simpel-e2e-portal npx playwright test --project=portal-cross-app
```

The `portal-core` project needs `PORTAL_URL` (not `BASE_URL`) because it drives
the portal through Playwright `page.route()` proxies rather than the ingress:

```bash
docker run --rm --network "$NET" \
  -e BASE_URL=http://portal:8080 \
  -e PORTAL_URL=http://portal:8080 \
  -e AUTHENC_URL=http://authenc:8088 \
  simpel-e2e-portal npx playwright test --project=portal-core
```

### Captcha

Authenc only demands a captcha after repeated failures
(`captcha_threshold`), so a fresh correct login needs none. The e2e stack builds
Authenc with `CAPTCHA_DEBUG=1`, and specs call
`skipUnlessCaptchaDebug(page.request)` to skip themselves rather than fail
against a stack that cannot answer a challenge.

The widget fetches its challenge **during page load**, which is earlier than a
listener attached after `goto` can see it — so a spec that needs to solve a
captcha must call `trackCaptchaChallenges(page)` **before** navigating, then pass
the returned getter into `loginViaSameOriginUi(page, user, getCaptchaAnswer)`.

## CI

Which of these jobs run on a pull request is decided by what the diff can affect, not
by file extension: see "CI scoping" in `.github/WORKFLOWS.md`. A change to the portal
runs the suites that use the portal image; a change to `layanan/secreton` runs the two
that use Secreton; `lib/core` runs all of them; a docs-only change runs none. The
nightly run and `workflow_dispatch` always run every suite.

| Job | Blocking? | Stack |
|---|---|---|
| `e2e-portal` | **yes** (`ci-summary.needs`) | portal + authenc |
| `e2e-perlengkapan` | **yes** | both FEs + backends |
| `e2e-portal-cross-app` | **yes** when it runs (opt-in via `E2E_INTEGRATION_ENABLED` / `run_e2e`) | single origin |
| `e2e-integrasi-authenc` | yes when it runs (opt-in) | authenc + secreton + integrasi |
| `e2e-secreton-bootstrap` | yes when it runs (opt-in) | postgres + secreton |

## Troubleshooting

### Captcha answer never arrives

Attach `trackCaptchaChallenges(page)` before the navigation that loads the login
page — see "Captcha" above.
