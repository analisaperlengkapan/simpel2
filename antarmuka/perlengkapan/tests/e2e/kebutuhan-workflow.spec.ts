/**
 * Kebutuhan BMN — FULL business workflow e2e (F-E2E E-1, flagship template).
 *
 * This is the first spec that DRIVES a real multi-role approval workflow through
 * the UI and asserts the resulting state in BOTH the browser and the backend —
 * the pattern every later domain (pakaian-dinas, pemakaian, penghapusan, …)
 * reuses. It replaces the old "reachability only" coverage (nav-access) for this
 * feature: buttons are clicked, a form is filled, and status transitions are
 * verified against the API, not screenshots.
 *
 * The kebutuhan flow is a PUSAT-authored campaign (periode) → per-satker
 * response. The wired UI is `KebutuhanBmnSatkerDetail`
 * (`/kebutuhan-bmn/satker/:id`), which gates actions by the row's `status_kode`:
 *   operator submits at 2000/2001 → validator_wilayah forwards at 2002 →
 *   validator_pusat decides at 2005 (→ 2006 approved / 2007 rejected).
 * (Note: `pages/kebutuhan_bmn/{submission_form,review_page}.rs` are DEAD/unwired
 * — the live workflow lives in `components/kebutuhan_bmn_satker.rs`.)
 *
 * The `seed-multisatker.sql` fixture seeds ONE open campaign + per-satker rows at
 * DISTINCT statuses (known UUIDs) so each role's transition is exercised
 * independently and deterministically — no brittle single long chain:
 *   S1 0200010 @2001 → operator_a: Tambah Barang + Submit ke Wilayah  (→2002)
 *   S2 0200020 @2002 → validator_wilayah (DKI): Teruskan ke Pusat     (off 2002)
 *   S3 0300010 @2005 → validator_pusat: Setujui                        (→2006)
 * Real per-role JWTs come from `auth.setup.ts` storageState.
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
const CAMPAIGN_ID = 'c1000000-0000-4c00-8c00-000000000001';
const S1_OPERATOR = 'c1000000-0000-4c00-8c00-0000000a0001'; // 0200010 @2001
const S2_WILAYAH = 'c1000000-0000-4c00-8c00-0000000a0002'; //  0200020 @2002
const S3_PUSAT = 'c1000000-0000-4c00-8c00-0000000a0003'; //    0300010 @2005

const userFor = (key: string) => {
  const u = TEST_USERS.find((t) => t.key === key);
  if (!u) throw new Error(`unknown test user ${key}`);
  return u;
};

/**
 * Backend source-of-truth: read a satker row's status_kode from the API. Uses
 * the validator_pusat token (cross-satker, sees every row) and the well-typed
 * list endpoint GET /pengajuan/{id}/satker → PengajuanKebutuhanBmnSatker[].
 */
async function beStatusKode(request: APIRequestContext, satkerRowId: string): Promise<number> {
  const { accessToken } = await apiLogin(request, credsFor(userFor('validator_pusat')));
  const resp = await request.get(
    `${PERLENGKAPAN_API_URL}/api/v1/perlengkapan/kebutuhan-bmn/pengajuan/${CAMPAIGN_ID}/satker?page=1&per_page=100`,
    { headers: { Authorization: `Bearer ${accessToken}` } },
  );
  expect(resp.ok(), `GET pengajuan/${CAMPAIGN_ID}/satker (${resp.status()})`).toBeTruthy();
  const body = await resp.json();
  const rows: Array<{ id: string; status_kode: number }> = body.data ?? [];
  const row = rows.find((r) => r.id === satkerRowId);
  expect(row, `satker row ${satkerRowId} present in campaign`).toBeTruthy();
  return row!.status_kode;
}

/** The authenticated app shell mounts a sticky <header> containing "SIMPEL". */
const shell = (page: import('@playwright/test').Page) =>
  page.locator('header').getByText('SIMPEL').first();

async function openSatker(page: import('@playwright/test').Page, satkerId: string) {
  await page.goto(`${BASE}/kebutuhan-bmn/satker/${satkerId}`, { waitUntil: 'domcontentloaded' });
  await expect(shell(page), 'app shell mounts').toBeVisible({ timeout: 20000 });
}

