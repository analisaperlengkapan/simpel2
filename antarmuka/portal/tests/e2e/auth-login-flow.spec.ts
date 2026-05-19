import { test, expect } from '@playwright/test';

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
  test.beforeEach(async ({ page }) => {
    await page.addInitScript(() => {
      localStorage.clear();
      sessionStorage.clear();
    });
  });

  test('should show landing page with login button', async ({ page }) => {
    await page.goto('/perlengkapan/', { waitUntil: 'domcontentloaded' });
    await page.waitForURL(/\/perlengkapan\/login/, { timeout: 30000 });
    await expect(page.getByRole('heading', { name: /masuk untuk melanjutkan/i })).toBeVisible({
      timeout: 30000,
    });
    await expect(page.getByRole('link', { name: /masuk via portal|masuk/i })).toBeVisible();
  });

  test('should redirect to portal login on click', async ({ page }) => {
    await page.goto('/perlengkapan/', { waitUntil: 'domcontentloaded' });
    const loginLink = page.getByRole('link', { name: /masuk via portal|masuk/i });
    await loginLink.click();
    // Should redirect to portal login with redirect_uri target to perlengkapan dashboard.
    await expect(page).toHaveURL(/\/portal\/login\?redirect_uri=/);
    await expect(page).toHaveURL(/%2Fperlengkapan%2Fdashboard/);
  });

  test('should show dashboard after successful login', async ({ page }) => {
    // Set mock perlengkapan session via addInitScript so it is available
    // before app initialization.
    const session = JSON.stringify({
      username: '199203142014031001',
      role: 'admin',
      access_token: 'mock-jwt-token-for-e2e',
    });
    await page.addInitScript((session) => {
      localStorage.setItem('perlengkapan_user_session', session);
      localStorage.setItem('auth_token', 'mock-jwt-token-for-e2e');
      localStorage.setItem('active_role', 'admin');
    }, session);

    // Navigate to dashboard
    await page.goto('/perlengkapan/dashboard');
    await page.waitForLoadState('networkidle');
    await expect(page).toHaveURL(/\/perlengkapan\/dashboard/);
    await expect(page.locator('button[title="Profil"]')).toBeVisible({ timeout: 30000 });
    await expect(page.getByText('SIMPEL Perlengkapan')).not.toBeVisible();
  });

  test('should show role switcher in dashboard header', async ({ page }) => {
    const session = JSON.stringify({
      username: '199203142014031001',
      role: 'admin',
      access_token: 'mock-jwt-token-for-e2e',
    });
    await page.addInitScript((session) => {
      localStorage.setItem('perlengkapan_user_session', session);
      localStorage.setItem('auth_token', 'mock-jwt-token-for-e2e');
      localStorage.setItem('active_role', 'admin');
    }, session);

    await page.goto('/perlengkapan/dashboard');
    await page.waitForLoadState('networkidle');
    // Open profile dropdown first, then role switcher content should appear.
    await page.locator('button[title="Profil"]').click();
    await expect(page.getByText('Ganti Role')).toBeVisible({ timeout: 30000 });
    await expect(
      page
        .locator('div:has-text("Ganti Role")')
        .getByRole('button', { name: /admin/i })
        .first(),
    ).toBeVisible({ timeout: 30000 });
  });

  test('should logout and redirect to landing page', async ({ page }) => {
    const session = JSON.stringify({
      username: '199203142014031001',
      role: 'operator_satker',
      access_token: 'mock-jwt-token-for-e2e',
    });
    await page.addInitScript((session) => {
      localStorage.setItem('perlengkapan_user_session', session);
      localStorage.setItem('auth_token', 'mock-jwt-token-for-e2e');
      localStorage.setItem('active_role', 'operator_satker');
    }, session);

    await page.goto('/perlengkapan/dashboard');
    await page.waitForLoadState('networkidle');
    await expect(page.locator('button[title="Profil"]')).toBeVisible({ timeout: 30000 });

    // Click logout
    await page.locator('button[title="Profil"]').click();
    await page.getByRole('button', { name: /keluar/i }).click();

    // Logout clears session and redirects to perlengkapan login page.
    await page.waitForURL(/\/perlengkapan\/login/, { timeout: 30000 });
    await expect(page).toHaveURL(/\/perlengkapan\/login/, { timeout: 30000 });
  });
});
