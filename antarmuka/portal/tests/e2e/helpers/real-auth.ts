/**
 * Shared real-auth helpers for portal e2e specs.
 *
 * Same mechanics as portal-auth-e2e.spec.ts (kept inline there — importing a
 * spec from a spec would re-register its tests): the portal WASM calls
 * /api/** origin-relative, so when the FE is reached directly (compose/CI)
 * every /api/** request is proxied by page.route() to the real authenc, and
 * the captcha is solved via the CAPTCHA_DEBUG answer endpoint.
 */
import { APIRequestContext, Page, Route, expect, test } from '@playwright/test';

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

/**
 * Cached answer to "does this origin already serve the portal under /portal/?"
 *
 * `null` until probed. Probing beats naming: the previous form of this check
 * asked whether the host was `localhost`, which is a guess about topology
 * dressed up as a fact — a compose stack published on a LAN address would have
 * answered it wrong.
 */
let ingressServesPortalPath: boolean | null = null;

async function servesPortalNatively(page: Page): Promise<boolean> {
  if (ingressServesPortalPath !== null) return ingressServesPortalPath;
  try {
    const resp = await page.request.get(`${PORTAL_URL}/portal/`, { timeout: 10000 });
    const body = resp.ok() ? await resp.text() : '';
    ingressServesPortalPath = resp.ok() && body.includes('<base href="/portal/"');
  } catch {
    ingressServesPortalPath = false;
  }
  return ingressServesPortalPath;
}

/**
 * Strip the /portal/ prefix for assets when — and ONLY when — the frontend is
 * served from the origin root.
 *
 * In compose the portal container answers at `/`, so a request for
 * `/portal/login` has to be rewritten to `/login`. Behind the staging ingress
 * the opposite is true: `/portal/login` is served natively and `/login`
 * returns 404, so the rewrite fetched a page that does not exist. The document
 * came back empty, the WASM app never mounted, and every assertion failed with
 * `element(s) not found` — including `getByText('Masuk ke Sistem')`, which
 * renders perfectly well when the page is opened directly.
 *
 * The condition for this already existed here, with a comment saying exactly
 * what should happen under an ingress. The `if` body was empty, so the route
 * was installed regardless: the knowledge was present and the enforcement was
 * not.
 */
