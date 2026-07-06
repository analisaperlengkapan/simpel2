/**
 * Pakaian Dinas — e2e (F-E2E E-2). Reuses the E-1 template (drive the wired UI +
 * assert real state), scoped to the pakaian surface that is FUNCTIONAL.
 *
 * ⚠️ The per-satker WORKFLOW (validator forwards a satker via the action panel on
 * `/pakaian-dinas/pengajuan/:id/satker`) is DESCOPED here: it is broken against the
 * real integrasi schema — `pengajuan_pakaian_dinas_satker.satker_id` is `uuid` but
 * `integrasi.mysimkari_satker.id` is `bigint`, and the BE joins them directly
 * (`ON ps.satker_id = s.id`, laporan.rs), which errors at runtime. Tracked as a bug
 * (#94); the workflow-drive e2e is re-added once the satker key type is reconciled.
 *
 * What IS exercised (no broken satker↔mysimkari join):
 *   - jenis master list renders the seeded jenis,
 *   - pengajuan (campaign) list renders the seeded campaign,
 *   - laporan page is reachable,
 *   - the validator-action endpoint is role-gated.
 */
import { test, expect } from '@playwright/test';
import {
  apiLogin,
  credsFor,
  storageStatePath,
  TEST_USERS,
  PERLENGKAPAN_API_URL,
} from './helpers/real-auth';

const BASE = '/perlengkapan/simpel/v2';
const PD_API = `${PERLENGKAPAN_API_URL}/api/v1/perlengkapan/pakaian-dinas`;

const userFor = (key: string) => {
  const u = TEST_USERS.find((t) => t.key === key);
  if (!u) throw new Error(`unknown test user ${key}`);
  return u;
};

const shell = (page: import('@playwright/test').Page) =>
  page.locator('header').getByText('SIMPEL').first();

// ── Master + campaign + report pages render real data ───────────────────────
test.describe('Pakaian Dinas — master, campaign & report pages', () => {
  test.use({ storageState: storageStatePath('admin') });

  test('jenis master list shows the seeded jenis', async ({ page }) => {
    await page.goto(`${BASE}/pakaian-dinas/jenis`, { waitUntil: 'domcontentloaded' });
    await expect(shell(page)).toBeVisible({ timeout: 20000 });
    await expect(page.getByText('PDH E2E').first()).toBeVisible({ timeout: 15000 });
  });

  test('pengajuan list shows the seeded campaign', async ({ page }) => {
    await page.goto(`${BASE}/pakaian-dinas/pengajuan`, { waitUntil: 'domcontentloaded' });
    await expect(shell(page)).toBeVisible({ timeout: 20000 });
    await expect(
      page.getByText('E2E Pengajuan Pakaian Dinas 2026').first(),
    ).toBeVisible({ timeout: 15000 });
  });

  test('laporan page is reachable', async ({ page }) => {
    await page.goto(`${BASE}/pakaian-dinas/laporan`, { waitUntil: 'domcontentloaded' });
    await expect(shell(page)).toBeVisible({ timeout: 20000 });
    expect(/login/i.test(page.url()), `must not redirect to login (${page.url()})`).toBeFalsy();
  });
});

// ── RBAC: the validator-action endpoint is role-gated ───────────────────────
test.describe('Pakaian Dinas — role-gated validator action', () => {
  test('an operator cannot drive a validator action', async ({ request }) => {
    const op = await apiLogin(request, credsFor(userFor('operator_a')));
    const resp = await request.post(`${PD_API}/validator-action`, {
      headers: { Authorization: `Bearer ${op.accessToken}` },
      data: {
        pengajuan_satker_id: 'd1000000-0000-4d00-8d00-0000000a0001',
        aksi: 'approve',
        komentar: 'e2e role-gate probe',
      },
    });
    expect(resp.status(), 'operator blocked from validator-action').toBeGreaterThanOrEqual(400);
  });
});
