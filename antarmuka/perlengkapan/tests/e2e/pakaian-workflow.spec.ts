/**
 * Pakaian Dinas — business workflow e2e (F-E2E E-2). Second domain, reusing the
 * E-1 template: drive the WIRED workflow UI and assert state transitions against
 * the backend.
 *
 * Pakaian dinas is a pusat-authored campaign (pengajuan_pakaian_dinas) → per-satker
 * response. Wired driver = `components/pakaian_dinas_satker.rs`
 * (`/pakaian-dinas/pengajuan/:pengajuan_id/satker`): an in-page satker LIST; a
 * validator clicks a satker row → an action panel appears → the status-derived
 * action ("Teruskan ke Pusat" = aksi "approve") opens an ApprovalDialog → "Kirim".
 * The backend decides the target `aktivitas_id` (statuses: 1000 INPUT, 1001
 * SUBMIT_TO_VALIDATOR, 1004 SUBMIT_TO_PUSAT, 1008 SELESAI, 100x revisi/ditolak).
 *
 * Seed (seed-multisatker.sql): one jenis (master) + one campaign + per-satker rows
 * at distinct aktivitas — P1 0200010 @1001 (validator_wilayah forwards), P2 0200020
 * @1004 (pusat). `satker_id` is the mysimkari UUID (seeded via subquery).
 */
import { test, expect, type APIRequestContext } from '@playwright/test';
import {
  apiLogin,
  credsFor,
  storageStatePath,
  TEST_USERS,
  PERLENGKAPAN_API_URL,
} from './helpers/real-auth';

const BASE = '/perlengkapan/simpel/v2';
const PD_API = `${PERLENGKAPAN_API_URL}/api/v1/perlengkapan/pakaian-dinas`;
const CAMPAIGN = 'd1000000-0000-4d00-8d00-0000000000c1';
const P1_WILAYAH = 'd1000000-0000-4d00-8d00-0000000a0001'; // 0200010 @1001

const userFor = (key: string) => {
  const u = TEST_USERS.find((t) => t.key === key);
  if (!u) throw new Error(`unknown test user ${key}`);
  return u;
};

/** Backend source-of-truth: a satker row's aktivitas_id, read as validator_pusat. */
async function beAktivitas(request: APIRequestContext, satkerRowId: string): Promise<number> {
  const { accessToken } = await apiLogin(request, credsFor(userFor('validator_pusat')));
  const resp = await request.get(
    `${PD_API}/pengajuan/${CAMPAIGN}/satker?page=1&per_page=100`,
    { headers: { Authorization: `Bearer ${accessToken}` } },
  );
  expect(resp.ok(), `GET pengajuan/${CAMPAIGN}/satker (${resp.status()})`).toBeTruthy();
  const rows: Array<{ id: string; aktivitas_id: number }> = (await resp.json()).data ?? [];
  const row = rows.find((r) => r.id === satkerRowId);
  expect(row, `pakaian satker row ${satkerRowId} present`).toBeTruthy();
  return row!.aktivitas_id;
}

const shell = (page: import('@playwright/test').Page) =>
  page.locator('header').getByText('SIMPEL').first();

// ── Validator drives the campaign's satker forward ──────────────────────────
test.describe('Pakaian Dinas — validator wilayah forwards a satker', () => {
  test.use({ storageState: storageStatePath('validator_wilayah') });

  test('validator_wilayah forwards P1 off 1001 via the satker action panel', async ({
    page,
    request,
  }) => {
    expect(await beAktivitas(request, P1_WILAYAH)).toBe(1001);

    await page.goto(`${BASE}/pakaian-dinas/pengajuan/${CAMPAIGN}/satker`, {
      waitUntil: 'domcontentloaded',
    });
    await expect(shell(page), 'satker page mounts').toBeVisible({ timeout: 20000 });

    // select the JAKPUS (0200010) satker row → its action panel appears
    await page.getByText('KEJAKSAAN NEGERI JAKARTA PUSAT').first().click();
    // approve = "Teruskan ke Pusat"; opens the confirm dialog → "Kirim"
    await page.getByRole('button', { name: /Teruskan ke Pusat/i }).click();
    await page.getByRole('button', { name: 'Kirim' }).click();

    // backend: P1 advanced off 1001 (SUBMIT_TO_VALIDATOR)
    await expect.poll(() => beAktivitas(request, P1_WILAYAH), { timeout: 15000 }).not.toBe(1001);
  });
});

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

  test('laporan page mounts with an export control', async ({ page }) => {
    await page.goto(`${BASE}/pakaian-dinas/laporan`, { waitUntil: 'domcontentloaded' });
    await expect(shell(page)).toBeVisible({ timeout: 20000 });
    await expect(
      page.getByRole('button', { name: /Cetak (PDF|Excel)/i }).first(),
    ).toBeVisible({ timeout: 15000 });
  });
});

// ── RBAC: the validator-action endpoint is role-gated ───────────────────────
test.describe('Pakaian Dinas — role-gated validator action', () => {
  test('an operator cannot drive a validator action', async ({ request }) => {
    const op = await apiLogin(request, credsFor(userFor('operator_a')));
    const resp = await request.post(`${PD_API}/validator-action`, {
      headers: { Authorization: `Bearer ${op.accessToken}` },
      data: { pengajuan_satker_id: P1_WILAYAH, aksi: 'approve', komentar: 'e2e role-gate probe' },
    });
    expect(resp.status(), 'operator blocked from validator-action').toBeGreaterThanOrEqual(400);
  });
});
