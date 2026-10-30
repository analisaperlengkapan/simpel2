import { test, expect, type Page } from '@playwright/test';
import {
  loginViaSameOriginUi,
  skipUnlessCaptchaDebug,
  trackCaptchaChallenges,
} from './helpers/real-auth';

/**
 * SIMPEL v1 (legacy Laravel) ↔ Portal SSO — REAL end-to-end.
 *
 * Runs against the single-origin `cross-app-ingress` (mirrors the prod Istio
 * VirtualService: `/portal`, `/perlengkapan/simpel/v1`, `/perlengkapan/simpel/v2`
 * and `/api/*` on ONE origin). v1 and the portal MUST share an origin for this
 * flow to work at all: the portal login writes `localStorage["auth_token"]`, and
 * the handoff back to v1 is a same-origin form POST.
 *
 * The flow is a TWO-HOP delegation, mirroring the v2 handoff
 * (`auth-login-flow.spec.ts`) but with an extra step because v1 has no login page
 * of its own and hands the token back by POST:
 *
 *   1. Unauthenticated `GET /perlengkapan/simpel/v1/dashboard` →
 *      v1's `TokenToOAuthMiddleware` answers 302 to
 *      `{origin}/portal/login?return_to=<v1 url>&callback=<v1 callback>&state=<nonce>`.
 *   2. The portal login page sees those params (`legacy_handoff::parse_handoff`),
 *      parks them, and — if a session already exists — bounces immediately.
 *      Otherwise the user logs in for real (authenc + captcha).
 *   3. On success the portal submits a hidden form POST to v1's
 *      `auth/oauth-callback` carrying the JWT + the `state` nonce. v1 verifies
 *      the token through the gateway, establishes its own Laravel session, and
 *      redirects to `return_to`.
 *   4. The v1 dashboard renders — proving the whole round trip, including the
 *      `/perlengkapan/simpel/v1` prefix surviving every redirect.
 *
 * The prefix is the part that silently broke: v1's `AppServiceProvider` forces
 * the URL root from `APP_URL`, the ingress must forward `$http_host` (not
 * `$host`, which drops the port) and must strip the prefix with a `rewrite`
 * before PHP sees the request. Each of those was wrong in turn and each one
 * produced a different dead end — hence the explicit prefix/port assertions in
 * the tests below rather than a single "it renders" check.
 */

// Dashboard-capable fixture user (seed-multisatker.sql): operator_satker,
// require_password_change=false, shared Argon2id password = the NIP string.
// Also present in dbsimpelv1 via seed-simpelv1-schema.sql, which the OAuth
// callback needs to resolve the token subject to a v1 `Pengguna` row.
const USER = { username: '200000000000000001', password: '199203142014031001' };

const V1_PREFIX = '/perlengkapan/simpel/v1';

