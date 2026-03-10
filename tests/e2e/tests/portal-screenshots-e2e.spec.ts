/**
 * Portal Screenshot E2E Tests
 *
 * Navigates ALL portal pages and captures full-page screenshots.
 * Tests run against the staging Istio gateway (10.1.7.121) with
 * real authenc backend integration.
 *
 * Screenshots saved to: tests/e2e/test-results/screenshots/
 *
 * Routes tested:
 * - Public: /, /login, /logged-out, /mfa/*
 * - Protected: /dashboard, /apps, /profile, /settings, /sessions, /notifications, /monitoring, /passkeys, /password
 * - Admin: /admin, /admin/users, /admin/realms, /admin/clients, /admin/roles,
 *          /admin/federation, /admin/permissions, /admin/groups, /admin/realm-settings,
 *          /admin/auth-flows, /admin/linked-accounts, /admin/audit
 */
import { test, expect, Page, Route } from '@playwright/test';
import path from 'path';
import fs from 'fs';

// ── Configuration ──────────────────────────────────────────────────────────

const BASE_URL = process.env.PORTAL_URL || 'http://10.1.7.121';
const AUTHENC_URL = process.env.AUTHENC_URL || 'http://10.1.7.121';
const PORTAL_PATH = '/portal';
const SCREENSHOT_DIR = path.join(__dirname, '..', 'test-results', 'screenshots');

// Admin user (has access to all pages)
const ADMIN_USER = {
    username: 'admin',
    password: 'admin',
};

// Regular NIP user
const NIP_USER = {
    username: '199203142014031001',
    password: '199203142014031001',
};

// ── Captcha Helpers ────────────────────────────────────────────────────────

interface CaptchaChallengeData {
    challenge_type: string;
    data: string;
    instructions: string;
}

function extractCaptchaAnswer(challengeDataJson: string): string | null {
    try {
        const parsed: CaptchaChallengeData = JSON.parse(challengeDataJson);
        if (parsed.challenge_type === 'text_recognition' && parsed.data) {
            const parts = parsed.data.split(':');
            if (parts.length >= 2) return parts[0];
        }
    } catch { /* ignore */ }
    return null;
}

/**
 * Fetch the captcha answer from the debug endpoint.
 * Requires CAPTCHA_DEBUG=true on the authenc server (or --features captcha-debug).
 *
 * GET /api/captcha/debug/{challenge_id}
 */
async function fetchCaptchaDebugAnswer(challengeId: string): Promise<string | null> {
    try {
        const resp = await fetch(`${AUTHENC_URL}/api/captcha/debug/${challengeId}`);
        if (!resp.ok) return null;
        const body = await resp.json();
        return body.answer ?? null;
    } catch {
        return null;
    }
}

// ── Screenshot Helper ──────────────────────────────────────────────────────

async function captureScreenshot(page: Page, name: string): Promise<void> {
    fs.mkdirSync(SCREENSHOT_DIR, { recursive: true });
    const filePath = path.join(SCREENSHOT_DIR, `${name}.png`);
    await page.screenshot({ path: filePath, fullPage: true });
}

// ── API Proxy for captcha solving via Istio ────────────────────────────────

/**
 * When running via Istio ingress (/portal/), API calls from WASM go to
 * window.location.origin + /api/*, which Istio routes to authenc.
 * No proxy needed if CORS is handled at the gateway level.
 *
 * For captcha answer extraction, we intercept API responses.
 */
async function setupCaptchaInterceptor(page: Page): Promise<{ getCaptchaAnswer: () => string | null }> {
    let captchaAnswer: string | null = null;

    await page.route('**/api/captcha/challenge', async (route: Route) => {
        const response = await route.fetch();
        const body = await response.json().catch(() => null);
        if (body?.challenge_id) {
            // Prefer the debug endpoint (accurate, case-sensitive)
            const debugAnswer = await fetchCaptchaDebugAnswer(body.challenge_id);
            if (debugAnswer) {
                captchaAnswer = debugAnswer;
            } else if (body.challenge_data) {
                // Fallback to legacy parsing
                captchaAnswer = extractCaptchaAnswer(body.challenge_data);
            }
        }
        await route.fulfill({ response });
    });

    return { getCaptchaAnswer: () => captchaAnswer };
}

// ── Login Helper ───────────────────────────────────────────────────────────

