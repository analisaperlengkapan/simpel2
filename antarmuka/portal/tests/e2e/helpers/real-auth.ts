/**
 * Shared real-auth helpers for portal e2e specs.
 *
 * Same mechanics as portal-auth-e2e.spec.ts (kept inline there — importing a
 * spec from a spec would re-register its tests): the portal WASM calls
 * /api/** origin-relative, so when the FE is reached directly (compose/CI)
 * every /api/** request is proxied by page.route() to the real authenc, and
 * the captcha is solved via the CAPTCHA_DEBUG answer endpoint.
 */
import { Page, Route, expect } from '@playwright/test';

export const PORTAL_URL = process.env.PORTAL_URL || 'http://localhost:18080';
export const AUTHENC_URL = process.env.AUTHENC_URL || 'http://localhost:18088';

/** Fixture users (tests/fixtures/e2e/seed-multisatker.sql). */
export const SEED_PASSWORD = '199203142014031001';
export const OPERATOR_USER = { username: '200000000000000001', password: SEED_PASSWORD };
export const OPERATOR_B_USER = { username: '200000000000000002', password: SEED_PASSWORD };
export const ADMIN_USER = { username: '200000000000000005', password: SEED_PASSWORD };

const CORS_HEADERS: Record<string, string> = {
  'access-control-allow-origin': '*',
  'access-control-allow-methods': 'GET, POST, PUT, PATCH, DELETE, OPTIONS',
  'access-control-allow-headers': 'content-type, authorization',
};

