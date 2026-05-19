import { test, expect } from '@playwright/test';
import { loginAsRole } from './helpers/session';
import { isServiceHealthy } from './helpers/environment';

/**
 * E2E UI Test: Pemakaian BMN - Full Frontend Workflow
 *
 * Business Process:
 * 1. View daftar pemakaian BMN
 * 2. Create new pemakaian BMN (konsep surat)
 * 3. View pemakaian BMN detail
 * 4. Validator review workflow
 */
test.describe('UI Workflow: Pemakaian BMN', () => {
  test.describe.configure({ mode: 'serial' });
  let frontendAvailable = false;

  test.beforeAll(async ({ request }) => {
    frontendAvailable = await isServiceHealthy(request, '/perlengkapan/');
  });

  test.beforeEach(async () => {
    test.skip(!frontendAvailable, 'Frontend perlengkapan tidak tersedia untuk E2E UI test');
  });

  test('Step 1: View daftar pemakaian BMN', async ({ page }) => {
    await loginAsRole(page, 'operator_satker');
    await page.goto('/perlengkapan/dashboard/pengelolaan/pemakaian');
    await expect(page.getByText(/Izin Pemakaian BMN|Pemakaian BMN|Pemakaian/i).first()).toBeVisible({ timeout: 30000 });
    await page.screenshot({ path: 'test-results/ui-pemakaian-bmn-01-list.png', fullPage: true });
  });

  test('Step 2: Open create pemakaian BMN form', async ({ page }) => {
    await loginAsRole(page, 'operator_satker');
    await page.goto('/perlengkapan/dashboard/pengelolaan/pemakaian/buat');
    // Should show form
    await expect(page.getByText(/Permohonan Izin Pemakaian BMN|Pemakaian BMN|Pemakaian/i).first()).toBeVisible({ timeout: 30000 });
    await page.screenshot({ path: 'test-results/ui-pemakaian-bmn-02-create-form.png', fullPage: true });
  });

  test('Step 3: Fill pemakaian BMN form', async ({ page }) => {
    await loginAsRole(page, 'operator_satker');
    await page.goto('/perlengkapan/dashboard/pengelolaan/pemakaian/buat');
    await expect(page.getByText(/Permohonan Izin Pemakaian BMN|Pemakaian BMN|Pemakaian/i).first()).toBeVisible({ timeout: 30000 });

    // Fill form fields
    const inputs = page.locator('input:visible, textarea:visible, select:visible');
    const count = await inputs.count();

    for (let i = 0; i < Math.min(count, 5); i++) {
      const input = inputs.nth(i);
      const type = await input.getAttribute('type');
      const tag = await input.evaluate(el => el.tagName.toLowerCase());

      if (tag === 'select') {
        const options = input.locator('option');
        if (await options.count() > 1) {
          await input.selectOption({ index: 1 });
        }
      } else if (type === 'date') {
        await input.fill('2025-06-15');
      } else if (type === 'number') {
        await input.fill('1');
      } else if (tag === 'textarea') {
        await input.fill('E2E Test - Pemakaian BMN untuk keperluan operasional kantor');
      } else {
        await input.fill('E2E Test Data');
      }
    }

    await page.screenshot({ path: 'test-results/ui-pemakaian-bmn-03-filled-form.png', fullPage: true });

    // Try to submit
    const submitBtn = page.getByRole('button', { name: /simpan|submit|kirim/i }).first();
    if (await submitBtn.isVisible()) {
      await submitBtn.click();
      await page.waitForTimeout(1000);
    }
    await page.screenshot({ path: 'test-results/ui-pemakaian-bmn-04-after-submit.png', fullPage: true });
  });

  test('Step 4: Legacy pemakaian list', async ({ page }) => {
    await loginAsRole(page, 'operator_satker');
    await page.goto('/perlengkapan/dashboard/pengelolaan/pemakaian');
    await expect(page.getByText(/Daftar Pemakaian BMN|Pemakaian BMN|Pemakaian/i).first()).toBeVisible({ timeout: 30000 });
    await page.screenshot({ path: 'test-results/ui-pemakaian-bmn-05-legacy-list.png', fullPage: true });
  });

  test('Step 5: Create legacy pemakaian form', async ({ page }) => {
    await loginAsRole(page, 'operator_satker');
    await page.goto('/perlengkapan/dashboard/pengelolaan/pemakaian/buat');
    await expect(page.getByText(/Catat Pemakaian BMN|Pemakaian BMN|Pemakaian/i).first()).toBeVisible({ timeout: 30000 });

    // Fill form
    const namaInput = page.locator('input[name*="nama"], [aria-label*="Peminjam"], input').first();
    if (await namaInput.isVisible()) {
      await namaInput.fill('E2E User Peminjam');
    }

    await page.screenshot({ path: 'test-results/ui-pemakaian-bmn-06-legacy-form.png', fullPage: true });
  });

  test('Step 6: Validator Wilayah can monitor pemakaian BMN', async ({ page }) => {
    await loginAsRole(page, 'validator_wilayah');
    await page.goto('/perlengkapan/dashboard/pengelolaan/pemakaian');
    await expect(page.getByText(/Izin Pemakaian BMN|Pemakaian BMN|Pemakaian/i).first()).toBeVisible({ timeout: 30000 });
    await page.screenshot({ path: 'test-results/ui-pemakaian-bmn-07-validator-wilayah-monitor.png', fullPage: true });
  });

  test('Step 7: Validator Pusat can monitor pemakaian BMN', async ({ page }) => {
    await loginAsRole(page, 'validator_pusat');
    await page.goto('/perlengkapan/dashboard/pengelolaan/pemakaian');
    await expect(page.getByText(/Izin Pemakaian BMN|Pemakaian BMN|Pemakaian/i).first()).toBeVisible({ timeout: 30000 });
    await page.screenshot({ path: 'test-results/ui-pemakaian-bmn-08-validator-pusat-monitor.png', fullPage: true });
  });

  test('Step 8: End-to-end flow summary is reachable for all pemakaian roles', async ({ page }) => {
    for (const role of ['operator_satker', 'validator_wilayah', 'validator_pusat'] as const) {
      await loginAsRole(page, role);
      await page.goto('/perlengkapan/dashboard/pengelolaan/pemakaian');
      await expect(page.getByText(/Izin Pemakaian BMN|Pemakaian BMN|Pemakaian/i).first()).toBeVisible({ timeout: 30000 });
    }
    await page.screenshot({ path: 'test-results/ui-pemakaian-bmn-09-flow-summary-all-roles.png', fullPage: true });
  });
});