async function setupPortalAssetProxy(page: Page): Promise<void> {
  if (await servesPortalNatively(page)) {
    // The origin routes /portal/** itself. Rewriting here would send every
    // asset request to a path this origin does not serve.
    return;
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

/**
 * Is the captcha answer readable from this build at all?
 *
 * `GET /api/captcha/debug/{id}` exists only when authenc is compiled with
 * `--features captcha-debug` (see `router.rs`: the route is behind `#[cfg]`,
 * so in a release image it is not merely disabled — it is absent). That is
 * correct: an environment that hands out captcha answers has no captcha.
 *
 * The consequence is structural, not incidental. Any spec that drives the
 * LOGIN FORM needs the answer, so those specs can only ever run where the
 * debug build runs — CI. Against staging or production they must SKIP, and say
 * why, rather than fail 21 times with `expect(received).toBeTruthy()` and
 * `Received: null`, which reads like a product defect and is not one.
 *
 * Specs that only need a session should use `loginViaApi` instead, which needs
 * no captcha and does run against staging.
 */
const captchaDebugByOrigin = new Map<string, boolean>();

/**
 * @param base origin to probe. Defaults to AUTHENC_URL; pass `''` from specs
 *   whose requests are baseURL-relative (the single-origin cross-app ingress),
 *   so the probe asks the same server the spec will actually talk to.
 */
export async function hasCaptchaDebugEndpoint(
  api: APIRequestContext,
  base: string = AUTHENC_URL,
): Promise<boolean> {
  const cached = captchaDebugByOrigin.get(base);
  if (cached !== undefined) return cached;
  let available = false;
  try {
    const challenge = await api.post(`${base}/api/captcha/challenge`, {
      data: {},
      headers: { 'content-type': 'application/json' },
      timeout: 10000,
    });
    if (challenge.ok()) {
      const id = (await challenge.json()).challenge_id;
      const probe = await api.get(`${base}/api/captcha/debug/${id}`, { timeout: 10000 });
      available = probe.ok();
    }
  } catch {
    available = false;
  }
  captchaDebugByOrigin.set(base, available);
  return available;
}

/**
 * Skip the calling test unless the captcha answer is readable from this build.
 *
 * Lives here rather than being pasted into each `beforeEach` so the reason
 * stays in one place: a skip whose message drifts out of date is worse than no
 * message, because the next reader trusts it.
 */
export async function skipUnlessCaptchaDebug(
  api: APIRequestContext,
  base: string = AUTHENC_URL,
): Promise<void> {
  test.skip(
    !(await hasCaptchaDebugEndpoint(api, base)),
    'authenc is built without --features captcha-debug, so the captcha answer ' +
      'cannot be read and the login FORM cannot be driven. Correct for a ' +
      'production-like build; use loginViaApi for session setup instead.',
  );
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

  // A WASM panic unmounts the whole Leptos app: the URL stays put but nothing
  // renders, so every later assertion fails with an unhelpful "element not
  // found". Surface the panic itself in the CI log instead.
  page.on('pageerror', (err) => {
    console.error(`[portal pageerror] ${err.message}`);
  });
  page.on('console', (msg) => {
    if (msg.type() === 'error') {
      console.error(`[portal console.error] ${msg.text()}`);
    }
  });

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
 * Start tracking the captcha challenges a login page issues, and return a
 * getter that resolves the latest one's answer.
 *
 * MUST be called BEFORE navigating to the login page. The WASM requests a
 * challenge as the page loads, so a listener attached after `goto` (or after an
 * `isVisible` probe) can miss it entirely and then wait forever on an empty
 * list — the failure mode is a 10s timeout, not a wrong answer. Returning a
 * getter rather than a value is what lets the caller attach early and read
 * late.
 *
 * Answers come from the debug endpoint, which only exists in a captcha-debug
 * build; `skipUnlessCaptchaDebug` is the caller's guard.
 */
export function trackCaptchaChallenges(page: Page): () => Promise<string | null> {
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

  return async () => {
    for (let i = 0; i < 50 && challengeIds.length === 0; i++) {
      await new Promise((resolve) => setTimeout(resolve, 200));
    }
    if (challengeIds.length === 0) return null;
    const challengeId = challengeIds[challengeIds.length - 1];
    const dbg = await page.request.get(`/api/captcha/debug/${challengeId}`);
    if (!dbg.ok()) return null;
    return (await dbg.json()).answer as string;
  };
}

/**
 * Full UI login against a SINGLE-ORIGIN stack (the cross-app ingress), where
 * the WASM's origin-relative `/api/**` calls already reach authenc — so no
 * `page.route` proxy is needed.
 *
 * Distinct from `loginViaUi`, which needs `setupAllProxies` because the compose
 * two-origin stack would otherwise send `/api/**` to the FE container. Both
 * live here so the specs that drive a real login form share one implementation
 * — the login form is the most selector-fragile surface in the suite, and a
 * second copy drifts.
 *
 * `getCaptchaAnswer` must come from `trackCaptchaChallenges(page)` called
 * before the navigation that lands on the login page.
 *
 * Leaves the browser wherever the app decides to go (dashboard, or a handoff
 * target); the caller asserts the destination.
 */
export async function loginViaSameOriginUi(
  page: Page,
  user: { username: string; password: string },
  getCaptchaAnswer: () => Promise<string | null>,
): Promise<void> {
  await expect(page.getByText('Masuk ke Sistem')).toBeVisible({ timeout: 30000 });
  const passwordToggle = page.getByText('Atau masuk dengan password');
  if (await passwordToggle.isVisible({ timeout: 3000 }).catch(() => false)) {
    await passwordToggle.click();
  }

  const captchaInput = page.locator('input[placeholder="Masukkan teks..."]');
  await expect(captchaInput).toBeVisible({ timeout: 15000 });

  const answer = await getCaptchaAnswer();
  expect(answer, 'captcha answer from the debug endpoint').toBeTruthy();

  await page.locator('#username').fill(user.username);
  await page.locator('#password').fill(user.password);
  await captchaInput.fill(answer!);

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