async function loginAs(
    page: Page,
    user: { username: string; password: string },
    interceptor: { getCaptchaAnswer: () => string | null },
): Promise<void> {
    await page.goto(`${BASE_URL}${PORTAL_PATH}/login`, { waitUntil: 'domcontentloaded' });

    // Wait for WASM to mount
    await expect(page.getByText('Masuk ke Sistem')).toBeVisible({ timeout: 30000 });

    // Open password form if passkey UI is showing
    const passwordToggle = page.getByText('Atau masuk dengan password');
    if (await passwordToggle.isVisible({ timeout: 3000 }).catch(() => false)) {
        await passwordToggle.click();
    }

    // Fill credentials
    await page.locator('#username').fill(user.username);
    await page.locator('#password').fill(user.password);

    // Wait for captcha to load and solve it
    await expect(page.locator('#captcha-title')).toBeVisible({ timeout: 15000 });
    await expect.poll(() => interceptor.getCaptchaAnswer(), { timeout: 10000 }).toBeTruthy();

    const captchaInput = page.locator('input[placeholder="Masukkan teks..."]');
    await expect(captchaInput).toBeVisible({ timeout: 5000 });
    await captchaInput.fill(interceptor.getCaptchaAnswer()!);

    const captchaSubmit = page.locator('.captcha-container button', { hasText: 'Kirim' });
    await Promise.all([
        page.waitForResponse(
            (resp) => resp.url().includes('/api/captcha/verify') && resp.request().method() === 'POST',
            { timeout: 15000 },
        ),
        captchaSubmit.click(),
    ]);
    await page.waitForTimeout(500);

    // Click login button
    const loginButton = page.getByRole('button', { name: /Masuk ke Portal/i });
    await expect(loginButton).toBeVisible({ timeout: 10000 });
    await expect(loginButton).toBeEnabled({ timeout: 5000 });

    await Promise.all([
        page.waitForResponse(
            (resp) => resp.url().includes('/login') && resp.request().method() === 'POST',
            { timeout: 15000 },
        ),
        loginButton.click(),
    ]);

    // Wait for redirect to dashboard
    await page.waitForURL(/\/(dashboard|portal\/dashboard)/, { timeout: 15000 });
}

// ════════════════════════════════════════════════════════════════════════════
// TEST SUITES
// ════════════════════════════════════════════════════════════════════════════

test.describe('Portal Screenshots - Public Pages', () => {
    test.beforeEach(async ({ page }) => {
        // Verify portal is reachable
        const resp = await page.request.get(`${BASE_URL}${PORTAL_PATH}/`);
        expect(resp.ok(), 'Portal must be reachable at staging').toBeTruthy();
    });

    test('01 - Home page (landing)', async ({ page }) => {
        await page.goto(`${BASE_URL}${PORTAL_PATH}/`, { waitUntil: 'domcontentloaded' });
        await page.waitForTimeout(2000); // Wait for WASM mount
        await captureScreenshot(page, '01-home-landing');
    });

    test('02 - Login page', async ({ page }) => {
        await page.goto(`${BASE_URL}${PORTAL_PATH}/login`, { waitUntil: 'domcontentloaded' });
        await expect(page.getByText('Masuk ke Sistem')).toBeVisible({ timeout: 30000 });
        await captureScreenshot(page, '02-login-page');
    });

    test('03 - Login page with password form', async ({ page }) => {
        await page.goto(`${BASE_URL}${PORTAL_PATH}/login`, { waitUntil: 'domcontentloaded' });
        await expect(page.getByText('Masuk ke Sistem')).toBeVisible({ timeout: 30000 });

        const passwordToggle = page.getByText('Atau masuk dengan password');
        if (await passwordToggle.isVisible({ timeout: 3000 }).catch(() => false)) {
            await passwordToggle.click();
        }
        await page.waitForTimeout(500);
        await captureScreenshot(page, '03-login-password-form');
    });

    test('04 - Login page with captcha loaded', async ({ page }) => {
        const interceptor = await setupCaptchaInterceptor(page);
        await page.goto(`${BASE_URL}${PORTAL_PATH}/login`, { waitUntil: 'domcontentloaded' });
        await expect(page.getByText('Masuk ke Sistem')).toBeVisible({ timeout: 30000 });

        const passwordToggle = page.getByText('Atau masuk dengan password');
        if (await passwordToggle.isVisible({ timeout: 3000 }).catch(() => false)) {
            await passwordToggle.click();
        }

        await expect(page.locator('#captcha-title')).toBeVisible({ timeout: 15000 });
        await captureScreenshot(page, '04-login-with-captcha');
    });

    test('05 - Logged out page', async ({ page }) => {
        await page.goto(`${BASE_URL}${PORTAL_PATH}/logged-out`, { waitUntil: 'domcontentloaded' });
        await page.waitForTimeout(2000);
        await captureScreenshot(page, '05-logged-out');
    });

    test('06 - MFA setup page (unauthenticated redirect)', async ({ page }) => {
        await page.goto(`${BASE_URL}${PORTAL_PATH}/mfa/setup`, { waitUntil: 'domcontentloaded' });
        await page.waitForTimeout(2000);
        await captureScreenshot(page, '06-mfa-setup');
    });

    test('07 - MFA verify page', async ({ page }) => {
        await page.goto(`${BASE_URL}${PORTAL_PATH}/mfa/verify`, { waitUntil: 'domcontentloaded' });
        await page.waitForTimeout(2000);
        await captureScreenshot(page, '07-mfa-verify');
    });

    test('08 - Not found page (404)', async ({ page }) => {
        await page.goto(`${BASE_URL}${PORTAL_PATH}/nonexistent-page`, { waitUntil: 'domcontentloaded' });
        await page.waitForTimeout(2000);
        await captureScreenshot(page, '08-not-found-404');
    });
});

