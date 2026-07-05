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
 * Flow asserted:
 *   1. Unauthenticated `/perlengkapan/` → redirect to `/portal/login?redirect_uri=…`.
 *   2. REAL portal login (authenc + captcha via the debug endpoint) as a
 *      dashboard-capable fixture user.
 *   3. Portal redirects back to the perlengkapan dashboard; perlengkapan mounts
 *      from the same-origin JWT and its authed API calls succeed (the backend
 *      validated the SAME token) → the app chrome renders.
 *   4. Logout clears the session and returns to the perlengkapan login page.
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
  test.beforeEach(async ({ page }) => {
    await page.addInitScript(() => {
      localStorage.clear();
      sessionStorage.clear();
    });
  });

  test('unauthenticated perlengkapan redirects to the portal login with a redirect_uri back', async ({
    page,
  }) => {
    await page.goto('/perlengkapan/', { waitUntil: 'domcontentloaded' });
    await expect(page).toHaveURL(/\/portal\/login\?redirect_uri=/, { timeout: 30000 });
    await expect(page).toHaveURL(/redirect_uri=.*perlengkapan/);
  });

  test('real portal login carries the JWT cross-app and mounts the perlengkapan dashboard', async ({
    page,
  }) => {
    // 1. Land on perlengkapan → bounced to the portal login (same origin).
    await page.goto('/perlengkapan/', { waitUntil: 'domcontentloaded' });
    await expect(page).toHaveURL(/\/portal\/login/, { timeout: 30000 });

    // 2. Real login (authenc + captcha-debug).
    await loginViaPortalUI(page);

    // 3. Portal saved auth_token (same origin) and redirected back to the
    //    perlengkapan dashboard, which mounts from that JWT and calls its API
    //    with it (backend validates the SAME token).
    await page.waitForURL(/\/perlengkapan\/.*dashboard/, { timeout: 30000 });
    await expect(page.locator('button[title="Profil"]')).toBeVisible({ timeout: 30000 });
    // The same-origin token is present for perlengkapan's authed API calls.
    const token = await page.evaluate(() => localStorage.getItem('auth_token'));
    expect(token, 'perlengkapan sees the portal-issued JWT same-origin').toBeTruthy();
    expect(token).not.toBe('mock-jwt-token-for-e2e');
  });

  test('logout from perlengkapan clears the session and returns to its login', async ({ page }) => {
    await page.goto('/perlengkapan/', { waitUntil: 'domcontentloaded' });
    await expect(page).toHaveURL(/\/portal\/login/, { timeout: 30000 });
    await loginViaPortalUI(page);
    await page.waitForURL(/\/perlengkapan\/.*dashboard/, { timeout: 30000 });

    await page.locator('button[title="Profil"]').click();
    await page.getByRole('button', { name: /keluar/i }).click();

    await page.waitForURL(/\/perlengkapan\/login/, { timeout: 30000 });
    const token = await page.evaluate(() => localStorage.getItem('auth_token'));
    expect(token, 'logout cleared the JWT').toBeFalsy();
  });
});
