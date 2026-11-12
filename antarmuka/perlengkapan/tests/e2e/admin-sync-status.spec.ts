/**
 * The admin "Status Sinkronisasi" page must show what integrasi's scheduler recorded.
 *
 * Every form that fills a person in from a NIP, or lets the operator pick an asset, reads a copy
 * of MySIMKARI and SIMAN that integrasi's CronJobs refresh. Until 2026-11-12 no run left its
 * outcome anywhere an administrator could read: `integrasi.sync_status` existed and nothing wrote
 * it. Every run now does, and this page shows it through `GET /admin/status-sinkronisasi`.
 *
 * The page is compared with that API answer row by row (judgement, row count, reason), so a page
 * that drops a row, shows another state, or loses the reason for a failure fails here. The rows
 * themselves are whatever the environment's runs left (on staging, the mock URLs make every run
 * fail, which is a reason the page must show), so the spec asserts agreement, not a fixed state.
 * Reached by clicking the sidebar, not by URL alone.
 */
import { test, expect, type Page } from '@playwright/test';
import { storageStatePath, tokenFor, PERLENGKAPAN_API_URL } from './helpers/real-auth';

const BASE = '/perlengkapan/simpel/v2';
const API = `${PERLENGKAPAN_API_URL}/api/v1/perlengkapan`;
const PAGE = `${BASE}/admin/status-sinkronisasi`;
const FORBIDDEN = /Akses Ditolak/i;
const shell = (page: Page) => page.locator('header').getByText('SIMPEL').first();

interface SyncStatusEntry {
  sumber: string;
  bagian: string;
  keadaan: string;
  jumlah_data: number;
  galat: string | null;
}

test.describe('Admin sees the sync status the scheduler recorded', () => {
  test.use({ storageState: storageStatePath('admin') });

  test('reached from the sidebar, every row agrees with the API', async ({ page, request }) => {
    const token = await tokenFor(request, 'admin');
    const res = await request.get(`${API}/admin/status-sinkronisasi`, {
      headers: { Authorization: `Bearer ${token}` },
    });
    expect(res.ok(), await res.text()).toBeTruthy();
    const api = (await res.json()).data as SyncStatusEntry[];
    // integrasi's migration 002 seeds a row per source and part; the three the forms depend on
    // must be there in every environment.
    const keys = api.map((r) => `${r.sumber}/${r.bagian}`);
    for (const k of ['MySIMKARI/satker', 'MySIMKARI/pegawai', 'SIMAN/aset']) {
      expect(keys, `the API has no row ${k}`).toContain(k);
    }

    await page.goto(`${BASE}/dashboard`);
    await expect(shell(page)).toBeVisible({ timeout: 20000 });
    const nav = page.locator('nav');
    await nav.getByRole('button', { name: 'Admin', exact: true }).click();
    await nav.locator(`a[href="${PAGE}"]`).click();
    await expect(page).toHaveURL(new RegExp(`${PAGE}$`));

    const rows = page.getByTestId('sync-row');
    await expect(rows).toHaveCount(api.length, { timeout: 20000 });
    for (const r of api) {
      const row = page.locator(
        `[data-testid="sync-row"][data-sumber="${r.sumber}"][data-bagian="${r.bagian}"]`,
      );
      await expect(row, `${r.sumber}/${r.bagian}`).toHaveCount(1);
      await expect(row).toHaveAttribute('data-keadaan', r.keadaan);
      await expect(row.getByTestId('sync-keadaan')).not.toHaveText('Tidak dikenal');
      await expect(row.getByTestId('sync-jumlah')).toHaveText(String(r.jumlah_data));
      const reason = r.galat && r.galat.trim() ? r.galat : '—';
      await expect(row.getByTestId('sync-galat')).toHaveText(reason);
    }

    // The banner appears exactly when something needs an administrator.
    const needs = api.filter((r) => r.keadaan === 'GAGAL' || r.keadaan === 'TERLAMBAT').length;
    const banner = page.getByTestId('sync-attention');
    await expect(banner).toHaveCount(needs ? 1 : 0);
    if (needs) {
      await expect(banner).toContainText(`${needs} sumber perlu perhatian`);
    }
  });
});

test.describe('Sync status is for administrators only', () => {
  test.use({ storageState: storageStatePath('operator_a') });

  test('operator: no menu entry, the page is forbidden, the API answers 403', async ({
    page,
    request,
  }) => {
    const token = await tokenFor(request, 'operator_a');
    const res = await request.get(`${API}/admin/status-sinkronisasi`, {
      headers: { Authorization: `Bearer ${token}` },
    });
    expect(res.status(), await res.text()).toBe(403);

    await page.goto(`${BASE}/dashboard`);
    await expect(shell(page)).toBeVisible({ timeout: 20000 });
    await expect(page.locator('nav').locator(`a[href="${PAGE}"]`)).toHaveCount(0);

    await page.goto(PAGE, { waitUntil: 'domcontentloaded' });
    await expect(page.getByText(FORBIDDEN).first()).toBeVisible({ timeout: 15000 });
  });
});