test.describe('Cross-app SSO — SIMPEL v1 (real, single-origin ingress)', () => {
  // NOTE: do NOT clear storage with page.addInitScript — that init script re-runs
  // on EVERY navigation, so it would wipe the token mid-handoff. Playwright
  // already isolates each test in a fresh context.

  test('unauthenticated v1 bounces to the portal with a prefixed return_to + callback', async ({
    page,
  }) => {
    await page.goto(`${V1_PREFIX}/dashboard`, { waitUntil: 'domcontentloaded' });

    // v1 has no login page: the middleware sends the browser straight to the
    // portal, carrying where to come back to.
    await page.waitForURL(/\/portal\/login\?/, { timeout: 30000 });
    const url = new URL(page.url());

    // Both targets must carry the public prefix, otherwise the post-login
    // redirect lands on a path the ingress does not route (a 404, far from the
    // cause). The port must survive too: the ingress listens on :8080.
    const returnTo = url.searchParams.get('return_to');
    const callback = url.searchParams.get('callback');
    expect(returnTo, 'return_to present').toBeTruthy();
    expect(callback, 'callback present').toBeTruthy();
    expect(new URL(returnTo!).pathname).toContain(V1_PREFIX);
    expect(new URL(callback!).pathname).toBe(`${V1_PREFIX}/auth/oauth-callback`);
    expect(new URL(returnTo!).port, 'ingress port preserved').toBe(url.port);

    // A single-use CSRF nonce must ride along, or the callback cannot verify it.
    expect(url.searchParams.get('state')).toBeTruthy();
  });

  test('real login hands the JWT back to v1 and renders the v1 dashboard', async ({ page }) => {
    await skipUnlessCaptchaDebug(page.request, '');
    const getCaptchaAnswer = trackCaptchaChallenges(page);
    await page.goto(`${V1_PREFIX}/dashboard`, { waitUntil: 'domcontentloaded' });
    await page.waitForURL(/\/portal\/login\?/, { timeout: 30000 });

    await loginViaSameOriginUi(page, USER, getCaptchaAnswer);

    // The portal POSTs the token to v1, which redirects to `return_to`. Landing
    // on the prefixed dashboard proves the callback URL, the `state` check and
    // the prefix round trip all held.
    await page.waitForURL(new RegExp(`${V1_PREFIX}/dashboard`), { timeout: 30000 });

    // A real v1 page, not v1's own login or an error page.
    await expect(page).not.toHaveURL(/\/auth\/login/);
    const title = await page.title();
    expect(title).not.toMatch(/500|Whoops|Server Error/i);
    // The sidebar is rendered from `ms_menu`; its presence means the layout
    // (and therefore the DB-backed menu query) worked, not just a redirect.
    await expect(page.locator('#navbar-nav')).toBeVisible({ timeout: 15000 });
    await expect(page.locator('a[href$="/dashboard"]').first()).toBeVisible();
  });

  test('launching SIMPEL v1 from the portal apps page lands on the v1 dashboard', async ({
    page,
  }) => {
    await skipUnlessCaptchaDebug(page.request, '');
    // This is the operator's actual path in: log in on the portal, open the
    // apps catalogue, click the SIMPEL v1 card. It is a DIFFERENT entry point
    // from the ones above — the card href is the bare prefix
    // `/perlengkapan/simpel/v1` (no trailing slash), so it goes through the
    // ingress's `location = /perlengkapan/simpel/v1` → 302 → `/…/v1/` hop
    // before v1's middleware ever runs. A prefix that only works WITH the
    // slash would strand the user on a 404 right after the click.
    const getCaptchaAnswer = trackCaptchaChallenges(page);
    await page.goto('/portal/login', { waitUntil: 'domcontentloaded' });
    await loginViaSameOriginUi(page, USER, getCaptchaAnswer);
    await page.waitForURL(/\/portal\/(dashboard|apps)/, { timeout: 30000 });

    await page.goto('/portal/apps', { waitUntil: 'domcontentloaded' });
    const v1Card = page.locator('a[href="/perlengkapan/simpel/v1"]');
    await expect(v1Card).toBeVisible({ timeout: 15000 });

    // The card opens in a new tab (v1 has its own session); follow it there.
    const [v1Page] = await Promise.all([page.waitForEvent('popup'), v1Card.click()]);
    await v1Page.waitForLoadState('domcontentloaded');

    // Already authenticated on the portal, so the handoff is immediate: no
    // login form, straight to the v1 dashboard.
    await v1Page.waitForURL(new RegExp(`${V1_PREFIX}/dashboard`), { timeout: 30000 });
    await expect(v1Page.locator('#navbar-nav')).toBeVisible({ timeout: 15000 });
    await expect(v1Page.getByText('Masuk ke Sistem')).toHaveCount(0);
  });

  test('v1 session persists across navigation (no re-bounce to the portal)', async ({ page }) => {
    await skipUnlessCaptchaDebug(page.request, '');
    const getCaptchaAnswer = trackCaptchaChallenges(page);
    await page.goto(`${V1_PREFIX}/dashboard`, { waitUntil: 'domcontentloaded' });
    await page.waitForURL(/\/portal\/login\?/, { timeout: 30000 });
    await loginViaSameOriginUi(page, USER, getCaptchaAnswer);
    await page.waitForURL(new RegExp(`${V1_PREFIX}/dashboard`), { timeout: 30000 });

    // Reloading must NOT start a new OAuth dance: v1 holds its own Laravel
    // session cookie, so the middleware finds the token and lets it through.
    await page.goto(`${V1_PREFIX}/dashboard`, { waitUntil: 'domcontentloaded' });
    await expect(page).not.toHaveURL(/\/portal\/login/);
    await expect(page.locator('#navbar-nav')).toBeVisible({ timeout: 15000 });
  });

  test('an existing portal session is reused instead of prompting again', async ({ page }) => {
    await skipUnlessCaptchaDebug(page.request, '');
    // Log in once directly on the portal, so the same-origin token exists.
    // No "is the form showing?" probe: we arrive with no token, so the form is
    // guaranteed, and probing for it just races the WASM mount — a probe that
    // answers "no form" silently skips the login and then fails 30s later on a
    // URL that never changes.
    const getCaptchaAnswer = trackCaptchaChallenges(page);
    await page.goto('/portal/login', { waitUntil: 'domcontentloaded' });
    await loginViaSameOriginUi(page, USER, getCaptchaAnswer);
    await page.waitForURL(/\/portal\/(dashboard|apps)/, { timeout: 30000 });

    // Now reach v1. It bounces to the portal, which must recognise the live
    // session and hand the token back without showing the login form again.
    await page.goto(`${V1_PREFIX}/dashboard`, { waitUntil: 'domcontentloaded' });
    await page.waitForURL(new RegExp(`${V1_PREFIX}/dashboard`), { timeout: 30000 });
    await expect(page.locator('#navbar-nav')).toBeVisible({ timeout: 15000 });
    // The login form must never have been shown on this leg.
    await expect(page.getByText('Masuk ke Sistem')).toHaveCount(0);
  });
});