test.describe('Portal Screenshots - Authenticated Pages (Admin)', () => {
    let interceptor: { getCaptchaAnswer: () => string | null };

    test.beforeEach(async ({ page }) => {
        interceptor = await setupCaptchaInterceptor(page);
        await loginAs(page, ADMIN_USER, interceptor);
    });

    test('10 - Dashboard', async ({ page }) => {
        await expect(page).toHaveURL(/dashboard/);
        await page.waitForTimeout(1000);
        await captureScreenshot(page, '10-dashboard');
    });

    test('11 - Apps page', async ({ page }) => {
        await page.goto(`${BASE_URL}${PORTAL_PATH}/apps`, { waitUntil: 'domcontentloaded' });
        await page.waitForTimeout(1000);
        await captureScreenshot(page, '11-apps');
    });

    test('12 - Profile page', async ({ page }) => {
        await page.goto(`${BASE_URL}${PORTAL_PATH}/profile`, { waitUntil: 'domcontentloaded' });
        await page.waitForTimeout(1000);
        await captureScreenshot(page, '12-profile');
    });

    test('13 - Settings page', async ({ page }) => {
        await page.goto(`${BASE_URL}${PORTAL_PATH}/settings`, { waitUntil: 'domcontentloaded' });
        await page.waitForTimeout(1000);
        await captureScreenshot(page, '13-settings');
    });

    test('14 - Sessions page', async ({ page }) => {
        await page.goto(`${BASE_URL}${PORTAL_PATH}/sessions`, { waitUntil: 'domcontentloaded' });
        await page.waitForTimeout(1000);
        await captureScreenshot(page, '14-sessions');
    });

    test('15 - Notifications page', async ({ page }) => {
        await page.goto(`${BASE_URL}${PORTAL_PATH}/notifications`, { waitUntil: 'domcontentloaded' });
        await page.waitForTimeout(1000);
        await captureScreenshot(page, '15-notifications');
    });

    test('16 - Monitoring page', async ({ page }) => {
        await page.goto(`${BASE_URL}${PORTAL_PATH}/monitoring`, { waitUntil: 'domcontentloaded' });
        await page.waitForTimeout(1000);
        await captureScreenshot(page, '16-monitoring');
    });

    test('17 - Passkeys page', async ({ page }) => {
        await page.goto(`${BASE_URL}${PORTAL_PATH}/passkeys`, { waitUntil: 'domcontentloaded' });
        await page.waitForTimeout(1000);
        await captureScreenshot(page, '17-passkeys');
    });

    test('18 - Password change page', async ({ page }) => {
        await page.goto(`${BASE_URL}${PORTAL_PATH}/password`, { waitUntil: 'domcontentloaded' });
        await page.waitForTimeout(1000);
        await captureScreenshot(page, '18-password-change');
    });

    test('19 - Pembinaan page', async ({ page }) => {
        await page.goto(`${BASE_URL}${PORTAL_PATH}/pembinaan`, { waitUntil: 'domcontentloaded' });
        await page.waitForTimeout(1000);
        await captureScreenshot(page, '19-pembinaan');
    });

    // ── Admin Pages ──────────────────────────────────────────────────────────

    test('20 - Admin overview', async ({ page }) => {
        await page.goto(`${BASE_URL}${PORTAL_PATH}/admin`, { waitUntil: 'domcontentloaded' });
        await page.waitForTimeout(1000);
        await captureScreenshot(page, '20-admin-overview');
    });

    test('21 - Admin users management', async ({ page }) => {
        await page.goto(`${BASE_URL}${PORTAL_PATH}/admin/users`, { waitUntil: 'domcontentloaded' });
        await page.waitForTimeout(1000);
        await captureScreenshot(page, '21-admin-users');
    });

    test('22 - Admin realms management', async ({ page }) => {
        await page.goto(`${BASE_URL}${PORTAL_PATH}/admin/realms`, { waitUntil: 'domcontentloaded' });
        await page.waitForTimeout(1000);
        await captureScreenshot(page, '22-admin-realms');
    });

    test('23 - Admin clients management', async ({ page }) => {
        await page.goto(`${BASE_URL}${PORTAL_PATH}/admin/clients`, { waitUntil: 'domcontentloaded' });
        await page.waitForTimeout(1000);
        await captureScreenshot(page, '23-admin-clients');
    });

    test('24 - Admin roles management', async ({ page }) => {
        await page.goto(`${BASE_URL}${PORTAL_PATH}/admin/roles`, { waitUntil: 'domcontentloaded' });
        await page.waitForTimeout(1000);
        await captureScreenshot(page, '24-admin-roles');
    });

    test('25 - Admin federation management', async ({ page }) => {
        await page.goto(`${BASE_URL}${PORTAL_PATH}/admin/federation`, { waitUntil: 'domcontentloaded' });
        await page.waitForTimeout(1000);
        await captureScreenshot(page, '25-admin-federation');
    });

    test('26 - Admin permissions management', async ({ page }) => {
        await page.goto(`${BASE_URL}${PORTAL_PATH}/admin/permissions`, { waitUntil: 'domcontentloaded' });
        await page.waitForTimeout(1000);
        await captureScreenshot(page, '26-admin-permissions');
    });

    test('27 - Admin groups management', async ({ page }) => {
        await page.goto(`${BASE_URL}${PORTAL_PATH}/admin/groups`, { waitUntil: 'domcontentloaded' });
        await page.waitForTimeout(1000);
        await captureScreenshot(page, '27-admin-groups');
    });

    test('28 - Admin realm settings', async ({ page }) => {
        await page.goto(`${BASE_URL}${PORTAL_PATH}/admin/realm-settings`, { waitUntil: 'domcontentloaded' });
        await page.waitForTimeout(1000);
        await captureScreenshot(page, '28-admin-realm-settings');
    });

    test('29 - Admin auth flows', async ({ page }) => {
        await page.goto(`${BASE_URL}${PORTAL_PATH}/admin/auth-flows`, { waitUntil: 'domcontentloaded' });
        await page.waitForTimeout(1000);
        await captureScreenshot(page, '29-admin-auth-flows');
    });

    test('30 - Admin linked accounts', async ({ page }) => {
        await page.goto(`${BASE_URL}${PORTAL_PATH}/admin/linked-accounts`, { waitUntil: 'domcontentloaded' });
        await page.waitForTimeout(1000);
        await captureScreenshot(page, '30-admin-linked-accounts');
    });

    test('31 - Admin audit logs', async ({ page }) => {
        await page.goto(`${BASE_URL}${PORTAL_PATH}/admin/audit`, { waitUntil: 'domcontentloaded' });
        await page.waitForTimeout(1000);
        await captureScreenshot(page, '31-admin-audit-logs');
    });
});

