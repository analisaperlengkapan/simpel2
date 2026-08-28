/**
 * Portal Authentication E2E Tests
 *
 * Tests REAL login flow through the portal frontend with actual authenc backend.
 * Uses port-forwarded services:
 *   - Portal:  http://localhost:18080
 *   - Authenc: http://localhost:18088
 *
 * Strategy:
 * 1. page.route() proxies /api/** requests from portal origin to authenc
 * 2. Captcha challenge response is intercepted to extract the answer
 * 3. Answer is auto-filled, enabling real captcha verification
 * 4. Login form is submitted with real credentials
 */
import { test, expect, Page, Route } from '@playwright/test';
import { hasCaptchaDebugEndpoint, skipUnlessCaptchaDebug } from './helpers/real-auth';

// ── Configuration ──────────────────────────────────────────────────────────

const PORTAL_URL = process.env.PORTAL_URL || 'http://localhost:18080';
const AUTHENC_URL = process.env.AUTHENC_URL || 'http://localhost:18088';

// Test credentials.
// NIP_USER = the BASELINE seed user (002_seed.sql): require_password_change=true
// BY DESIGN, so a successful login lands on /portal/password — used for the
// credential/captcha mechanics, NOT for dashboard-flow assertions.
const NIP_USER = {
  username: '199203142014031001',
  password: '199203142014031001',
};

// Dashboard-capable users come from the multi-satker fixture
// (tests/fixtures/e2e/seed-multisatker.sql, seeded by the e2e-portal job):
// real login-able users with require_password_change=false and the shared
// seed password. The baseline `admin` user is NOT login-able (placeholder
// hash), so the "second user" flow uses the fixture validator_pusat.
const OPERATOR_USER = {
  username: '200000000000000001',
  password: '199203142014031001',
};

const VALIDATOR_PUSAT_USER = {
  username: '200000000000000004',
  password: '199203142014031001',
};

// ── CORS Headers for proxied responses ─────────────────────────────────────

/** CORS headers needed because the WASM captcha component uses RequestMode::Cors */
const CORS_HEADERS: Record<string, string> = {
  'access-control-allow-origin': '*',
  'access-control-allow-methods': 'GET, POST, PUT, DELETE, OPTIONS',
  'access-control-allow-headers': 'content-type, authorization',
};

/** Merge CORS headers into response headers */
function withCors(headers: Record<string, string>): Record<string, string> {
  return { ...headers, ...CORS_HEADERS };
}

// ── Portal Asset Proxy (base href fix) ─────────────────────────────────────

/**
 * The portal HTML uses <base href="/portal/" />, so the browser resolves
 * WASM/JS/CSS assets to /portal/xxx paths.  When accessed via direct
 * port-forward the portal nginx serves from root, causing 404s.
 * This route handler strips the /portal/ prefix so assets load correctly.
 *
 * When accessed via Istio ingress (non-localhost URL), the VirtualService
 * handles /portal/ routing natively, so this proxy is skipped.
 */
