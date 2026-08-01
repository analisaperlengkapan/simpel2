/**
 * Pakaian Dinas — e2e (F-E2E E-2). Reuses the E-1 template (drive the wired UI +
 * assert real state), scoped to the pakaian surface that is FUNCTIONAL.
 *
 * The per-satker WORKFLOW was descoped when this spec was written because it was
 * broken against the real integrasi schema: `pengajuan_pakaian_dinas_satker.satker_id`
 * was `uuid` while `integrasi.mysimkari_satker.id` is `bigint` (BIGSERIAL), and the
 * BE joined them directly (`ON ps.satker_id = s.id`), which Postgres rejects. V006/#94
 * reconciles the key to MySIMKARI `kode_satker`, so the workflow drive is RESTORED
 * below — it is the regression test for that fix.
 *
 * Exercised here:
 *   - jenis master + pengajuan (campaign) + laporan pages are reachable,
 *   - the per-satker list renders satker names resolved THROUGH the mysimkari join
 *     (the join that used to be invalid SQL),
 *   - validator_wilayah forwards a satker 1001 → 1004, validator_pusat approves
 *     1004 → 1008, each asserted against backend state,
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

  // These three assert page REACHABILITY (shell mounts, no auth bounce). The
  // seeded-row assertions that matter for #94 live in the satker describe below,
  // which is where the mysimkari join actually runs.
  // (Full literal paths so the route-coverage gate detects these as visited.)
  const reachable = async (page: import('@playwright/test').Page, path: string) => {
    await page.goto(path, { waitUntil: 'domcontentloaded' });
    await expect(shell(page), `shell mounts on ${path}`).toBeVisible({ timeout: 20000 });
    expect(/login/i.test(page.url()), `must not redirect to login (${page.url()})`).toBeFalsy();
  };

  test('jenis master page is reachable', async ({ page }) => {
    await reachable(page, `${BASE}/pakaian-dinas/jenis`);
  });
  test('pengajuan (campaign) page is reachable', async ({ page }) => {
    await reachable(page, `${BASE}/pakaian-dinas/pengajuan`);
  });
  test('laporan page is reachable', async ({ page }) => {
    await reachable(page, `${BASE}/pakaian-dinas/laporan`);
  });
});

// ── Per-satker list + workflow (restored by V006/#94) ───────────────────────
// Seeded by tests/fixtures/e2e/seed-perlengkapan-workflow.sql section 6:
//   P1 satker 0200010 @1001 SUBMIT_TO_VALIDATOR → validator_wilayah approves → 1004
//   P2 satker 0200020 @1004 SUBMIT_TO_PUSAT     → validator_pusat  approves → 1008
const CAMPAIGN = 'd1000000-0000-4d00-8d00-0000000000c1';
const P1_WILAYAH = 'd1000000-0000-4d00-8d00-0000000a0001';
const P2_PUSAT = 'd1000000-0000-4d00-8d00-0000000a0002';

/**
 * Backend source-of-truth for a satker row's aktivitas_id. Reads the per-campaign
 * satker list as validator_pusat (cross-satker, sees every row) — the SAME endpoint
 * whose `LEFT JOIN integrasi.mysimkari_satker` was invalid SQL before #94, so a
 * successful read here is itself part of the regression assertion.
 */
async function beAktivitas(
  request: import('@playwright/test').APIRequestContext,
  satkerRowId: string,
): Promise<number> {
  const { accessToken } = await apiLogin(request, credsFor(userFor('validator_pusat')));
  const resp = await request.get(`${PD_API}/pengajuan/${CAMPAIGN}/satker?page=1&per_page=100`, {
    headers: { Authorization: `Bearer ${accessToken}` },
  });
  expect(resp.ok(), `GET pengajuan/${CAMPAIGN}/satker (${resp.status()})`).toBeTruthy();
  const body = await resp.json();
  const rows: Array<{ id: string; aktivitas_id: number }> = body.data ?? [];
  const row = rows.find((r) => r.id === satkerRowId);
  expect(row, `satker row ${satkerRowId} present in campaign`).toBeTruthy();
  return row!.aktivitas_id;
}

test.describe('Pakaian Dinas — per-satker list resolves the mysimkari join', () => {
  test('satker list carries names resolved from integrasi.mysimkari_satker', async ({
    request,
  }) => {
    const { accessToken } = await apiLogin(request, credsFor(userFor('validator_pusat')));
    const resp = await request.get(`${PD_API}/pengajuan/${CAMPAIGN}/satker?page=1&per_page=100`, {
      headers: { Authorization: `Bearer ${accessToken}` },
    });
    expect(resp.status(), 'satker list no longer errors on uuid=bigint').toBe(200);

    const body = await resp.json();
    const rows: Array<{ satker_id: string; satker_nama: string | null }> = body.data ?? [];
    const codes = rows.map((r) => r.satker_id);
    expect(codes, 'both seeded satkers present, keyed by kode_satker').toEqual(
      expect.arrayContaining(['0200010', '0200020']),
    );

    // satker_nama comes ONLY from the join, so a non-null value proves it resolved.
    const jakpus = rows.find((r) => r.satker_id === '0200010');
    expect(jakpus?.satker_nama, 'name resolved through mysimkari_satker').toContain(
      'JAKARTA PUSAT',
    );
  });
});

test.describe('Pakaian Dinas — validator wilayah forwards a satker', () => {
  test('validator_wilayah advances P1 (1001 → 1004)', async ({ request }) => {
    expect(await beAktivitas(request, P1_WILAYAH)).toBe(1001);

    const wil = await apiLogin(request, credsFor(userFor('validator_wilayah')));
    const resp = await request.post(`${PD_API}/validator-action`, {
      headers: { Authorization: `Bearer ${wil.accessToken}` },
      data: { pengajuan_satker_id: P1_WILAYAH, aksi: 'approve', komentar: 'e2e teruskan' },
    });
    expect(resp.status(), 'validator_wilayah may approve at 1001').toBe(200);

    await expect.poll(() => beAktivitas(request, P1_WILAYAH), { timeout: 15000 }).toBe(1004);
  });
});

test.describe('Pakaian Dinas — validator pusat decides', () => {
  test('validator_pusat approves P2 (1004 → 1008 Selesai)', async ({ request }) => {
    expect(await beAktivitas(request, P2_PUSAT)).toBe(1004);

    const pusat = await apiLogin(request, credsFor(userFor('validator_pusat')));
    const resp = await request.post(`${PD_API}/validator-action`, {
      headers: { Authorization: `Bearer ${pusat.accessToken}` },
      data: { pengajuan_satker_id: P2_PUSAT, aksi: 'approve', komentar: 'e2e setujui' },
    });
    expect(resp.status(), 'validator_pusat may approve at 1004').toBe(200);

    await expect.poll(() => beAktivitas(request, P2_PUSAT), { timeout: 15000 }).toBe(1008);
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
