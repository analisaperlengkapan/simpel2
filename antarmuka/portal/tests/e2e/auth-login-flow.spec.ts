import { test, expect, type Page } from '@playwright/test';

/**
 * Cross-app SSO — REAL end-to-end (F-GW PR-E), no mock token.
 *
 * Runs against the single-origin `cross-app-ingress` (mirrors the prod Istio
 * VirtualService: `/portal`, `/perlengkapan`, `/api/*` on ONE origin). That
 * single origin is what makes SSO work: the portal login writes the JWT to
 * `localStorage["auth_token"]` and redirects to `/perlengkapan/…`, which reads
 * the SAME-ORIGIN token and derives its whole session by decoding that JWT
 * (perlengkapan intentionally no longer reads any legacy session object — see
 * antarmuka/perlengkapan/src/features/auth.rs). The compose two-origin stack
 * can't exercise this (localStorage isn't shared), so this suite is its own
 * project (`portal-cross-app`) with the ingress as baseURL.
 *
 * The handoff is a TWO-HOP delegation (this is the real FE design, not a direct
 * bounce): perlengkapan does NOT redirect unauthenticated users straight to the
 * portal. It renders its OWN login page (`/perlengkapan/simpel/v2/login`, see
 * antarmuka/perlengkapan/src/pages/login.rs) that carries a single "Masuk via
 * Portal" link → `/portal/login?redirect_uri=%2Fperlengkapan%2Fsimpel%2Fv2%2Fdashboard`.
 * The portal login honors that `redirect_uri` (only when it starts with
 * `/perlengkapan`, see resolve_perlengkapan_redirect_target in
 * antarmuka/portal/src/features/auth/pages/login.rs) and hard-redirects back
 * after a successful login. Logout in perlengkapan clears the token and sends the
 * user to `{origin}/portal/login` (profile_menu.rs), NOT its own login page.
 *
 * Flow asserted:
 *   1. Unauthenticated `/perlengkapan/` → perlengkapan's own login page, which
 *      offers a "Masuk via Portal" link with a `redirect_uri` back to perlengkapan.
 *   2. Follow that link → portal login → REAL login (authenc + captcha via the
 *      debug endpoint) as a dashboard-capable fixture user.
 *   3. Portal honors the redirect_uri → lands back on the perlengkapan dashboard;
 *      perlengkapan mounts from the same-origin JWT and its authed API calls
 *      succeed (the backend validated the SAME token) → the app chrome renders.
 *   4. Logout clears the session and returns to the portal login page.
 */

// Dashboard-capable fixture user (seed-multisatker.sql): validator_pusat,
// require_password_change=false, shared Argon2id password = the NIP string.
const USER = { username: '200000000000000004', password: '199203142014031001' };

/** Perform a real portal-UI login on the current (login) page. Captures the
 *  captcha challenge the WASM issues, resolves it via the debug endpoint, then
 *  submits credentials. Assumes the page is already on `/portal/login`. */
async function loginViaPortalUI(page: Page): Promise<void> {
  // Collect captcha challenge ids as the WASM requests them (same-origin — the
  // ingress proxies /api/captcha to authenc, so no page.route proxy is needed).
  const challengeIds: string[] = [];
  page.on('response', async (resp) => {
    if (resp.url().includes('/api/captcha/challenge') && resp.request().method() === 'POST') {
      try {
        const body = await resp.json();
        if (body?.challenge_id) challengeIds.push(body.challenge_id);
      } catch {
        /* ignore non-JSON */
      }
    }
  });

  await expect(page.getByText('Masuk ke Sistem')).toBeVisible({ timeout: 30000 });
  const passwordToggle = page.getByText('Atau masuk dengan password');
  if (await passwordToggle.isVisible({ timeout: 3000 }).catch(() => false)) {
    await passwordToggle.click();
  }

  const captchaInput = page.locator('input[placeholder="Masukkan teks..."]');
  await expect(captchaInput).toBeVisible({ timeout: 15000 });

  // Wait for the WASM's captcha challenge to have landed, then fetch its answer.
  await expect.poll(() => challengeIds.length, { timeout: 10000 }).toBeGreaterThan(0);
  const challengeId = challengeIds[challengeIds.length - 1];
  const dbg = await page.request.get(`/api/captcha/debug/${challengeId}`);
  expect(dbg.ok()).toBeTruthy();
  const answer = (await dbg.json()).answer as string;
  expect(answer).toBeTruthy();

  await page.locator('#username').fill(USER.username);
  await page.locator('#password').fill(USER.password);
  await captchaInput.fill(answer);

  const [verify] = await Promise.all([
    page.waitForResponse(
      (r) => r.url().includes('/api/captcha/verify') && r.request().method() === 'POST',
      { timeout: 15000 },
    ),
    page.locator('.captcha-container button', { hasText: 'Kirim' }).click(),
  ]);
  expect(verify.status()).toBe(200);
  await page.waitForTimeout(500); // let the captcha_token signal propagate

  const loginBtn = page.getByRole('button', { name: /Masuk ke Portal/i });
  await expect(loginBtn).toBeEnabled({ timeout: 10000 });
  const [loginResp] = await Promise.all([
    page.waitForResponse(
      (r) => r.url().includes('/login') && r.request().method() === 'POST',
      { timeout: 15000 },
    ),
    loginBtn.click(),
  ]);
  expect(loginResp.status()).toBe(200);
}