/** Strip the /portal/ prefix for assets when the FE is served from root. */
async function setupPortalAssetProxy(page: Page): Promise<void> {
  const portalHost = new URL(PORTAL_URL).hostname;
  if (portalHost !== 'localhost' && portalHost !== '127.0.0.1') {
    // Under an ingress the /portal/ routing is native — only the /api proxy
    // below is needed.
  }

  await page.route(`${PORTAL_URL}/portal/**`, async (route: Route) => {
    const url = new URL(route.request().url());
    if (url.pathname.startsWith('/portal/api/') || url.pathname.startsWith('/api/')) {
      await route.continue();
      return;
    }
    const newPath = url.pathname.replace(/^\/portal\//, '/');
    try {
      const response = await page.request.fetch(`${PORTAL_URL}${newPath}${url.search}`, {
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

async function fetchCaptchaAnswer(page: Page, challengeId: string): Promise<string | null> {
  try {
    const resp = await page.request.get(`${AUTHENC_URL}/api/captcha/debug/${challengeId}`);
    if (!resp.ok()) return null;
    return (await resp.json()).answer ?? null;
  } catch {
    return null;
  }
}

/**
 * Register the asset proxy + the unified /api/** → authenc proxy.
 * Returns a getter for the captcha answer extracted from intercepted
 * challenge responses.
 */
export async function setupAllProxies(
  page: Page,
): Promise<{ getCaptchaAnswer: () => string | null }> {
  let captchaAnswer: string | null = null;

  await setupPortalAssetProxy(page);

  await page.route('**/api/**', async (route: Route) => {
    const request = route.request();
    const url = new URL(request.url());

    if (request.method() === 'OPTIONS') {
      await route.fulfill({ status: 204, headers: CORS_HEADERS });
      return;
    }

    try {
      const origHeaders = request.headers();
      const proxyHeaders: Record<string, string> = {
        'content-type': origHeaders['content-type'] || 'application/json',
        accept: origHeaders['accept'] || 'application/json',
        host: new URL(AUTHENC_URL).host,
      };
      if (origHeaders['authorization']) {
        proxyHeaders['authorization'] = origHeaders['authorization'];
      }

      const response = await page.request.fetch(
        `${AUTHENC_URL}${url.pathname}${url.search}`,
        {
          method: request.method(),
          headers: proxyHeaders,
          data: request.postData() ?? undefined,
        },
      );

      const body = await response.body();
      if (url.pathname.includes('/api/captcha/challenge')) {
        try {
          const parsed = JSON.parse(body.toString());
          if (parsed?.challenge_id) {
            captchaAnswer = await fetchCaptchaAnswer(page, parsed.challenge_id);
          }
        } catch {
          /* non-JSON challenge response — ignore */
        }
      }

      await route.fulfill({
        status: response.status(),
        headers: {
          ...response.headers(),
          ...CORS_HEADERS,
          'content-type': response.headers()['content-type'] || 'application/json',
        },
        body,
      });
    } catch (error) {
      await route.fulfill({
        status: 502,
        headers: { ...CORS_HEADERS, 'content-type': 'application/json' },
        body: JSON.stringify({ error: 'Proxy error', details: String(error) }),
      });
    }
  });

  return { getCaptchaAnswer: () => captchaAnswer };
}

/**
 * Full UI login: navigate to /portal/login, solve the captcha in the real
 * captcha component, submit real credentials, wait for the app to leave the
 * login page. `setupAllProxies` must have been called on the page first.
 */
export async function loginViaUi(
  page: Page,
  user: { username: string; password: string },
  getCaptchaAnswer: () => string | null,
): Promise<void> {
  await page.goto(`${PORTAL_URL}/portal/login`, { waitUntil: 'domcontentloaded' });
  await expect(page.getByText('Masuk ke Sistem')).toBeVisible({ timeout: 30000 });

  const passwordToggle = page.getByText('Atau masuk dengan password');
  if (await passwordToggle.isVisible({ timeout: 3000 }).catch(() => false)) {
    await passwordToggle.click();
  }

  await expect(page.locator('#captcha-title')).toBeVisible({ timeout: 15000 });
  await expect.poll(() => getCaptchaAnswer(), { timeout: 10000 }).toBeTruthy();

  await page.locator('#username').fill(user.username);
  await page.locator('#password').fill(user.password);

  const captchaInput = page.locator('input[placeholder="Masukkan teks..."]');
  await expect(captchaInput).toBeVisible({ timeout: 5000 });
  await captchaInput.fill(getCaptchaAnswer()!);

  const captchaSubmit = page.locator('.captcha-container button', { hasText: 'Kirim' });
  const [verifyResponse] = await Promise.all([
    page.waitForResponse(
      (resp) => resp.url().includes('/api/captcha/verify') && resp.request().method() === 'POST',
      { timeout: 15000 },
    ),
    captchaSubmit.click(),
  ]);
  expect(verifyResponse.status()).toBe(200);
  await page.waitForTimeout(500);

  const loginButton = page.getByRole('button', { name: /Masuk ke Portal/i });
  await expect(loginButton).toBeVisible({ timeout: 10000 });
  await expect(loginButton).toBeEnabled({ timeout: 5000 });

  const [loginResponse] = await Promise.all([
    page.waitForResponse(
      (resp) => resp.url().includes('/login') && resp.request().method() === 'POST',
      { timeout: 15000 },
    ),
    loginButton.click(),
  ]);
  expect(loginResponse.status()).toBe(200);

  // Fixture users land on the dashboard (require_password_change=false).
  await page.waitForURL(/\/(dashboard|portal\/dashboard)/, { timeout: 15000 });
}

/**
 * API login (no browser): solve captcha via debug endpoint, POST login,
 * return the tokens. Used for backend cross-checks and to mint additional
 * sessions.
 */
export async function loginViaApi(
  page: Page,
  user: { username: string; password: string },
): Promise<{ access_token: string; refresh_token?: string }> {
  const challengeResp = await page.request.post(`${AUTHENC_URL}/api/captcha/challenge`, {
    data: { challenge_type: 'text_recognition', difficulty: 2 },
  });
  expect(challengeResp.ok()).toBeTruthy();
  const challenge = await challengeResp.json();

  const answer = await fetchCaptchaAnswer(page, challenge.challenge_id);
  expect(answer).toBeTruthy();

  const verifyResp = await page.request.post(`${AUTHENC_URL}/api/captcha/verify`, {
    data: {
      challenge_id: challenge.challenge_id,
      answer: answer!,
      session_id: `e2e_core_${Date.now()}_${Math.floor(Math.random() * 1e6)}`,
    },
  });
  expect(verifyResp.ok()).toBeTruthy();
  const verifyResult = await verifyResp.json();
  expect(verifyResult.success).toBe(true);

  const loginResp = await page.request.post(`${AUTHENC_URL}/api/v1/auth/login`, {
    data: {
      username: user.username,
      password: user.password,
      realm: 'master',
      captcha_token: verifyResult.token,
    },
  });
  expect(loginResp.status(), `API login for ${user.username}`).toBe(200);
  return loginResp.json();
}

/** Read the app's JWT from localStorage (canonical `auth_token` key). */
export async function pageToken(page: Page): Promise<string> {
  const token = await page.evaluate(() => window.localStorage.getItem('auth_token'));
  expect(token, 'auth_token must be present after login').toBeTruthy();
  return token!;
}
