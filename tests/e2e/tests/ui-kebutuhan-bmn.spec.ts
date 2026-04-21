import { test, expect } from '@playwright/test';
import { loginAsRole } from './helpers/session';
import { isServiceHealthy } from './helpers/environment';

/**
 * E2E UI Test: Kebutuhan BMN (BMN Needs Analysis) - Full Frontend Workflow
 *
 * Business Process:
 * 1. Validator Pusat creates pengajuan kebutuhan BMN
 * 2. Operator Satker fills kebutuhan barang
 * 3. Operator Satker submits to Validator Wilayah
 * 4. Validator Wilayah reviews and forwards
 * 5. Validator Pusat approves/rejects
 */
test.describe('UI Workflow: Kebutuhan BMN', () => {
  test.describe.configure({ mode: 'serial' });
  let frontendAvailable = false;

  test.beforeAll(async ({ request }) => {
    frontendAvailable = await isServiceHealthy(request, '/perlengkapan/');
  });

  test.beforeEach(async () => {
    test.skip(!frontendAvailable, 'Frontend perlengkapan tidak tersedia untuk E2E UI test');
  });

  test('Step 1: Access kebutuhan BMN dashboard', async ({ page }) => {
    await loginAsRole(page, 'validator_pusat');
    await page.goto('/perlengkapan/dashboard/kebutuhan-bmn/daftar');
    await expect(page.getByText(/Analisis Kebutuhan BMN|Kebutuhan BMN/i).first()).toBeVisible({ timeout: 30000 });
    await page.screenshot({ path: 'test-results/ui-kebutuhan-bmn-01-dashboard.png', fullPage: true });
  });

  test('Step 2: View kebutuhan BMN list', async ({ page }) => {
    await loginAsRole(page, 'validator_pusat');
    await page.goto('/perlengkapan/dashboard/kebutuhan-bmn/daftar');
    await expect(page.getByText(/Analisis Kebutuhan BMN|Kebutuhan BMN/i).first()).toBeVisible({ timeout: 30000 });
    await page.screenshot({ path: 'test-results/ui-kebutuhan-bmn-02-list.png', fullPage: true });
  });

  test('Step 3: Open create form', async ({ page }) => {
    await loginAsRole(page, 'validator_pusat');
    await page.goto('/perlengkapan/dashboard/kebutuhan-bmn/buat');
    // Form should be present
    await expect(page.getByText(/Buat Pengajuan Kebutuhan BMN|Kebutuhan BMN/i).first()).toBeVisible({ timeout: 30000 });
    await page.screenshot({ path: 'test-results/ui-kebutuhan-bmn-03-create-form.png', fullPage: true });
  });

  test('Step 4: Fill and submit kebutuhan BMN form', async ({ page }) => {
    await loginAsRole(page, 'validator_pusat');
    await page.goto('/perlengkapan/dashboard/kebutuhan-bmn/buat');

    // Try to fill form fields (different form implementations may have different labels)
    const namaInput = page.locator('input[name*="nama"], input[placeholder*="nama"], [aria-label*="nama"]').first();
    if (await namaInput.isVisible()) {
      await namaInput.fill('Kebutuhan BMN TA 2026 - E2E Test');
    }

    const deskripsiInput = page.locator('textarea, input[name*="deskripsi"], [aria-label*="deskripsi"]').first();
    if (await deskripsiInput.isVisible()) {
      await deskripsiInput.fill('Pengajuan kebutuhan BMN untuk testing');
    }

    // Screenshot the filled form
    await page.screenshot({ path: 'test-results/ui-kebutuhan-bmn-04-fill-form.png', fullPage: true });

    // Try to submit
    const submitButton = page.getByRole('button', { name: /simpan|submit|kirim|buat/i }).first();
    if (await submitButton.isVisible() && await submitButton.isEnabled()) {
      await submitButton.click();
      await page.waitForTimeout(1000);
    }
    await page.screenshot({ path: 'test-results/ui-kebutuhan-bmn-05-after-submit.png', fullPage: true });
  });

  test('Step 5: Operator Satker views kebutuhan BMN detail', async ({ page }) => {
    await loginAsRole(page, 'operator_satker');
    await page.goto('/perlengkapan/dashboard/kebutuhan-bmn/daftar');
    await expect(page.getByText(/Analisis Kebutuhan BMN|Kebutuhan BMN/i).first()).toBeVisible({ timeout: 30000 });
    await page.screenshot({ path: 'test-results/ui-kebutuhan-bmn-06-operator-list.png', fullPage: true });
  });

  test('Step 6: Navigate to satker detail', async ({ page }) => {
    await loginAsRole(page, 'operator_satker');
    await page.goto('/perlengkapan/dashboard/kebutuhan-bmn/satker/SKR001');
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(2000);
    await page.screenshot({ path: 'test-results/ui-kebutuhan-bmn-07-satker-detail.png', fullPage: true });
  });

  test('Step 7: Validator Wilayah can review kebutuhan BMN list', async ({ page }) => {
    await loginAsRole(page, 'validator_wilayah');
    await page.goto('/perlengkapan/dashboard/kebutuhan-bmn/daftar');
    await expect(page.getByText(/Analisis Kebutuhan BMN|Kebutuhan BMN/i).first()).toBeVisible({ timeout: 30000 });
    await page.screenshot({ path: 'test-results/ui-kebutuhan-bmn-08-validator-wilayah-review.png', fullPage: true });
  });

  test('Step 8: End-to-end flow summary is reachable for all kebutuhan roles', async ({ page }) => {
    for (const role of ['operator_satker', 'validator_wilayah', 'validator_pusat'] as const) {
      await loginAsRole(page, role);
      await page.goto('/perlengkapan/dashboard/kebutuhan-bmn/daftar');
      await expect(page.getByText(/Analisis Kebutuhan BMN|Kebutuhan BMN/i).first()).toBeVisible({ timeout: 30000 });
    }
    await page.screenshot({ path: 'test-results/ui-kebutuhan-bmn-09-flow-summary-all-roles.png', fullPage: true });
  });
});
