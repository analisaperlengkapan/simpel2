import { test, expect, Page } from '@playwright/test';

async function loginAs(page: Page, role: string) {
  const mockSession = JSON.stringify({
    id: '20000000-0000-0000-0000-000000000001',
    username: '199203142014031001',
    name: 'User E2E',
    email: 'e2e@kejaksaan.go.id',
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
  // Use addInitScript so localStorage is set BEFORE any page scripts run
  await page.addInitScript(({ session, role }) => {
    localStorage.setItem('user_session', session);
    localStorage.setItem('auth_token', 'mock-jwt-token-for-e2e');
    localStorage.setItem('active_role', role);
  }, { session: mockSession, role });
}

/**
 * E2E UI Test: Bank Aset, Mutasi, Hibah, Pemeliharaan, Pengalihan
 *
 * Tests remaining business workflows:
 * - Bank Aset: View & search assets
 * - Mutasi BMN: Transfer assets between satker
 * - Hibah BMN: Grant management
 * - Pemeliharaan: Maintenance tracking
 * - Pengalihan: Asset transfer/disposal
 */
test.describe('UI Workflow: Bank Aset & Pengelolaan BMN', () => {
  // ===== Bank Aset =====
  test.describe('Bank Aset', () => {
    test('should display asset list', async ({ page }) => {
      await loginAs(page, 'operator_satker');
      await page.goto('/perlengkapan/dashboard/bank-aset/daftar');
      await expect(page.getByText('Daftar Aset')).toBeVisible({ timeout: 30000 });
      await page.screenshot({ path: 'test-results/ui-bank-aset-01-list.png', fullPage: true });
    });
  });

  // ===== Mutasi BMN =====
  test.describe('Mutasi BMN', () => {
    test('should display mutasi list', async ({ page }) => {
      await loginAs(page, 'operator_satker');
      await page.goto('/perlengkapan/dashboard/pengelolaan/mutasi/daftar');
      await expect(page.getByText('Daftar Mutasi BMN')).toBeVisible({ timeout: 30000 });
      await page.screenshot({ path: 'test-results/ui-mutasi-01-list.png', fullPage: true });
    });

    test('should open create mutasi form', async ({ page }) => {
      await loginAs(page, 'operator_satker');
      await page.goto('/perlengkapan/dashboard/pengelolaan/mutasi/baru');
      await expect(page.getByText('Catat Mutasi BMN')).toBeVisible({ timeout: 30000 });
      await page.screenshot({ path: 'test-results/ui-mutasi-02-create-form.png', fullPage: true });
    });

    test('should fill mutasi form', async ({ page }) => {
      await loginAs(page, 'operator_satker');
      await page.goto('/perlengkapan/dashboard/pengelolaan/mutasi/baru');
      await expect(page.getByText('Catat Mutasi BMN')).toBeVisible({ timeout: 30000 });

      const inputs = page.locator('input:visible, textarea:visible').first();
      if (await inputs.isVisible()) {
        await inputs.fill('E2E Mutasi Test');
      }

      await page.screenshot({ path: 'test-results/ui-mutasi-03-filled-form.png', fullPage: true });
    });
  });

  // ===== Hibah BMN =====
  test.describe('Hibah BMN', () => {
    test('should display hibah list', async ({ page }) => {
      await loginAs(page, 'operator_satker');
      await page.goto('/perlengkapan/dashboard/pengelolaan/hibah/daftar');
      await expect(page.getByText('Daftar Hibah BMN')).toBeVisible({ timeout: 30000 });
      await page.screenshot({ path: 'test-results/ui-hibah-01-list.png', fullPage: true });
    });

    test('should open create hibah form', async ({ page }) => {
      await loginAs(page, 'operator_satker');
      await page.goto('/perlengkapan/dashboard/pengelolaan/hibah/baru');
      await expect(page.getByText('Catat Hibah BMN')).toBeVisible({ timeout: 30000 });
      await page.screenshot({ path: 'test-results/ui-hibah-02-create-form.png', fullPage: true });
    });

    test('should fill hibah form', async ({ page }) => {
      await loginAs(page, 'operator_satker');
      await page.goto('/perlengkapan/dashboard/pengelolaan/hibah/baru');
      await expect(page.getByText('Catat Hibah BMN')).toBeVisible({ timeout: 30000 });

      const inputs = page.locator('input:visible, textarea:visible');
      const count = await inputs.count();
      for (let i = 0; i < Math.min(count, 4); i++) {
        const input = inputs.nth(i);
        const tag = await input.evaluate(el => el.tagName.toLowerCase());
        const type = await input.getAttribute('type');
        if (tag === 'textarea') {
          await input.fill('E2E Test - Hibah BMN untuk instansi pemerintah');
        } else if (type === 'date') {
          await input.fill('2025-06-15');
        } else if (type === 'number') {
          await input.fill('1');
        } else {
          await input.fill('E2E Hibah Data');
        }
      }

      await page.screenshot({ path: 'test-results/ui-hibah-03-filled-form.png', fullPage: true });
    });
  });

  // ===== Pemeliharaan =====
  test.describe('Pemeliharaan', () => {
    test('should display pemeliharaan list', async ({ page }) => {
      await loginAs(page, 'operator_satker');
      await page.goto('/perlengkapan/dashboard/pemeliharaan/daftar');
      await expect(page.getByText('Daftar Pemeliharaan Aset')).toBeVisible({ timeout: 30000 });
      await page.screenshot({ path: 'test-results/ui-pemeliharaan-01-list.png', fullPage: true });
    });

    test('should open create pemeliharaan form', async ({ page }) => {
      await loginAs(page, 'operator_satker');
      await page.goto('/perlengkapan/dashboard/pemeliharaan/baru');
      await expect(page.getByText('Catat Pemeliharaan Aset')).toBeVisible({ timeout: 30000 });
      await page.screenshot({ path: 'test-results/ui-pemeliharaan-02-create-form.png', fullPage: true });
    });
  });

  // ===== Pengalihan =====
  test.describe('Pengalihan BMN', () => {
    test('should display pengalihan list', async ({ page }) => {
      await loginAs(page, 'operator_satker');
      await page.goto('/perlengkapan/dashboard/pengelolaan/pengalihan/daftar');
      await expect(page.getByText('Daftar Pengalihan Aset')).toBeVisible({ timeout: 30000 });
      await page.screenshot({ path: 'test-results/ui-pengalihan-01-list.png', fullPage: true });
    });

    test('should open create pengalihan form', async ({ page }) => {
      await loginAs(page, 'operator_satker');
      await page.goto('/perlengkapan/dashboard/pengelolaan/pengalihan/baru');
      await expect(page.getByText('Form Pengalihan Aset')).toBeVisible({ timeout: 30000 });
      await page.screenshot({ path: 'test-results/ui-pengalihan-02-create-form.png', fullPage: true });
    });
  });

  // ===== Analisis Kebutuhan =====
  test.describe('Analisis Kebutuhan', () => {
    test('should display analisis list', async ({ page }) => {
      await loginAs(page, 'operator_satker');
      await page.goto('/perlengkapan/dashboard/analisis/daftar');
      await expect(page.getByText('Buat Analisis Baru')).toBeVisible({ timeout: 30000 });
      await page.screenshot({ path: 'test-results/ui-analisis-01-list.png', fullPage: true });
    });

    test('should open create analisis form', async ({ page }) => {
      await loginAs(page, 'operator_satker');
      await page.goto('/perlengkapan/dashboard/analisis/baru');
      await expect(page.getByText('Buat Analisis Kebutuhan')).toBeVisible({ timeout: 30000 });
      await page.screenshot({ path: 'test-results/ui-analisis-02-create-form.png', fullPage: true });
    });
  });
});