test.describe('Cross-app SSO (real, single-origin ingress)', () => {
  // NOTE: do NOT clear storage with page.addInitScript — that init script re-runs
  // on EVERY navigation, so it would wipe the portal-issued auth_token on the hard
  // redirect into the perlengkapan dashboard (window.location.set_href), booting
  // the session the instant it lands. Playwright already isolates each test in a
  // fresh context, so localStorage starts empty; the SSO handoff must be allowed
  // to persist the token across the portal→perlengkapan navigation.

  /** Land on perlengkapan unauthenticated → its own login page → follow the
   *  "Masuk via Portal" link to the portal login. Leaves the page on the portal
   *  login with the `redirect_uri` back to the perlengkapan dashboard. */
  async function reachPortalLoginViaPerlengkapan(page: Page): Promise<void> {
    await page.goto('/perlengkapan/', { waitUntil: 'domcontentloaded' });
    // Perlengkapan bounces the unauthenticated user to ITS OWN login page
    // (it delegates auth to the portal rather than redirecting there directly).
    await expect(page).toHaveURL(/\/perlengkapan\/simpel\/v2\/login/, { timeout: 30000 });
    // Follow the single SSO entry point (a hard <a href> navigation).
    await page.getByRole('link', { name: /Masuk via Portal/i }).click();
    await expect(page).toHaveURL(/\/portal\/login\?redirect_uri=/, { timeout: 30000 });
  }

  test('unauthenticated perlengkapan shows its login with a portal SSO link carrying a redirect_uri back', async ({
    page,
  }) => {
    await page.goto('/perlengkapan/', { waitUntil: 'domcontentloaded' });
    // Its own login page, not a direct bounce to the portal.
    await expect(page).toHaveURL(/\/perlengkapan\/simpel\/v2\/login/, { timeout: 30000 });

    // The "Masuk via Portal" link is the SSO entry point; its href hands the
    // portal a redirect_uri that points back to the perlengkapan dashboard.
    const portalLink = page.getByRole('link', { name: /Masuk via Portal/i });
    await expect(portalLink).toBeVisible({ timeout: 15000 });
    const href = await portalLink.getAttribute('href');
    expect(href, 'portal SSO link present').toBeTruthy();
    expect(href).toMatch(/\/portal\/login\?redirect_uri=/);
    // redirect_uri is URL-encoded but must resolve back into perlengkapan.
    expect(decodeURIComponent(href!)).toMatch(/redirect_uri=\/perlengkapan/);
  });

  test('real portal login carries the JWT cross-app and mounts the perlengkapan dashboard', async ({
    page,
  }) => {
    // 1. perlengkapan → its own login → follow the SSO link to the portal login.
    await reachPortalLoginViaPerlengkapan(page);

    // 2. Real login (authenc + captcha-debug).
    await loginViaPortalUI(page);

    // 3. Portal honored the redirect_uri: it saved auth_token (same origin) and
    //    redirected back to the perlengkapan dashboard, which mounts from that
    //    JWT and calls its API with it (backend validates the SAME token).
    await page.waitForURL(/\/perlengkapan\/.*dashboard/, { timeout: 30000 });
    await expect(page.locator('button[title="Profil"]')).toBeVisible({ timeout: 30000 });
    // The same-origin token is present for perlengkapan's authed API calls.
    const token = await page.evaluate(() => localStorage.getItem('auth_token'));
    expect(token, 'perlengkapan sees the portal-issued JWT same-origin').toBeTruthy();
    expect(token).not.toBe('mock-jwt-token-for-e2e');
  });

  test('logout from perlengkapan clears the session and returns to the portal login', async ({
    page,
  }) => {
    await reachPortalLoginViaPerlengkapan(page);
    await loginViaPortalUI(page);
    await page.waitForURL(/\/perlengkapan\/.*dashboard/, { timeout: 30000 });

    await page.locator('button[title="Profil"]').click();
    await page.getByRole('button', { name: /keluar/i }).click();

    // Perlengkapan logout hard-redirects to the portal login (not its own).
    await page.waitForURL(/\/portal\/login/, { timeout: 30000 });
    const token = await page.evaluate(() => localStorage.getItem('auth_token'));
    expect(token, 'logout cleared the JWT').toBeFalsy();
  });
});