// ── Operator: add a barang + submit the satker to Validator Wilayah ──────────
test.describe('Kebutuhan BMN — operator submit to wilayah', () => {
  test.use({ storageState: storageStatePath('operator_a') });

  test('operator_a adds a barang and submits S1 (2001 → 2002)', async ({ page, request }) => {
    // precondition: backend says S1 is at 2001 (Input Barang)
    expect(await beStatusKode(request, S1_OPERATOR)).toBe(2001);

    await openSatker(page, S1_OPERATOR);
    // the pre-seeded barang is visible in the Barang tab table
    await expect(page.getByText('E2E Barang Seed A').first()).toBeVisible({ timeout: 20000 });

    // add a second barang through the real modal form
    await page.getByRole('button', { name: 'Tambah Barang' }).click();
    const form = page.locator('form:has(button:has-text("Simpan"))');
    await expect(form).toBeVisible();
    await form.getByRole('textbox').first().fill('E2E Barang UI Operator'); // Nama Barang
    await form.getByRole('spinbutton').first().fill('4'); // Jumlah
    await form.getByRole('button', { name: 'Simpan' }).click();
    await expect(page.getByText('E2E Barang UI Operator').first()).toBeVisible({ timeout: 15000 });

    // submit to wilayah
    await page.getByRole('button', { name: /Submit ke Wilayah/i }).click();

    // UI reflects the transition: the operator submit action is gone after reload
    await page.reload({ waitUntil: 'domcontentloaded' });
    await expect(shell(page)).toBeVisible({ timeout: 20000 });
    await expect(page.getByRole('button', { name: /Submit ke Wilayah/i })).toHaveCount(0);

    // backend source-of-truth: S1 moved 2001 → 2002
    await expect
      .poll(() => beStatusKode(request, S1_OPERATOR), { timeout: 15000 })
      .toBe(2002);
  });
});

// ── Validator Wilayah: forward the submitted satker to Pusat ─────────────────
test.describe('Kebutuhan BMN — validator wilayah forwards to pusat', () => {
  test.use({ storageState: storageStatePath('validator_wilayah') });

  test('validator_wilayah forwards S2 off 2002', async ({ page, request }) => {
    expect(await beStatusKode(request, S2_WILAYAH)).toBe(2002);

    await openSatker(page, S2_WILAYAH);
    await page.getByRole('button', { name: /Teruskan ke Pusat/i }).click();

    await page.reload({ waitUntil: 'domcontentloaded' });
    await expect(shell(page)).toBeVisible({ timeout: 20000 });
    // the wilayah forward action is no longer offered
    await expect(page.getByRole('button', { name: /Teruskan ke Pusat/i })).toHaveCount(0);

    // backend: S2 advanced past "submitted to wilayah" (2002)
    await expect.poll(() => beStatusKode(request, S2_WILAYAH), { timeout: 15000 }).not.toBe(2002);
  });
});

// ── Validator Pusat: approve the satker ─────────────────────────────────────
test.describe('Kebutuhan BMN — validator pusat approves', () => {
  test.use({ storageState: storageStatePath('validator_pusat') });

  test('validator_pusat approves S3 (2005 → 2006)', async ({ page, request }) => {
    expect(await beStatusKode(request, S3_PUSAT)).toBe(2005);

    await openSatker(page, S3_PUSAT);
    await page.getByRole('button', { name: 'Setujui' }).click();

    await page.reload({ waitUntil: 'domcontentloaded' });
    await expect(shell(page)).toBeVisible({ timeout: 20000 });
    // approval action gone once decided
    await expect(page.getByRole('button', { name: 'Setujui' })).toHaveCount(0);

    // backend: S3 approved (2006)
    await expect.poll(() => beStatusKode(request, S3_PUSAT), { timeout: 15000 }).toBe(2006);
  });
});

// ── RBAC isolation: an operator cannot act on another satker's row ───────────
test.describe('Kebutuhan BMN — cross-satker isolation', () => {
  test.use({ storageState: storageStatePath('operator_b') });

  test('operator_b cannot submit operator_a’s S1 (0200010)', async ({ page, request }) => {
    // operator_b belongs to 0200020; the API must not expose the 0200010 row's
    // action to them. Open A's satker and assert no operator action is offered.
    await openSatker(page, S1_OPERATOR);
    await expect(page.getByRole('button', { name: /Submit ke Wilayah/i })).toHaveCount(0);

    // and the backend denies a direct read of a foreign satker to operator_b
    const { accessToken } = await apiLogin(request, credsFor(userFor('operator_b')));
    const resp = await request.get(
      `${PERLENGKAPAN_API_URL}/api/v1/perlengkapan/kebutuhan-bmn/satker/${S1_OPERATOR}`,
      { headers: { Authorization: `Bearer ${accessToken}` } },
    );
    expect(resp.status(), 'operator_b reading foreign satker is denied').toBeGreaterThanOrEqual(400);
  });
});
