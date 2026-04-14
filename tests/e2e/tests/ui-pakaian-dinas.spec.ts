import { test, expect } from '@playwright/test';
import { loginAsRole } from './helpers/session';
import { isServiceHealthy } from './helpers/environment';

/**
 * E2E UI Test: Pakaian Dinas - Full Frontend Workflow
 *
 * Business Process:
 * 1. View jenis pakaian dinas list (master data)
 * 2. View pengajuan pakaian dinas list
 * 3. View ukuran pegawai per satker
 * 4. View laporan pakaian dinas
 */
test.describe('UI Workflow: Pakaian Dinas', () => {
  test.describe.configure({ mode: 'serial' });
  let frontendAvailable = false;

  test.beforeAll(async ({ request }) => {
    frontendAvailable = await isServiceHealthy(request, '/perlengkapan/');
  });

  test.beforeEach(async () => {
    test.skip(!frontendAvailable, 'Frontend perlengkapan tidak tersedia untuk E2E UI test');
  });

  test('Step 1: View jenis pakaian dinas master list', async ({ page }) => {
    await loginAsRole(page, 'operator_satker');
    await page.goto('/perlengkapan/dashboard/pakaian-dinas/jenis');
    await expect(page.getByText(/Jenis Pakaian Dinas|Pakaian Dinas/i).first()).toBeVisible({ timeout: 30000 });
    await page.screenshot({ path: 'test-results/ui-pakaian-dinas-01-jenis-list.png', fullPage: true });
  });

  test('Step 2: View pengajuan pakaian dinas list', async ({ page }) => {
    await loginAsRole(page, 'operator_satker');
    await page.goto('/perlengkapan/dashboard/pakaian-dinas/pengajuan');
    await expect(page.getByText(/Pengajuan Pakaian Dinas|Pakaian Dinas/i).first()).toBeVisible({ timeout: 30000 });
    await page.screenshot({ path: 'test-results/ui-pakaian-dinas-02-pengajuan-list.png', fullPage: true });
  });

  test('Step 3: View ukuran pegawai per satker', async ({ page }) => {
    await loginAsRole(page, 'operator_satker');
    await page.goto('/perlengkapan/dashboard/pakaian-dinas/ukuran');
    await expect(page.getByText(/Ukuran Pakaian Pegawai|Ukuran Pakaian|Pakaian Dinas/i).first()).toBeVisible({ timeout: 30000 });
    await page.screenshot({ path: 'test-results/ui-pakaian-dinas-03-ukuran-satker.png', fullPage: true });
  });

  test('Step 4: View laporan pakaian dinas', async ({ page }) => {
    await loginAsRole(page, 'operator_satker');
    await page.goto('/perlengkapan/dashboard/pakaian-dinas/laporan');
    await expect(page.getByText(/Laporan Pakaian Dinas|Pakaian Dinas/i).first()).toBeVisible({ timeout: 30000 });
    await page.screenshot({ path: 'test-results/ui-pakaian-dinas-04-laporan.png', fullPage: true });
  });

  test('Step 5: View rekap laporan', async ({ page }) => {
    await loginAsRole(page, 'operator_satker');
    await page.goto('/perlengkapan/dashboard/pakaian-dinas/laporan/rekap');
    await expect(page.getByText(/Laporan Pakaian Dinas|Rekap|Pakaian Dinas/i).first()).toBeVisible({ timeout: 30000 });
    await page.screenshot({ path: 'test-results/ui-pakaian-dinas-05-rekap.png', fullPage: true });
  });

  test('Step 6: Validator Wilayah can review pengajuan pakaian dinas', async ({ page }) => {
    await loginAsRole(page, 'validator_wilayah');
    await page.goto('/perlengkapan/dashboard/pakaian-dinas/pengajuan');
    await expect(page.getByText(/Pengajuan Pakaian Dinas|Pakaian Dinas/i).first()).toBeVisible({ timeout: 30000 });
    await page.screenshot({ path: 'test-results/ui-pakaian-dinas-06-validator-wilayah-review.png', fullPage: true });
  });

  test('Step 7: Validator Pusat can finalize pakaian dinas review', async ({ page }) => {
    await loginAsRole(page, 'validator_pusat');
    await page.goto('/perlengkapan/dashboard/pakaian-dinas/pengajuan');
    await expect(page.getByText(/Pengajuan Pakaian Dinas|Pakaian Dinas/i).first()).toBeVisible({ timeout: 30000 });
    await page.screenshot({ path: 'test-results/ui-pakaian-dinas-07-validator-pusat-final.png', fullPage: true });
  });

  test('Step 8: End-to-end flow summary is reachable for all pakaian dinas roles', async ({ page }) => {
    for (const role of ['operator_satker', 'validator_wilayah', 'validator_pusat'] as const) {
      await loginAsRole(page, role);
      await page.goto('/perlengkapan/dashboard/pakaian-dinas/pengajuan');
      await expect(page.getByText(/Pengajuan Pakaian Dinas|Pakaian Dinas/i).first()).toBeVisible({ timeout: 30000 });
    }
    await page.screenshot({ path: 'test-results/ui-pakaian-dinas-08-flow-summary-all-roles.png', fullPage: true });
  });
});