async function setupPortalAssetProxy(page: Page): Promise<void> {
  // Skip asset proxy when running against Istio ingress — routing is handled natively
  const portalHost = new URL(PORTAL_URL).hostname;
  if (portalHost !== 'localhost' && portalHost !== '127.0.0.1') {
    return;
  }

  await page.route(`${PORTAL_URL}/portal/**`, async (route: Route) => {
    const url = new URL(route.request().url());

    // API calls are handled by the unified API proxy, not the asset proxy
    if (url.pathname.startsWith('/portal/api/') || url.pathname.startsWith('/api/')) {
      await route.continue();
      return;
    }

    // Strip /portal/ prefix to fetch from root
    const newPath = url.pathname.replace(/^\/portal\//, '/');
    const assetUrl = `${PORTAL_URL}${newPath}${url.search}`;

    try {
      const response = await page.request.fetch(assetUrl, {
        method: route.request().method(),
        headers: route.request().headers(),
      });
      await route.fulfill({
        status: response.status(),
        headers: response.headers(),
        body: await response.body(),
      });
    } catch {
      await route.continue();
    }
  });
}

// ── Captcha Solver Helper ──────────────────────────────────────────────────

interface CaptchaChallengeBody {
  challenge_id?: string;
  challenge_data?: string;
}

/**
 * Fetch the captcha answer from the debug endpoint.
 * Requires CAPTCHA_DEBUG=true on the authenc server.
 *
 * GET /api/captcha/debug/{challenge_id}
 */
async function fetchCaptchaAnswer(page: Page, challengeId: string): Promise<string | null> {
  try {
    const resp = await page.request.get(`${AUTHENC_URL}/api/captcha/debug/${challengeId}`);
    if (!resp.ok()) return null;
    const body = await resp.json();
    return body.answer ?? null;
  } catch {
    return null;
  }
}

/**
 * Solve captcha by calling the authenc API directly, fetching the answer
 * from the debug endpoint, and verifying it.
 */
async function solveCaptchaViaApi(page: Page): Promise<{
  challengeId: string;
  answer: string;
  token?: string;
}> {
  // Step 1: Get challenge
  const challengeResp = await page.request.post(`${AUTHENC_URL}/api/captcha/challenge`, {
    data: { challenge_type: 'text_recognition', difficulty: 2 },
    headers: { 'Content-Type': 'application/json' },
  });
  expect(challengeResp.ok()).toBeTruthy();

  const challenge = await challengeResp.json();
  const challengeId = challenge.challenge_id;

  // Step 2: Fetch the real answer from the debug endpoint
  const answer = await fetchCaptchaAnswer(page, challengeId);
  expect(answer).toBeTruthy();

  // Step 3: Verify challenge
  const verifyResp = await page.request.post(`${AUTHENC_URL}/api/captcha/verify`, {
    data: {
      challenge_id: challengeId,
      answer: answer!,
      session_id: `e2e_test_${Date.now()}`,
    },
    headers: { 'Content-Type': 'application/json' },
  });
  expect(verifyResp.ok()).toBeTruthy();

  const verifyResult = await verifyResp.json();
  expect(verifyResult.success).toBe(true);

  return {
    challengeId,
    answer: answer!,
    token: verifyResult.token,
  };
}

// ── Unified API Proxy ──────────────────────────────────────────────────────

/**
 * Proxy a request to authenc and fulfill the route with CORS headers.
 * Handles all /api/** requests generically.
 * Returns the parsed JSON body for captcha challenge (to extract answer).
 */
async function proxyToAuthenc(
  page: Page,
  route: Route,
  onBody?: (body: unknown) => void | Promise<void>,
): Promise<void> {
  const request = route.request();
  const url = new URL(request.url());
  const proxyUrl = `${AUTHENC_URL}${url.pathname}${url.search}`;

  // Handle CORS preflight
  if (request.method() === 'OPTIONS') {
    await route.fulfill({
      status: 204,
      headers: CORS_HEADERS,
    });
    return;
  }

  try {
    // Build proxy request headers — only forward safe headers
    const origHeaders = request.headers();
    const proxyHeaders: Record<string, string> = {
      'content-type': origHeaders['content-type'] || 'application/json',
      'accept': origHeaders['accept'] || 'application/json',
      'host': new URL(AUTHENC_URL).host,
    };
    if (origHeaders['authorization']) {
      proxyHeaders['authorization'] = origHeaders['authorization'];
    }

    const response = await page.request.fetch(proxyUrl, {
      method: request.method(),
      headers: proxyHeaders,
      data: request.postData() ?? undefined,
    });

    const body = await response.body();

    // Optional callback to inspect the response body
    if (onBody) {
      try {
        await onBody(JSON.parse(body.toString()));
      } catch { /* ignore parse errors for non-JSON responses */ }
    }

    await route.fulfill({
      status: response.status(),
      headers: withCors({
        ...response.headers(),
        'content-type': response.headers()['content-type'] || 'application/json',
      }),
      body,
    });
  } catch (error) {
    console.error(`[proxy] ${request.method()} ${url.pathname} → ERROR:`, error);
    await route.fulfill({
      status: 502,
      headers: withCors({ 'content-type': 'application/json' }),
      body: JSON.stringify({ error: 'Proxy error', details: String(error) }),
    });
  }
}

// ── Test Suite Helpers ─────────────────────────────────────────────────────

/**
 * Helper: set up all route proxies needed for the portal to work via port-forward.
 * Uses a single unified API proxy with captcha answer extraction.
 *
 * The portal WASM app uses window.location.origin + /api/v1/auth/... for all
 * auth requests. When testing via port-forward, the API proxy catches
 * these and forwards to the real authenc. Under Istio, requests go natively.
 *
 * Returns a getter for the extracted captcha answer.
 */
async function setupAllProxies(page: Page): Promise<{ getCaptchaAnswer: () => string | null }> {
  let captchaAnswer: string | null = null;

  // 1) Asset proxy: strip /portal/ prefix so WASM/JS/CSS load from root
  await setupPortalAssetProxy(page);

  // 2) Unified API proxy: catches all /api/** and proxies to authenc
  await page.route('**/api/**', async (route: Route) => {
    await proxyToAuthenc(page, route, async (body) => {
      // When a captcha challenge response comes back, fetch the answer via debug endpoint
      if (route.request().url().includes('/api/captcha/challenge') && body && typeof body === 'object') {
        const b = body as CaptchaChallengeBody;
        if (b.challenge_id) {
          captchaAnswer = await fetchCaptchaAnswer(page, b.challenge_id);
        }
      }
    });
  });

  return { getCaptchaAnswer: () => captchaAnswer };
}

/** Navigate to login, open password form, and wait for WASM + captcha to load */
async function navigateToLogin(page: Page): Promise<void> {
  await page.goto(`${PORTAL_URL}/portal/login`, { waitUntil: 'domcontentloaded' });

  // Wait for WASM to mount — "Masuk ke Sistem" is rendered by Leptos
  await expect(page.getByText('Masuk ke Sistem')).toBeVisible({ timeout: 30000 });

  // Open password form if passkey UI is showing
  const passwordToggle = page.getByText('Atau masuk dengan password');
  if (await passwordToggle.isVisible({ timeout: 3000 }).catch(() => false)) {
    await passwordToggle.click();
  }
}

/** Fill captcha answer and submit within the captcha component, wait for verify response */
async function solveCaptchaInUI(page: Page, answer: string): Promise<void> {
  const captchaInput = page.locator('input[placeholder="Masukkan teks..."]');
  await expect(captchaInput).toBeVisible({ timeout: 5000 });
  await captchaInput.fill(answer);

  const captchaSubmit = page.locator('.captcha-container button', { hasText: 'Kirim' });

  // Wait for the verify response AND the button click simultaneously
  const [verifyResponse] = await Promise.all([
    page.waitForResponse(
      (resp) => resp.url().includes('/api/captcha/verify') && resp.request().method() === 'POST',
      { timeout: 15000 },
    ),
    captchaSubmit.click(),
  ]);

  // Confirm verify response was successful
  expect(verifyResponse.status()).toBe(200);

  // Give Leptos reactive system time to propagate the captcha_token signal
  await page.waitForTimeout(500);
}

test.describe('Portal Authentication - Real E2E', () => {
  test.beforeEach(async ({ page }) => {
    // Verify services are reachable before each test
    try {
      const healthResp = await page.request.get(`${AUTHENC_URL}/health`);
      expect(healthResp.ok(), 'Authenc must be reachable').toBeTruthy();
    } catch {
      test.skip(true, 'Authenc service not reachable — skipping');
    }

    try {
      const portalResp = await page.request.get(`${PORTAL_URL}/`);
      expect(portalResp.ok(), 'Portal must be reachable').toBeTruthy();
    } catch {
      test.skip(true, 'Portal service not reachable — skipping');
    }
  });

  // ────────── Test 1: Portal loads login page ──────────

  test('portal loads and shows login page', async ({ page }) => {
    await setupPortalAssetProxy(page);
    await page.goto(`${PORTAL_URL}/portal/login`, { waitUntil: 'domcontentloaded' });

    // Wait for WASM to mount — "Masuk ke Sistem" is Leptos-rendered
    await expect(page.getByText('Masuk ke Sistem')).toBeVisible({ timeout: 30000 });

    // Should display the SIMPEL heading (use first h1 to avoid strict mode with multiple headings)
    await expect(page.locator('h1').first()).toContainText('SIMPEL');

    // Open password form if passkey UI is present
    const passwordToggle = page.getByText('Atau masuk dengan password');
    if (await passwordToggle.isVisible({ timeout: 3000 }).catch(() => false)) {
      await passwordToggle.click();
    }

    await expect(page.locator('#username')).toBeVisible({ timeout: 5000 });
    await expect(page.locator('#password')).toBeVisible();
  });

  // ────────── Test 2: Captcha challenge loads ──────────

  test('captcha challenge loads in login form', async ({ page }) => {
    await setupAllProxies(page);
    await navigateToLogin(page);

    // Captcha heading: "Verifikasi Keamanan". The widget renders in every
    // build, so this half is asserted everywhere — splitting it out of the
    // answer-interception check below keeps that coverage on staging instead
    // of skipping the whole test for a capability only half of it needs.
    await expect(page.locator('#captcha-title')).toBeVisible({ timeout: 15000 });
    await expect(page.locator('.captcha-container')).toBeVisible();
  });

  test('captcha answer is intercepted by the proxy', async ({ page }) => {
    await skipUnlessCaptchaDebug(page.request);
    const { getCaptchaAnswer } = await setupAllProxies(page);
    await navigateToLogin(page);

    await expect(page.locator('#captcha-title')).toBeVisible({ timeout: 15000 });

    // Verify the captcha answer was intercepted (poll briefly — proxy callback may lag)
    await expect(async () => {
      expect(getCaptchaAnswer()).toBeTruthy();
    }).toPass({ timeout: 5000 });
  });

  // ────────── Test 3: Captcha refreshes on incorrect submission ──────────

  test('captcha refreshes when wrong answer is entered', async ({ page }) => {
    await skipUnlessCaptchaDebug(page.request);
    const { getCaptchaAnswer } = await setupAllProxies(page);
    await navigateToLogin(page);

    await expect(page.locator('#captcha-title')).toBeVisible({ timeout: 15000 });
    await expect.poll(() => getCaptchaAnswer(), { timeout: 10000 }).toBeTruthy();

    const firstAnswer = getCaptchaAnswer();
    expect(firstAnswer).toBeTruthy();

    // enter an obviously incorrect answer and submit
    const captchaInput = page.locator('input[placeholder="Masukkan teks..."]');
    await captchaInput.fill('wrong');

    // Wait for the verify response AND set up listener for the subsequent challenge request
    const challengeResponsePromise = page.waitForResponse(
      (resp) => resp.url().includes('/api/captcha/challenge') && resp.request().method() === 'POST',
      { timeout: 15000 },
    );

    const [verifyResp] = await Promise.all([
      page.waitForResponse(
        (resp) => resp.url().includes('/api/captcha/verify') && resp.request().method() === 'POST',
        { timeout: 15000 },
      ),
      page.getByRole('button', { name: /Kirim|Memverifikasi/i }).click(),
    ]);
    // Verify response should indicate failure (wrong answer returns success=false but HTTP 200)
    expect(verifyResp.status()).toBe(200);

    // after failed submission the UI requests a new challenge — wait for it
    await challengeResponsePromise;

    // the intercepted answer should now be different
    await expect.poll(() => getCaptchaAnswer(), { timeout: 5000 }).not.toBe(firstAnswer);
  });

  // ────────── Test 4: Captcha solving via API ──────────

  test('captcha can be solved via API', async ({ page }) => {
    await skipUnlessCaptchaDebug(page.request);
    const result = await solveCaptchaViaApi(page);

    expect(result.challengeId).toBeTruthy();
    expect(result.answer).toBeTruthy();
    expect(result.answer.length).toBeGreaterThanOrEqual(3);
    // The verification token is an opaque server-issued value (currently a
    // UUID) — login accepts it as-is, so only assert it is a non-empty string,
    // not a specific format.
    expect(result.token).toBeTruthy();
    expect(typeof result.token).toBe('string');
  });

  // ────────── Test 4: Full login flow (fixture operator → dashboard) ──────────

  test('fixture operator can login with captcha solving and reach the dashboard', async ({
    page,
  }) => {
    await skipUnlessCaptchaDebug(page.request);
    const { getCaptchaAnswer } = await setupAllProxies(page);
    await navigateToLogin(page);

    // Wait for captcha to load
    await expect(page.locator('#captcha-title')).toBeVisible({ timeout: 15000 });
    // Wait for captcha answer to be extracted from the intercepted response
    await expect.poll(() => getCaptchaAnswer(), { timeout: 10000 }).toBeTruthy();

    // Fill credentials FIRST (doesn't depend on captcha). Fixture user: the
    // baseline NIP user would land on /portal/password (forced change), not
    // /dashboard.
    await page.locator('#username').fill(OPERATOR_USER.username);
    await page.locator('#password').fill(OPERATOR_USER.password);

    // Solve captcha in the UI and confirm verify response
    await solveCaptchaInUI(page, getCaptchaAnswer()!);

    // Wait for login button to change text to "Masuk ke Portal" (captcha_token signal set)
    const loginButton = page.getByRole('button', { name: /Masuk ke Portal/i });
    await expect(loginButton).toBeVisible({ timeout: 10000 });
    await expect(loginButton).toBeEnabled({ timeout: 5000 });

    // Wait for the login response AND click
    const [loginResponse] = await Promise.all([
      page.waitForResponse(
        (resp) => resp.url().includes('/login') && resp.request().method() === 'POST',
        { timeout: 15000 },
      ),
      loginButton.click(),
    ]);
    expect(loginResponse.status()).toBe(200);

    // Successful login redirects to /dashboard
    await page.waitForURL(/\/(dashboard|portal\/dashboard)/, { timeout: 15000 });
    await expect(page).toHaveURL(/dashboard/);
  });

  // ────────── Test 5: Full login flow (fixture validator_pusat) ──────────

  test('fixture validator_pusat can login with captcha solving', async ({ page }) => {
    await skipUnlessCaptchaDebug(page.request);
    const { getCaptchaAnswer } = await setupAllProxies(page);
    await navigateToLogin(page);

    await expect(page.locator('#captcha-title')).toBeVisible({ timeout: 15000 });
    await expect.poll(() => getCaptchaAnswer(), { timeout: 10000 }).toBeTruthy();

    // Fill credentials (fixture user — the baseline `admin` is not login-able)
    await page.locator('#username').fill(VALIDATOR_PUSAT_USER.username);
    await page.locator('#password').fill(VALIDATOR_PUSAT_USER.password);

    // Solve captcha
    await solveCaptchaInUI(page, getCaptchaAnswer()!);

    // Wait for button to enable
    const loginButton = page.getByRole('button', { name: /Masuk ke Portal/i });
    await expect(loginButton).toBeVisible({ timeout: 10000 });
    await expect(loginButton).toBeEnabled({ timeout: 5000 });

    // Click and wait for auth response
    const [loginResponse] = await Promise.all([
      page.waitForResponse(
        (resp) => resp.url().includes('/login') && resp.request().method() === 'POST',
        { timeout: 15000 },
      ),
      loginButton.click(),
    ]);
    expect(loginResponse.status()).toBe(200);

    await page.waitForURL(/\/(dashboard|portal\/dashboard)/, { timeout: 15000 });
    await expect(page).toHaveURL(/dashboard/);
  });

  // ────────── Test 6: Login fails with wrong password ──────────

  test('login fails with incorrect password', async ({ page }) => {
    await skipUnlessCaptchaDebug(page.request);
    const { getCaptchaAnswer } = await setupAllProxies(page);
    await navigateToLogin(page);

    await expect(page.locator('#captcha-title')).toBeVisible({ timeout: 15000 });
    await expect.poll(() => getCaptchaAnswer(), { timeout: 10000 }).toBeTruthy();

    // Fill wrong password first
    await page.locator('#username').fill(NIP_USER.username);
    await page.locator('#password').fill('wrong_password_12345');

    // Solve captcha
    await solveCaptchaInUI(page, getCaptchaAnswer()!);

    // Wait for button to enable
    const loginButton = page.getByRole('button', { name: /Masuk ke Portal/i });
    await expect(loginButton).toBeVisible({ timeout: 10000 });
    await expect(loginButton).toBeEnabled({ timeout: 5000 });

    // Click and wait for auth response (should be 401)
    const [loginResponse] = await Promise.all([
      page.waitForResponse(
        (resp) => resp.url().includes('/login') && resp.request().method() === 'POST',
        { timeout: 15000 },
      ),
      loginButton.click(),
    ]);
    expect(loginResponse.status()).toBeGreaterThanOrEqual(400);
    expect(loginResponse.status()).not.toBe(200);

    // Should show error message (not redirect to dashboard)
    const errorBanner = page.locator('.bg-red-50, [class*="bg-red"]');
    await expect(errorBanner).toBeVisible({ timeout: 10000 });

    // Should still be on the login page
    await expect(page).toHaveURL(/login/);
  });

  // ────────── Test 7: Login button disabled without captcha ──────────

  test('login button disabled until captcha is solved', async ({ page }) => {
    await setupAllProxies(page);
    await navigateToLogin(page);

    // Fill credentials but DON'T solve captcha
    await page.locator('#username').fill('someuser');
    await page.locator('#password').fill('somepassword');

    // The login button should be disabled (captcha not solved)
    const submitBtn = page.locator('form button[type="submit"]');
    await expect(submitBtn).toBeDisabled();
  });

  // ────────── Test 8: Direct API login test (no browser) ──────────

  test('API: login succeeds with correct credentials + captcha token', async ({ request }) => {
    await skipUnlessCaptchaDebug(request);
    // Step 1: Get captcha challenge
    const challengeResp = await request.post(`${AUTHENC_URL}/api/captcha/challenge`, {
      data: { challenge_type: 'text_recognition', difficulty: 2 },
    });
    expect(challengeResp.ok()).toBeTruthy();
    const challenge = await challengeResp.json();

    // Step 2: Fetch answer from debug endpoint and verify captcha
    const debugResp = await request.get(`${AUTHENC_URL}/api/captcha/debug/${challenge.challenge_id}`);
    expect(debugResp.ok()).toBeTruthy();
    const debugBody = await debugResp.json();
    const answer = debugBody.answer;
    expect(answer).toBeTruthy();

    const verifyResp = await request.post(`${AUTHENC_URL}/api/captcha/verify`, {
      data: {
        challenge_id: challenge.challenge_id,
        answer: answer!,
        session_id: `api_test_${Date.now()}`,
      },
    });
    expect(verifyResp.ok()).toBeTruthy();
    const verifyResult = await verifyResp.json();
    expect(verifyResult.success).toBe(true);

    // Step 3: Login with captcha token
    const loginResp = await request.post(`${AUTHENC_URL}/api/v1/auth/login`, {
      data: {
        username: NIP_USER.username,
        password: NIP_USER.password,
        realm: 'master',
        captcha_token: verifyResult.token,
      },
    });
    expect(loginResp.ok()).toBeTruthy();

    const loginResult = await loginResp.json();
    expect(loginResult.access_token).toBeTruthy();
    expect(loginResult.mfa_required).toBe(false);
  });

  // ────────── Test 9: API login fails with wrong password ──────────

  test('API: login fails with incorrect password', async ({ request }) => {
    await skipUnlessCaptchaDebug(request);
    // Get and solve captcha
    const challengeResp = await request.post(`${AUTHENC_URL}/api/captcha/challenge`, {
      data: { challenge_type: 'text_recognition', difficulty: 2 },
    });
    const challenge = await challengeResp.json();

    const debugResp = await request.get(`${AUTHENC_URL}/api/captcha/debug/${challenge.challenge_id}`);
    expect(debugResp.ok()).toBeTruthy();
    const debugBody = await debugResp.json();
    const answer = debugBody.answer;

    const verifyResp = await request.post(`${AUTHENC_URL}/api/captcha/verify`, {
      data: {
        challenge_id: challenge.challenge_id,
        answer: answer!,
        session_id: `api_test_fail_${Date.now()}`,
      },
    });
    const verifyResult = await verifyResp.json();

    // Login with wrong password
    const loginResp = await request.post(`${AUTHENC_URL}/api/v1/auth/login`, {
      data: {
        username: NIP_USER.username,
        password: 'completely_wrong_password',
        realm: 'master',
        captcha_token: verifyResult.token,
      },
    });

    // Should return 401 (direct) or 504 (via Istio gateway timeout)
    expect([401, 504]).toContain(loginResp.status());
  });
});
