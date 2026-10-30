# SIMPEL v1/v2 E2E Test Suite

## Overview

End-to-end coverage of the boundary between the Portal (Leptos/WASM), SIMPEL v2
(`perlengkapan` microfrontend + Rust backend) and SIMPEL v1 (the legacy Laravel
monolith). Tests are Playwright specs, split across the two frontends' own
`tests/e2e/` trees — there is no third top-level suite.

The v1 leg runs against the **single-origin** `cross-app-ingress`
(`infra/nginx/e2e-cross-app-ingress.conf`), which mirrors the production Istio
VirtualService: `/portal`, `/perlengkapan/simpel/v1`,
`/perlengkapan/simpel/v2` and `/api/*` all on one origin. One origin is not a
test convenience — SSO depends on it. The Portal writes
`localStorage["auth_token"]` and v1 completes its handoff with a same-origin form
POST, so a second origin silently detaches both.

## Where the tests live

| Concern | Spec | Project | Job |
|---|---|---|---|
| Portal login (captcha, token) | `antarmuka/portal/tests/e2e/portal-auth-e2e.spec.ts` | `portal-chromium` | `e2e-portal` |
| Portal core workflows, apps catalogue | `antarmuka/portal/tests/e2e/portal-core-workflow.spec.ts` | `portal-core` | `e2e-portal` |
| Portal → v2 SSO | `antarmuka/portal/tests/e2e/auth-login-flow.spec.ts` | `portal-cross-app` | `e2e-portal-cross-app` |
| Portal ↔ **v1** SSO | `antarmuka/portal/tests/e2e/v1-sso-flow.spec.ts` | `portal-cross-app` | `e2e-portal-cross-app` |
| Integrasi ↔ Authenc (gRPC) | `antarmuka/portal/tests/e2e/integrasi-authenc-e2e.spec.ts` | `portal-integration` | `e2e-integrasi-authenc` |
| v1 permissions / RBAC | `antarmuka/perlengkapan/tests/e2e/rbac-scoping*.spec.ts` | `perlengkapan-rbac*` | `e2e-perlengkapan` |

## The v1 handoff, step by step

v1 has **no login page of its own**. When an unauthenticated browser reaches a
protected v1 route, `TokenToOAuthMiddleware` bounces it to the Portal carrying
where to come back to:

```
{origin}/portal/login?return_to=<v1 url>&callback=<v1 oauth-callback>&state=<nonce>
```

1. The Portal login page parses those parameters
   (`features/auth/legacy_handoff.rs::parse_handoff`) and **parks** them in
   `sessionStorage`, so an MFA or forced-password-change detour — which navigates
   away and drops the query string — can still complete the handoff.
2. If a live Portal session already exists, the handoff completes immediately
   and the login form is never shown.
3. Otherwise the user authenticates for real (Authenc + captcha).
4. On success the Portal submits a hidden **form POST** to v1's
   `auth/oauth-callback` with the JWT and the `state` nonce. A POST, not a
   navigation with `?token=…`, so the token does not land in browser history,
   access logs or `Referer` headers.
5. v1 verifies the token through the gateway, establishes its own Laravel
   session, and redirects to `return_to`.

`v1-sso-flow.spec.ts` covers both entry points — a direct deep link and a click
on the **SIMPEL v1** card in the Portal apps catalogue — plus session reuse and
persistence.

## The apps catalogue

The Portal apps page lists SIMPEL v1 and SIMPEL v2 as **two first-class cards**,
not one card with a version dropdown: the user is choosing which application to
open, not a setting inside one. The registry is
`antarmuka/portal/src/features/microfrontends.rs`; the page is
`antarmuka/portal/src/features/apps/mod.rs`. Both cards open in a new tab
(`target="_blank"`), since each has its own session.

## Running the tests

### Prerequisites

The stack must be up on one origin. In CI this is the `e2e-portal-cross-app` job;
locally:

```bash
docker compose -f docker-compose.yml -f docker-compose.e2e.yml --profile cross-app up -d \
  postgres redis authenc layanan-integrasi layanan-perlengkapan perlengkapan portal \
  simpelv1 gateway cross-app-ingress
```

Then seed the fixtures — **both** databases, in order:

```bash
docker compose -f docker-compose.yml -f docker-compose.e2e.yml run -T --rm e2e-seed \
  -v ON_ERROR_STOP=1 -h postgres -U postgres -d dbsimpelv2 -f - \
  < tests/fixtures/e2e/seed-multisatker.sql

POSTGRES_PASSWORD=e2e_test_pw docker compose -f docker-compose.yml -f docker-compose.e2e.yml \
  run -T --rm e2e-seed -v ON_ERROR_STOP=1 -v "pw=e2e_test_pw" \
  -h postgres -U postgres -d dbsimpelv1 -f - \
  < tests/fixtures/e2e/seed-simpelv1-schema.sql
```

`dbsimpelv1` is created empty; without its schema and fixture user the OAuth
callback dead-ends at "User tidak ditemukan di sistem".

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

| Job | Blocking? | Stack |
|---|---|---|
| `e2e-portal` | **yes** (`ci-summary.needs`) | portal + authenc |
| `e2e-perlengkapan` | **yes** | both FEs + backends |
| `e2e-portal-cross-app` | no — opt-in via `E2E_INTEGRATION_ENABLED` / `run_e2e` | single origin incl. v1 |
| `e2e-integrasi-authenc` | no — opt-in | authenc + secreton + integrasi |
| `e2e-simpelv1-integration` | no — opt-in | PHPUnit `IntegrationLive` vs the real gateway |

## Troubleshooting

### v1 bounces but the post-login redirect 404s

Check the prefix survived. v1 forces its URL root from `APP_URL`
(`AppServiceProvider::boot`), the ingress must forward `$http_host` (**not**
`$host`, which drops the port) and must strip `/perlengkapan/simpel/v1` with a
`rewrite` before PHP sees the request. `v1-sso-flow.spec.ts` asserts the prefix
and the port explicitly for exactly this reason.

### "User tidak ditemukan di sistem" after a valid token

The OAuth callback resolves the v1 row by **`nip`** (`$claims['nip'] ??
$claims['username']`), not by `sub` — `sub` is the Authenc user UUID and matches
no v1 row. Make sure `seed-simpelv1-schema.sql` ran and the fixture NIP exists in
`dbsimpelv1`.

### Seed fails with `other objects depend on it`

`seed-simpelv1-schema.sql` drops and recreates the foreign table, so dependent
views must be dropped first. The file already handles this; if you add a view,
add it to the drop list too.

### Captcha answer never arrives

Attach `trackCaptchaChallenges(page)` before the navigation that loads the login
page — see "Captcha" above.
