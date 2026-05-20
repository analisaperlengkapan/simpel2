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
    username: nip,
    role,
    access_token: 'mock-jwt-token-for-e2e',
  });
  await page.addInitScript(({ session, role }) => {
    localStorage.setItem('perlengkapan_user_session', session);
    localStorage.setItem('auth_token', 'mock-jwt-token-for-e2e');
    localStorage.setItem('active_role', role);
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
    await expect(page.getByText('Portal Perlengkapan Kejaksaan')).toBeVisible({ timeout: 30000 });
    await expect(page.getByRole('heading', { name: 'Ringkasan Sistem Manajemen' })).toBeVisible({ timeout: 30000 });
  });

  test('should display quick navigation menu items', async ({ page }) => {
    await page.waitForLoadState('networkidle');
    // Check current module cards in dashboard main area.
    await expect(
      page.locator('main').getByRole('link', { name: /Bank Aset Katalog dan registrasi BMN/i }),
    ).toBeVisible({ timeout: 30000 });
    await expect(
      page.locator('main').getByRole('link', { name: /Kebutuhan BMN Analisis kebutuhan dan perencanaan/i }),
    ).toBeVisible({ timeout: 30000 });
    await expect(
      page.locator('main').getByRole('link', { name: /Pakaian Dinas Pengajuan dan distribusi atribut/i }),
    ).toBeVisible({ timeout: 30000 });
    await expect(
      page.locator('main').getByRole('link', { name: /Pemakaian BMN Izin pemakaian dan monitoring/i }),
    ).toBeVisible({ timeout: 30000 });
    await expect(
      page.locator('main').getByRole('link', { name: /Penghapusan BMN Disposal dan penghapusan BMN/i }),
    ).toBeVisible({ timeout: 30000 });
  });

  test('should show admin panel for admin role', async ({ page }) => {
    await page.waitForLoadState('networkidle');
    await expect(page.getByText('Panel Administrator')).toBeVisible({ timeout: 30000 });
    await expect(page.getByText('Manajemen Pengguna')).toBeVisible({ timeout: 30000 });
    await expect(page.getByText('Otorisasi (RBAC)')).toBeVisible({ timeout: 30000 });
  });

  test('should hide admin panel for non-admin roles', async ({ page }) => {
    await setupAuthenticatedSession(page, { role: 'operator_satker' });
    await page.goto('/perlengkapan/dashboard');
    await page.waitForLoadState('networkidle');
    await expect(page.getByText('Panel Administrator')).not.toBeVisible({ timeout: 30000 });
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
    const pemakaianCard = page
      .locator('main')
      .getByRole('link', { name: /Pemakaian BMN Izin pemakaian dan monitoring/i });
    await pemakaianCard.scrollIntoViewIfNeeded();
    await pemakaianCard.click();
    await expect(page).toHaveURL(/\/dashboard\/pengelolaan\/pemakaian/);
  });

  test('should navigate to pakaian dinas', async ({ page }) => {
    await page.getByRole('link', { name: /Pakaian Dinas/i }).first().click();
    await expect(page).toHaveURL(/\/dashboard\/pakaian-dinas/);
  });

  test('should show header with app branding', async ({ page }) => {
    await page.waitForLoadState('networkidle');
    await expect(page.locator('header').getByText('SIMPEL')).toBeVisible({ timeout: 30000 });
    await expect(page.locator('header')).toBeVisible({ timeout: 30000 });
  });

  test('should show footer with copyright', async ({ page }) => {
    await page.waitForLoadState('networkidle');
    await expect(page.getByText(/SIMPEL v0\.1\.0 · Kejaksaan Agung RI/)).toBeVisible({ timeout: 30000 });
  });
});
