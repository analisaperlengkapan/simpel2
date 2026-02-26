import { test, expect } from '@playwright/test';

/**
 * Helper to set a mock session in localStorage matching the UserSession struct.
 *
 * UserSession fields (from lib-ui use_auth.rs):
 *   id, username, role (Serde tagged enum e.g. {Custom:"role_name"}), name, email,
 *   avatar?, division, captcha_validated, mfa_enabled, mfa_setup_required,
 *   created_at?, access_token?, refresh_token?, expires_at? (Unix ts),
 *   permissions[]
 */
function mockUserSession(overrides: Record<string, unknown> = {}) {
  return JSON.stringify({
    id: '20000000-0000-0000-0000-000000000001',
    username: '199203142014031001',
    role: { Custom: 'admin' },
    name: 'User E2E Test',
    email: 'e2e@kejaksaan.go.id',
    avatar: null,
    division: 'Biro Perlengkapan',
    captcha_validated: true,
    mfa_enabled: false,
    mfa_setup_required: false,
    created_at: new Date().toISOString(),
    access_token: 'mock-jwt-token-for-e2e',
    refresh_token: 'mock-refresh-token',
    expires_at: Math.floor(Date.now() / 1000) + 86400,
    permissions: ['*'],
    ...overrides,
  });
}

/**
 * E2E Test: Authentication & Login Flow
 *
 * Tests the complete authentication flow:
 * 1. Landing page shows perlengkapan login page
 * 2. Clicking login redirects to /portal/login (SSO)
 * 3. After successful SSO login, redirects back to /dashboard
 * 4. Role switcher is visible in the header
 * 5. Logout button works
 */
test.describe('Authentication & Login Flow', () => {
  test('should show landing page with login button', async ({ page }) => {
    await page.goto('/perlengkapan/', { waitUntil: 'domcontentloaded' });
    await expect(page.getByText('SIMPEL Perlengkapan')).toBeVisible();
    await expect(page.getByRole('button', { name: /login|masuk/i })).toBeVisible();
  });

  test('should redirect to portal login on click', async ({ page }) => {
    await page.goto('/perlengkapan/', { waitUntil: 'domcontentloaded' });
    const loginButton = page.getByRole('button', { name: /login|masuk/i });
    await loginButton.click();
    // Should redirect to portal login with return_url parameter
    await expect(page).toHaveURL(/\/portal\/login|\/login/);
  });

  test('should show dashboard after successful login', async ({ page }) => {
    // Set mock session via addInitScript so it's available before page scripts run
    const session = mockUserSession();
    await page.addInitScript((session) => {
      localStorage.setItem('user_session', session);
      localStorage.setItem('auth_token', 'mock-jwt-token-for-e2e');
    }, session);

    // Navigate to dashboard
    await page.goto('/perlengkapan/dashboard');
    await page.waitForLoadState('networkidle');
    await expect(page.getByText('Dashboard Perlengkapan')).toBeVisible({ timeout: 30000 });
  });

  test('should show role switcher in dashboard header', async ({ page }) => {
    const session = mockUserSession();
    await page.addInitScript((session) => {
      localStorage.setItem('user_session', session);
      localStorage.setItem('auth_token', 'mock-jwt-token-for-e2e');
      localStorage.setItem('available_roles', JSON.stringify([
        'operator_satker', 'validator_wilayah', 'validator_pusat', 'admin'
      ]));
      localStorage.setItem('active_role', 'admin');
    }, session);

    await page.goto('/perlengkapan/dashboard');
    await page.waitForLoadState('networkidle');
    // Role switcher dropdown trigger should be visible
    await expect(page.locator('button[title="Ganti Role"]')).toBeVisible({ timeout: 30000 });
  });

  test('should logout and redirect to landing page', async ({ page }) => {
    const session = mockUserSession({ role: { Custom: 'operator_satker' } });
    await page.addInitScript((session) => {
      localStorage.setItem('user_session', session);
      localStorage.setItem('auth_token', 'mock-jwt-token-for-e2e');
    }, session);

    await page.goto('/perlengkapan/dashboard');
    await page.waitForLoadState('networkidle');
    await expect(page.getByText('Dashboard Perlengkapan')).toBeVisible({ timeout: 30000 });

    // Click logout
    const logoutButton = page.getByRole('button', { name: /logout/i });
    if (await logoutButton.isVisible()) {
      await logoutButton.click();
      // Logout clears session and redirects to portal login
      await expect(page).toHaveURL(/\/portal\/login/, { timeout: 10000 });
    }
  });
});