test.describe('Portal Screenshots - Login Flow Sequence', () => {
    test('40 - Full login sequence (NIP user)', async ({ page }) => {
        const interceptor = await setupCaptchaInterceptor(page);

        // Screenshot 1: Login page
        await page.goto(`${BASE_URL}${PORTAL_PATH}/login`, { waitUntil: 'domcontentloaded' });
        await expect(page.getByText('Masuk ke Sistem')).toBeVisible({ timeout: 30000 });
        await captureScreenshot(page, '40a-login-initial');

        // Open password form
        const passwordToggle = page.getByText('Atau masuk dengan password');
        if (await passwordToggle.isVisible({ timeout: 3000 }).catch(() => false)) {
            await passwordToggle.click();
        }

        // Fill credentials
        await page.locator('#username').fill(NIP_USER.username);
        await page.locator('#password').fill(NIP_USER.password);
        await captureScreenshot(page, '40b-login-filled-credentials');

        // Wait for captcha and solve
        await expect(page.locator('#captcha-title')).toBeVisible({ timeout: 15000 });
        await expect.poll(() => interceptor.getCaptchaAnswer(), { timeout: 10000 }).toBeTruthy();

        const captchaInput = page.locator('input[placeholder="Masukkan teks..."]');
        await captchaInput.fill(interceptor.getCaptchaAnswer()!);
        await captureScreenshot(page, '40c-login-captcha-filled');

        // Solve captcha
        const captchaSubmit = page.locator('.captcha-container button', { hasText: 'Kirim' });
        await Promise.all([
            page.waitForResponse(
                (resp) => resp.url().includes('/api/captcha/verify') && resp.request().method() === 'POST',
                { timeout: 15000 },
            ),
            captchaSubmit.click(),
        ]);
        await page.waitForTimeout(500);
        await captureScreenshot(page, '40d-login-captcha-solved');

        // Click login
        const loginButton = page.getByRole('button', { name: /Masuk ke Portal/i });
        await expect(loginButton).toBeEnabled({ timeout: 5000 });

        await Promise.all([
            page.waitForResponse(
                (resp) => resp.url().includes('/login') && resp.request().method() === 'POST',
                { timeout: 15000 },
            ),
            loginButton.click(),
        ]);

        // Dashboard after login
        await page.waitForURL(/\/(dashboard|portal\/dashboard)/, { timeout: 15000 });
        await page.waitForTimeout(1000);
        await captureScreenshot(page, '40e-login-success-dashboard');
    });
});

