import { test, expect, Page } from '@playwright/test';

/**
 * E2E Test Helper: Authenticated Page Setup
 *
 * Provides reusable helper to set up authenticated session in localStorage
 * for all perlengkapan UI E2E tests.
 */

interface AuthSetupOptions {
  role?: string;
  nip?: string;
  name?: string;
}

export async function setupAuthenticatedSession(page: Page, options: AuthSetupOptions = {}) {
  const {
    role = 'admin',
    nip = '199203142014031001',
    name = 'User E2E Test',
  } = options;

  const mockSession = JSON.stringify({
    id: '20000000-0000-0000-0000-000000000001',
    username: nip,
    name,
    email: `${nip}@kejaksaan.go.id`,
    role: { Custom: role },
    avatar: null,
    division: 'Biro Perlengkapan',
    captcha_validated: true,
    mfa_enabled: false,
    mfa_setup_required: false,
    created_at: new Date().toISOString(),
    access_token: 'mock-jwt-token',
    refresh_token: 'mock-refresh',
    expires_at: Math.floor(Date.now() / 1000) + 86400,
    permissions: role === 'admin' ? ['*'] : [],
  });
  await page.addInitScript(({ session, role }) => {
    localStorage.setItem('user_session', session);
    localStorage.setItem('auth_token', 'mock-jwt-token-for-e2e');
    localStorage.setItem('active_role', role);
    localStorage.setItem(
      'available_roles',
      JSON.stringify([
        'operator_satker',
        'validator_wilayah',
        'validator_pusat',
        'admin',
      ]),
    );
  }, { session: mockSession, role });
}

/**
 * E2E Test: Dashboard Navigation & Quick Actions
 *
 * Verifies the dashboard loads correctly with:
 * - Stats cards (Total Aset, Kondisi Baik, Kondisi Rusak, Total Satker)
 * - Quick navigation menu
 * - Role-aware admin panel section
 */
test.describe('Dashboard & Navigation', () => {
  test.beforeEach(async ({ page }) => {
    await setupAuthenticatedSession(page, { role: 'admin' });
    await page.goto('/perlengkapan/dashboard');
  });

  test('should display dashboard with welcome banner', async ({ page }) => {
    await page.waitForLoadState('networkidle');
    await expect(page.getByText('Dashboard Perlengkapan')).toBeVisible({ timeout: 30000 });
    await expect(page.getByText('Sistem Informasi Manajemen Perlengkapan')).toBeVisible({ timeout: 30000 });
  });

  test('should display quick navigation menu items', async ({ page }) => {
    await page.waitForLoadState('networkidle');
    // Check main menu items
    await expect(page.getByText('Bank Aset')).toBeVisible({ timeout: 30000 });
    await expect(page.getByText('Kebutuhan BMN')).toBeVisible({ timeout: 30000 });
    await expect(page.getByText('Pemakaian BMN')).toBeVisible({ timeout: 30000 });
    await expect(page.getByText('Pakaian Dinas')).toBeVisible({ timeout: 30000 });
    await expect(page.getByText('Penghapusan BMN')).toBeVisible({ timeout: 30000 });
    await expect(page.getByText('Mutasi BMN')).toBeVisible({ timeout: 30000 });
    await expect(page.getByRole('heading', { name: 'Pemeliharaan' })).toBeVisible({ timeout: 30000 });
    await expect(page.getByText('Hibah BMN')).toBeVisible({ timeout: 30000 });
  });

  test('should show admin panel for admin role', async ({ page }) => {
    await page.waitForLoadState('networkidle');
    await expect(page.getByText('Panel Admin')).toBeVisible({ timeout: 30000 });
    await expect(page.getByText('Manajemen User')).toBeVisible({ timeout: 30000 });
    await expect(page.getByText('Manajemen Role')).toBeVisible({ timeout: 30000 });
  });

  test('should hide admin panel for non-admin roles', async ({ page }) => {
    await setupAuthenticatedSession(page, { role: 'operator_satker' });
    await page.goto('/perlengkapan/dashboard');
    await page.waitForLoadState('networkidle');
    await expect(page.getByText('Panel Admin')).not.toBeVisible({ timeout: 30000 });
  });

  test('should navigate to bank aset', async ({ page }) => {
    await page.getByRole('link', { name: /Bank Aset/i }).first().click();
    await expect(page).toHaveURL(/\/dashboard\/bank-aset/);
  });

  test('should navigate to kebutuhan BMN', async ({ page }) => {
    await page.getByRole('link', { name: /Kebutuhan BMN/i }).first().click();
    await expect(page).toHaveURL(/\/dashboard\/kebutuhan-bmn/);
  });

  test('should navigate to pemakaian BMN', async ({ page }) => {
    await page.getByRole('link', { name: /Pemakaian BMN/i }).first().click();
    await expect(page).toHaveURL(/\/dashboard\/pemakaian-bmn/);
  });

  test('should navigate to pakaian dinas', async ({ page }) => {
    await page.getByRole('link', { name: /Pakaian Dinas/i }).first().click();
    await expect(page).toHaveURL(/\/dashboard\/pakaian-dinas/);
  });

  test('should show header with app branding', async ({ page }) => {
    await page.waitForLoadState('networkidle');
    await expect(page.getByText('SIMPEL')).toBeVisible({ timeout: 30000 });
    await expect(page.locator('header')).toBeVisible({ timeout: 30000 });
  });

  test('should show footer with copyright', async ({ page }) => {
    await page.waitForLoadState('networkidle');
    await expect(page.getByText('Kejaksaan Republik Indonesia')).toBeVisible({ timeout: 30000 });
  });
});
