import { test, expect } from '@playwright/test';
import { loginAsRole } from './helpers/session';
import { isServiceHealthy } from './helpers/environment';

/**
 * E2E UI Test: Bank Aset & Dashboard Pengelolaan
 *
 * Focused on routes that are currently registered in frontend router.
 */
test.describe('UI Workflow: Bank Aset & Pengelolaan BMN', () => {
  test.describe.configure({ mode: 'serial' });
  let frontendAvailable = false;

  test.beforeAll(async ({ request }) => {
    frontendAvailable = await isServiceHealthy(request, '/perlengkapan/');
  });

  test.beforeEach(async () => {
    test.skip(!frontendAvailable, 'Frontend perlengkapan tidak tersedia untuk E2E UI test');
  });

  test('Step 1: Operator can open bank aset daftar', async ({ page }) => {
    await loginAsRole(page, 'operator_satker');
    await page.goto('/perlengkapan/dashboard/bank-aset/daftar');
    await expect(page.getByText(/Bank Aset|Daftar Aset|Aset BMN/i).first()).toBeVisible({ timeout: 30000 });
    await page.screenshot({ path: 'test-results/ui-bank-aset-01-daftar.png', fullPage: true });
  });

  test('Step 2: Operator can open bank aset QR route', async ({ page }) => {
    await loginAsRole(page, 'operator_satker');
    await page.goto('/perlengkapan/dashboard/bank-aset/qrcode');
    await expect(page.getByText(/QR|QRCode|Bank Aset|Aset BMN/i).first()).toBeVisible({ timeout: 30000 });
    await page.screenshot({ path: 'test-results/ui-bank-aset-02-qrcode.png', fullPage: true });
  });

  test('Step 3: Cross-role access to bank aset daftar', async ({ page }) => {
    for (const role of ['operator_satker', 'validator_wilayah', 'validator_pusat'] as const) {
      await loginAsRole(page, role);
      await page.goto('/perlengkapan/dashboard/bank-aset/daftar');
      await expect(page.getByText(/Bank Aset|Daftar Aset|Aset BMN/i).first()).toBeVisible({ timeout: 30000 });
    }
    await page.screenshot({ path: 'test-results/ui-bank-aset-03-cross-role.png', fullPage: true });
  });

  test('Step 4: Role access for pengelolaan pemakaian', async ({ page }) => {
    for (const role of ['operator_satker', 'validator_wilayah', 'validator_pusat'] as const) {
      await loginAsRole(page, role);
      await page.goto('/perlengkapan/dashboard/pengelolaan/pemakaian');
      await expect(page.getByText(/Pemakaian BMN|Pemakaian/i).first()).toBeVisible({ timeout: 30000 });
    }
    await page.screenshot({ path: 'test-results/ui-pengelolaan-04-pemakaian-roles.png', fullPage: true });
  });

  test('Step 5: Role access for pengelolaan penghapusan', async ({ page }) => {
    for (const role of ['operator_satker', 'validator_wilayah', 'validator_pusat'] as const) {
      await loginAsRole(page, role);
      await page.goto('/perlengkapan/dashboard/pengelolaan/penghapusan');
      await expect(page.getByText(/Penghapusan BMN|Penghapusan/i).first()).toBeVisible({ timeout: 30000 });
    }
    await page.screenshot({ path: 'test-results/ui-pengelolaan-05-penghapusan-roles.png', fullPage: true });
  });

  test('Step 6: Analyst routes are reachable by validator roles', async ({ page }) => {
    for (const role of ['validator_wilayah', 'validator_pusat'] as const) {
      await loginAsRole(page, role);
      await page.goto('/perlengkapan/dashboard/analitik/roadmap');
      await expect(page.getByText(/Roadmap|Analitik/i).first()).toBeVisible({ timeout: 30000 });

      await page.goto('/perlengkapan/dashboard/analitik/kodefikasi');
      await expect(page.getByText(/Kodefikasi|Analitik/i).first()).toBeVisible({ timeout: 30000 });
    }
    await page.screenshot({ path: 'test-results/ui-analitik-06-validator-routes.png', fullPage: true });
  });
});
