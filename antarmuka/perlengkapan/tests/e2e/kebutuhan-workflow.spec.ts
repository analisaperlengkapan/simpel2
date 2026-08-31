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
import { clickAction } from './helpers/workflow';

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

    // add a second barang through the real modal form.
    //
    // Nama and kode barang are no longer typed — both already exist in the
    // SIMAN codification, and typing them is what produced wrong codes and two
    // spellings of one item that stop aggregating together. The form now picks
    // an entry, so the assertion is on the STANDARD name that belongs to the
    // code, not on a string this test invented.
    await page.getByRole('button', { name: 'Tambah Barang' }).click();
    const form = page.locator('form:has(button:has-text("Simpan"))');
    await expect(form).toBeVisible();

    await form.getByLabel('Cari Barang').fill('Station Wagon');
    const hasil = form.getByTestId('kodefikasi-hasil');
    await expect(hasil, 'codification picker returned nothing').toBeVisible({ timeout: 20000 });
    const baris = hasil.locator('li');
    await expect(baris, 'one entry per code, not one per asset').toHaveCount(1);
    await baris.first().getByRole('button').click();

    // Picked, so the code comes with it rather than being typed beside it.
    const terpilih = form.getByTestId('kodefikasi-terpilih');
    await expect(terpilih).toBeVisible();
    await expect(terpilih, 'the code travels with the name').toContainText('3020101003');

    await form.getByRole('spinbutton').first().fill('4'); // Jumlah
    await form.getByTestId('barang-simpan').click();
    await expect(page.getByText('Station Wagon').first()).toBeVisible({ timeout: 15000 });

    // The manual escape hatch, checked here rather than in its own test
    // because the modal is only reachable while the satker sits at 2001 —
    // a separate test would depend on this one not having submitted yet.
    //
    // It has to exist: the codification is derived from assets already on the
    // register, so a barang no satker owns yet is genuinely absent, and a needs
    // request naming one is legitimate. It also has to be an ESCAPE hatch —
    // reachable only after a search actually came back empty, never offered
    // alongside the picker as an equal path.
    await page.getByRole('button', { name: 'Tambah Barang' }).click();
    await expect(form).toBeVisible();
    await expect(
      form.getByTestId('kodefikasi-manual'),
      'the manual fields must not be reachable before a search fails',
    ).toHaveCount(0);
    await expect(
      form.getByTestId('barang-simpan'),
      'nothing picked, nothing to save',
    ).toBeDisabled();

    await form.getByLabel('Cari Barang').fill('Barang Yang Belum Ada Di Register');
    await expect(form.getByTestId('kodefikasi-kosong')).toBeVisible({ timeout: 20000 });
    await form.getByRole('button', { name: 'Isi manual' }).click();
    await expect(form.getByTestId('kodefikasi-manual')).toBeVisible();
    await expect(
      form.getByTestId('barang-simpan'),
      'the manual name carries over from the search box',
    ).toBeEnabled();
    // Scoped to the form: 'Batal' is a common label and the page behind the
    // modal has its own.
    await form.getByRole('button', { name: 'Batal' }).click();
    await expect(form).toHaveCount(0);

    // submit to wilayah
    await clickAction(
      page,
      page.getByRole('button', { name: /Submit ke Wilayah/i }),
      'submit-wilayah',
    );

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
    await clickAction(
      page,
      page.getByRole('button', { name: /Teruskan ke Pusat/i }),
      'validator-wilayah',
    );

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
    await clickAction(
      page,
      page.getByRole('button', { name: 'Setujui' }),
      'keputusan-pusat',
    );

    await page.reload({ waitUntil: 'domcontentloaded' });
    await expect(shell(page)).toBeVisible({ timeout: 20000 });
    // approval action gone once decided
    await expect(page.getByRole('button', { name: 'Setujui' })).toHaveCount(0);

    // backend: S3 approved (2006)
    await expect.poll(() => beStatusKode(request, S3_PUSAT), { timeout: 15000 }).toBe(2006);
  });
});

// ── RBAC: workflow actions are role-gated ────────────────────────────────────
// Each transition endpoint enforces the caller's ROLE (require_role):
// submit-wilayah = operator_satker only; keputusan-pusat = validator_pusat only.
// A wrong-role caller must be rejected (no unauthorized mutation).
const KB_API = `${PERLENGKAPAN_API_URL}/api/v1/perlengkapan/kebutuhan-bmn`;