test.describe('Portal - Authenc Integration Verification', () => {
    test('50 - Authenc health check', async ({ request }) => {
        const resp = await request.get(`${AUTHENC_URL}/health`);
        expect(resp.ok()).toBeTruthy();
    });

    test('51 - Captcha challenge/verify flow', async ({ request }) => {
        // Get challenge
        const challengeResp = await request.post(`${AUTHENC_URL}/api/captcha/challenge`, {
            data: { challenge_type: 'Visual', difficulty: 3 },
        });
        expect(challengeResp.ok()).toBeTruthy();
        const challenge = await challengeResp.json();
        expect(challenge.challenge_id).toBeTruthy();

        // Extract answer via debug endpoint
        const answer = await fetchCaptchaDebugAnswer(challenge.challenge_id);
        expect(answer).toBeTruthy();

        // Verify
        const verifyResp = await request.post(`${AUTHENC_URL}/api/captcha/verify`, {
            data: {
                challenge_id: challenge.challenge_id,
                answer: answer!,
                session_id: `screenshot_test_${Date.now()}`,
            },
        });
        expect(verifyResp.ok()).toBeTruthy();
        const result = await verifyResp.json();
        expect(result.success).toBe(true);
        expect(result.token).toMatch(/^captcha_/);
    });

    test('52 - Full auth API flow (login + token)', async ({ request }) => {
        // Captcha
        const challengeResp = await request.post(`${AUTHENC_URL}/api/captcha/challenge`, {
            data: { challenge_type: 'Visual', difficulty: 3 },
        });
        const challenge = await challengeResp.json();
        const answer = await fetchCaptchaDebugAnswer(challenge.challenge_id);

        const verifyResp = await request.post(`${AUTHENC_URL}/api/captcha/verify`, {
            data: {
                challenge_id: challenge.challenge_id,
                answer: answer!,
                session_id: `api_flow_${Date.now()}`,
            },
        });
        const verifyResult = await verifyResp.json();

        // Login
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
        expect(loginResult.access_token.split('.').length).toBe(3); // JWT format
    });

    test('53 - Auth rejects invalid credentials', async ({ request }) => {
        const challengeResp = await request.post(`${AUTHENC_URL}/api/captcha/challenge`, {
            data: { challenge_type: 'Visual', difficulty: 3 },
        });
        const challenge = await challengeResp.json();
        const answer = await fetchCaptchaDebugAnswer(challenge.challenge_id);

        const verifyResp = await request.post(`${AUTHENC_URL}/api/captcha/verify`, {
            data: {
                challenge_id: challenge.challenge_id,
                answer: answer!,
                session_id: `invalid_test_${Date.now()}`,
            },
        });
        const verifyResult = await verifyResp.json();

        const loginResp = await request.post(`${AUTHENC_URL}/api/v1/auth/login`, {
            data: {
                username: 'nonexistent_user',
                password: 'wrong_password',
                realm: 'master',
                captcha_token: verifyResult.token,
            },
        });
        expect(loginResp.status()).toBe(401);
    });
});