test.describe('Kebutuhan BMN — role-gated workflow actions', () => {
  test('a wrong-role caller cannot drive a workflow transition', async ({ request }) => {
    // validator_pusat may NOT perform the operator-only "submit to wilayah"
    const pusat = await apiLogin(request, credsFor(userFor('validator_pusat')));
    const r1 = await request.post(`${KB_API}/satker/${S1_OPERATOR}/submit-wilayah`, {
      headers: { Authorization: `Bearer ${pusat.accessToken}` },
      data: { catatan_satker: 'e2e role-gate probe' },
    });
    expect(r1.status(), 'validator_pusat blocked from operator submit').toBeGreaterThanOrEqual(400);

    // operator_a may NOT perform the validator_pusat-only decision
    const op = await apiLogin(request, credsFor(userFor('operator_a')));
    const r2 = await request.post(`${KB_API}/satker/${S3_PUSAT}/keputusan-pusat`, {
      headers: { Authorization: `Bearer ${op.accessToken}` },
      data: { is_approved: true, alasan_keputusan: 'e2e role-gate probe' },
    });
    expect(r2.status(), 'operator blocked from pusat decision').toBeGreaterThanOrEqual(400);
  });
});

// ── RBAC: object-level satker scoping on the single-record READ (#93) ────────
// The role gating above answers "may this ROLE act?". It does NOT answer "may
// this caller touch THIS row?" — the gap #93 was filed for: `get_satker_detail`
// took `_claims` (unused) and returned 200 to any authenticated user. Because
// the seeded campaign is nationwide (`scope_satker = 'semua'`), every satker has
// a row in it, so operator_a could read operator_b's submission by id alone.
//
// The seed gives all four SatkerScope tiers against one endpoint:
//   S1 0200010 DKI JAKARTA (operator_a's own satker)
//   S2 0200020 DKI JAKARTA (operator_b's satker — same wilayah as S1)
//   S3 0300010 JAWA BARAT  (different wilayah)
// so this also exercises the DB-backed Wilayah tier, not only the pure ones.
//
// Out-of-scope reads must answer 404, NOT 403: a 403 confirms the row exists and
// turns the endpoint into an existence oracle across satkers.
test.describe('Kebutuhan BMN — object-level satker scoping (#93)', () => {
  const detail = (request: APIRequestContext, token: string, id: string) =>
    request.get(`${KB_API}/satker/${id}?page=1&per_page=20`, {
      headers: { Authorization: `Bearer ${token}` },
    });

  test('operator reads its own satker row but not another satker in the campaign', async ({
    request,
  }) => {
    const { accessToken } = await apiLogin(request, credsFor(userFor('operator_a')));

    const own = await detail(request, accessToken, S1_OPERATOR);
    expect(own.status(), 'operator_a reads its OWN satker detail').toBe(200);

    const other = await detail(request, accessToken, S2_WILAYAH);
    expect(other.status(), 'operator_a denied operator_b detail (fails closed)').toBe(404);
  });

  test('validator_wilayah reads across its wilayah but not beyond it', async ({ request }) => {
    const { accessToken } = await apiLogin(request, credsFor(userFor('validator_wilayah')));

    // DKI validator over a DKI satker it does not itself belong to: allowed.
    const inRegion = await detail(request, accessToken, S2_WILAYAH);
    expect(inRegion.status(), 'validator_wilayah (DKI) reads a DKI satker').toBe(200);

    // Same role, different wilayah: denied.
    const outOfRegion = await detail(request, accessToken, S3_PUSAT);
    expect(outOfRegion.status(), 'validator_wilayah (DKI) denied a JAWA BARAT satker').toBe(404);
  });

  test('validator_pusat is unrestricted across satkers', async ({ request }) => {
    const { accessToken } = await apiLogin(request, credsFor(userFor('validator_pusat')));

    for (const [id, label] of [
      [S1_OPERATOR, '0200010'],
      [S2_WILAYAH, '0200020'],
      [S3_PUSAT, '0300010'],
    ] as const) {
      const resp = await detail(request, accessToken, id);
      expect(resp.status(), `validator_pusat reads satker ${label}`).toBe(200);
    }
  });
});
