/**
 * F-E2E E-4 — Bank Aset + Analitik + Dashboard + Notifikasi (perlengkapan).
 *
 * Real-auth, per-role storageState; asserts real UI state AND the backend behind
 * it (never presence-only, never screenshots). Preconditions come from
 * tests/fixtures/e2e/seed-multisatker.sql (5 SIMAN assets) +
 * seed-perlengkapan-workflow.sql (notifikasi inbox).
 *
 * SCOPE NOTE — deliberately NOT duplicated here:
 *   bank-aset list *scoping* per role already has dedicated coverage in
 *   rbac-scoping.spec.ts (API) and rbac-scoping-ui.spec.ts (UI). E-4 covers what
 *   those don't: the filter/search/sort/pagination controls, the detail page,
 *   sebaran, the QR generator, dashboard + global search, analitik roadmap
 *   create→list, kodefikasi, and the notifikasi read lifecycle.
 *
 * FINDING #97 (surfaced by this domain's recon, filed rather than faked):
 *   The BE wires GET /dashboard/perlengkapan/export/excel, /export/pdf and
 *   /dashboard/ws, but NOTHING in antarmuka/ calls them — the dashboard renders
 *   stat cards only (no export button, no websocket). The F-E2E plan's "dashboard
 *   export (verify downloaded file) + websocket realtime" is therefore
 *   UI-unreachable. This spec asserts the export endpoints server-side via the
 *   API (they are real and must not rot) and does NOT pretend to drive a UI that
 *   does not exist. When the FE lands, the drive moves into the browser.
 *
 * OBSERVED (documented, not asserted as desired): /analisis has NO role gate —
 * get_all_analisis/get_analisis_by_id ignore claims (`_claims`) and create_analisis
 * only stamps created_by. analisis_kebutuhan has no satker column, so the roadmap
 * is a global, any-authenticated-user artifact, unlike every other module which
 * gates writes with require_role. Asserted below as the behaviour that exists.
 *
 * Seeded assets (integrasi.siman_aset, shared seed section 3):
 *   E2E-A-1 Toyota Avanza  Peralatan dan Mesin / Alat Angkutan / BAIK         0200010
 *   E2E-A-2 Laptop Dell    Peralatan dan Mesin / Alat Kantor   / BAIK         0200010
 *   E2E-B-1 Honda Vario    Peralatan dan Mesin / Alat Angkutan / BAIK         0200020
 *   E2E-B-2 Tanah Kantor   Tanah               / Tanah         / BAIK         0200020
 *   E2E-C-1 Printer Epson  Peralatan dan Mesin / Alat Kantor   / RUSAK RINGAN 0300010
 * validator_pusat sees all 5 → the only role for which filter maths is stable.
 */
import { test, expect, type Page, type APIRequestContext } from '@playwright/test';
import { credsFor, apiLogin, storageStatePath, TEST_USERS, PERLENGKAPAN_API_URL } from './helpers/real-auth';

const BASE = '/perlengkapan/simpel/v2';
const API = `${PERLENGKAPAN_API_URL}/api/v1/perlengkapan`;

/** Seeded notifikasi UUIDs (seed-perlengkapan-workflow.sql section 9). */
const N1 = 'f1000000-0000-4f00-8f00-0000000c0001'; // operator_a, unread
const N2 = 'f1000000-0000-4f00-8f00-0000000c0002'; // operator_a, unread
const N3 = 'f1000000-0000-4f00-8f00-0000000c0003'; // operator_a, already read
const N4 = 'f1000000-0000-4f00-8f00-0000000c0004'; // operator_b, unread

/** The authenticated app shell has mounted (not bounced to the portal login). */
const shell = (page: Page) => page.getByRole('heading', { name: 'SIMPEL' }).first();

/** Real per-role JWT (apiLogin returns a token PAIR — the header needs .accessToken). */
async function tokenFor(request: APIRequestContext, userKey: string): Promise<string> {
  const user = TEST_USERS.find((u) => u.key === userKey);
  if (!user) throw new Error(`unknown seeded test user: ${userKey}`);
  const { accessToken } = await apiLogin(request, credsFor(user));
  return accessToken;
}

/** Rows currently rendered in the bank-aset list (the NUP column is unique per asset). */
async function visibleNups(page: Page): Promise<string[]> {
  const body = await page.locator('body').innerText();
  return ['E2E-A-1', 'E2E-A-2', 'E2E-B-1', 'E2E-B-2', 'E2E-C-1'].filter((n) => body.includes(n));
}

// ---------------------------------------------------------------------------
// Bank Aset — filters / search / sort / pagination drive the real list
// ---------------------------------------------------------------------------
test.describe('Bank Aset — daftar controls (validator_pusat: all 5 assets in scope)', () => {
  test.use({ storageState: storageStatePath('validator_pusat') });

  test('list renders every seeded asset before filtering', async ({ page }) => {
    await page.goto(`${BASE}/bank-aset/daftar`);
    await expect(shell(page)).toBeVisible();
    await expect(page.getByText('Daftar Aset').first()).toBeVisible();
    await expect.poll(() => visibleNups(page), { timeout: 20000 }).toEqual(
      ['E2E-A-1', 'E2E-A-2', 'E2E-B-1', 'E2E-B-2', 'E2E-C-1'],
    );
  });

  test('search narrows to the matching asset and Reset restores the full list', async ({ page }) => {
    await page.goto(`${BASE}/bank-aset/daftar`);
    await expect(shell(page)).toBeVisible();
    await expect.poll(() => visibleNups(page), { timeout: 20000 }).toHaveLength(5);

    await page.getByPlaceholder('Cari nama/kode/NUP/merk...').fill('Toyota');
    await page.getByRole('button', { name: 'Terapkan' }).click();
    // Real server-side search: only the Toyota row survives.
    await expect.poll(() => visibleNups(page), { timeout: 20000 }).toEqual(['E2E-A-1']);
    await expect(page.getByText('Toyota Avanza').first()).toBeVisible();

    await page.getByRole('button', { name: 'Reset' }).click();
    await expect.poll(() => visibleNups(page), { timeout: 20000 }).toHaveLength(5);
  });

  test('kondisi filter isolates the single RUSAK RINGAN asset', async ({ page }) => {
    await page.goto(`${BASE}/bank-aset/daftar`);
    await expect(shell(page)).toBeVisible();
    await expect.poll(() => visibleNups(page), { timeout: 20000 }).toHaveLength(5);

    await page.getByRole('combobox').filter({ hasText: 'Semua Kondisi' }).selectOption({ label: 'RUSAK RINGAN' });
    await expect.poll(() => visibleNups(page), { timeout: 20000 }).toEqual(['E2E-C-1']);
    await expect(page.getByText('Printer Epson').first()).toBeVisible();
  });

  test('jenis filter isolates the single Tanah asset', async ({ page }) => {
    await page.goto(`${BASE}/bank-aset/daftar`);
    await expect(shell(page)).toBeVisible();
    await expect.poll(() => visibleNups(page), { timeout: 20000 }).toHaveLength(5);

    await page.getByRole('combobox').filter({ hasText: 'Semua Jenis BMN' }).selectOption({ label: 'Tanah' });
    await expect.poll(() => visibleNups(page), { timeout: 20000 }).toEqual(['E2E-B-2']);
    await expect(page.getByText('Tanah Kantor').first()).toBeVisible();
  });

  test('sort by Nama A→Z reorders the rows server-side', async ({ page }) => {
    await page.goto(`${BASE}/bank-aset/daftar`);
    await expect(shell(page)).toBeVisible();
    await expect.poll(() => visibleNups(page), { timeout: 20000 }).toHaveLength(5);

    await page.getByRole('combobox').filter({ hasText: 'Terbaru diperbarui' }).selectOption('nama_asc');
    // Honda Vario < Laptop Dell < Printer Epson < Tanah Kantor < Toyota Avanza
    await expect
      .poll(async () => {
        const body = await page.locator('body').innerText();
        return body.indexOf('Honda Vario') < body.indexOf('Toyota Avanza');
      }, { timeout: 20000 })
      .toBe(true);
  });

  test('Detail opens the asset detail page with the real record', async ({ page }) => {
    await page.goto(`${BASE}/bank-aset/daftar`);
    await expect(shell(page)).toBeVisible();
    await page.getByPlaceholder('Cari nama/kode/NUP/merk...').fill('Toyota');
    await page.getByRole('button', { name: 'Terapkan' }).click();
    await expect.poll(() => visibleNups(page), { timeout: 20000 }).toEqual(['E2E-A-1']);

    // siman_aset.id is BIGSERIAL (no natural key) → reach detail by clicking,
    // not by deep-link. This also proves the list→detail link is wired.
    await page.getByRole('link', { name: 'Detail' }).first().click();
    await expect(page).toHaveURL(/\/bank-aset\/daftar\/\d+/);
    await expect(page.getByText('Detail Aset').first()).toBeVisible();
    await expect(page.getByText('Toyota Avanza').first()).toBeVisible();
    await expect(page.getByText('3.05.01.04.001').first()).toBeVisible(); // kode barang
  });
});

// ---------------------------------------------------------------------------
// Bank Aset — sebaran + QR generator (clears route-coverage debt with real drive)
// ---------------------------------------------------------------------------
test.describe('Bank Aset — sebaran + QR code', () => {
  test.use({ storageState: storageStatePath('validator_pusat') });

  test('sebaran aggregates assets per satker', async ({ page }) => {
    await page.goto(`${BASE}/bank-aset/sebaran`);
    await expect(shell(page)).toBeVisible();
    await expect(page.getByText('Sebaran Aset per Satker').first()).toBeVisible();
    // Real aggregation of the seeded rows — the satkers holding assets must appear.
    await expect(page.getByText('KEJAKSAAN NEGERI JAKARTA PUSAT').first()).toBeVisible({ timeout: 20000 });
    await expect(page.getByText('KEJAKSAAN NEGERI JAKARTA SELATAN').first()).toBeVisible();
  });

  test('QR generator selects assets and renders label previews', async ({ page }) => {
    await page.goto(`${BASE}/bank-aset/qrcode`);
    await expect(shell(page)).toBeVisible();
    await expect(page.getByText('Generator QR Code Aset').first()).toBeVisible();

    // Candidate assets load from the real list endpoint.
    await expect.poll(() => visibleNups(page), { timeout: 20000 }).not.toHaveLength(0);

    await page.getByRole('button', { name: 'Pilih semua' }).click();
    // Selecting drives the preview panel — real state, not just a visible button.
    await expect(page.getByText('Pratinjau Label QR Code').first()).toBeVisible({ timeout: 20000 });
    await expect(page.getByRole('button', { name: 'Cetak' })).toBeVisible();

    await page.getByRole('button', { name: 'Bersihkan' }).click();
    await expect(page.getByText('Pratinjau Label QR Code')).toHaveCount(0);
  });
});

// ---------------------------------------------------------------------------
// Dashboard
// (The old /dashboard/search "global search" page was deleted in the FE audit:
// it called a phantom endpoint — /api/v1/perlengkapan/search does not exist on
// the backend — and no UI ever linked to it.)
// ---------------------------------------------------------------------------
test.describe('Dashboard', () => {
  test.use({ storageState: storageStatePath('validator_pusat') });

  test('dashboard renders real stat cards', async ({ page }) => {
    await page.goto(`${BASE}/dashboard`);
    await expect(shell(page)).toBeVisible();
    await expect(page.getByText('Total Aset BMN').first()).toBeVisible({ timeout: 20000 });
    await expect(page.getByText('Kondisi Baik').first()).toBeVisible();
  });

  test('dashboard export endpoints respond server-side (#97: no FE caller yet)', async ({ request }) => {
    const token = await tokenFor(request, 'validator_pusat');
    for (const fmt of ['excel', 'pdf']) {
      const res = await request.get(`${API}/dashboard/perlengkapan/export/${fmt}`, {
        headers: { Authorization: `Bearer ${token}` },
      });
      // The endpoint is wired and must keep working even though the dashboard
      // UI offers no button for it (#97). Assert it is not a 404/5xx rot.
      expect(res.status(), `dashboard export/${fmt} status`).toBeLessThan(400);
      expect((await res.body()).length, `dashboard export/${fmt} body`).toBeGreaterThan(0);
    }
  });
});

// ---------------------------------------------------------------------------
// Analitik — roadmap create→list (real CRUD) + kodefikasi
// ---------------------------------------------------------------------------
test.describe('Analitik — roadmap + kodefikasi', () => {
  test.use({ storageState: storageStatePath('operator_a') });

  test('create an analisis via the form, then see it in the list and the backend', async ({ page, request }) => {
    // Unique title so the assertion cannot pass on a leftover row from a retry.
    const judul = `E2E Roadmap ${Date.now()}`;

    await page.goto(`${BASE}/analitik/roadmap/buat`);
    await expect(shell(page)).toBeVisible();
    await expect(page.getByText('Buat Analisis Kebutuhan').first()).toBeVisible();

    await page.getByRole('textbox').first().fill(judul);
    await page.getByRole('combobox').filter({ hasText: 'Pilih Kategori' }).selectOption('TIK');
    await page.getByRole('combobox').filter({ hasText: 'Sedang' }).selectOption('tinggi');
    await page.getByRole('spinbutton').fill('125000000');

    await page.getByRole('button', { name: 'Simpan' }).click();

    // The form navigates to the roadmap list after a successful create.
    await expect(page).toHaveURL(/\/analitik\/roadmap$/, { timeout: 20000 });
    await expect(page.getByText(judul).first()).toBeVisible({ timeout: 20000 });

    // Backend truth: the row really persisted with the values we typed.
    const token = await tokenFor(request, 'operator_a');
    const res = await request.get(`${API}/analisis?page=1&per_page=100`, {
      headers: { Authorization: `Bearer ${token}` },
    });
    expect(res.ok(), `GET /analisis -> ${res.status()}`).toBeTruthy();
    const body = await res.json();
    const rows: Array<{ judul: string; kategori: string; prioritas: string; estimasi_biaya: number | null }> =
      body.data ?? [];
    const created = rows.find((r) => r.judul === judul);
    expect(created, 'created analisis present in GET /analisis').toBeTruthy();
    expect(created!.kategori).toBe('TIK');
    expect(created!.prioritas).toBe('tinggi');
    expect(created!.estimasi_biaya).toBe(125000000);
  });

  test('roadmap list page mounts with the create entry point', async ({ page }) => {
    await page.goto(`${BASE}/analitik/roadmap`);
    await expect(shell(page)).toBeVisible();
    await expect(page.getByText('Buat Analisis Baru').first()).toBeVisible({ timeout: 20000 });
  });

  test('kodefikasi dashboard renders its mapping table', async ({ page }) => {
    await page.goto(`${BASE}/analitik/kodefikasi`);
    await expect(shell(page)).toBeVisible();
    // Column headers of the real mapping table (data may legitimately be empty:
    // mapping rows are produced by the #43 satker_code_map / kodefikasi pipeline).
    await expect(page.getByText('Kode Lama').first()).toBeVisible({ timeout: 20000 });
    await expect(page.getByText('Kode Standar Rujukan').first()).toBeVisible();
  });
});

// ---------------------------------------------------------------------------
// Notifikasi — per-user inbox, read lifecycle, cross-user isolation
// ---------------------------------------------------------------------------
test.describe('Notifikasi — inbox read lifecycle', () => {
  test.use({ storageState: storageStatePath('operator_a') });

  test('inbox lists this user rows only, and the unread filter hides the read one', async ({ page }) => {
    await page.goto(`${BASE}/notifikasi`);
    await expect(shell(page)).toBeVisible();
    await expect(page.getByRole('heading', { name: 'Notifikasi' }).first()).toBeVisible();

    await expect(page.getByText('E2E Notifikasi Satu').first()).toBeVisible({ timeout: 20000 });
    await expect(page.getByText('E2E Notifikasi Dua').first()).toBeVisible();
    await expect(page.getByText('E2E Notifikasi Terbaca').first()).toBeVisible();
    // Per-user isolation: operator_b's row must never appear here.
    await expect(page.getByText('E2E Notifikasi Operator B')).toHaveCount(0);

    // "Hanya yang belum dibaca" re-queries with unread_only=true.
    await page.getByRole('checkbox').check();
    await expect(page.getByText('E2E Notifikasi Terbaca')).toHaveCount(0, { timeout: 20000 });
    await expect(page.getByText('E2E Notifikasi Satu').first()).toBeVisible();
  });

  test('Tandai semua dibaca drives every row read in the backend', async ({ page, request }) => {
    const token = await tokenFor(request, 'operator_a');

    // GET /notifikasi/unread-count -> ApiResponse<UnreadCountResponse> = { data: { count } }
    const unreadCount = async () => {
      const res = await request.get(`${API}/notifikasi/unread-count`, {
        headers: { Authorization: `Bearer ${token}` },
      });
      expect(res.ok(), `unread-count -> ${res.status()}`).toBeTruthy();
      return (await res.json()).data.count as number;
    };

    await page.goto(`${BASE}/notifikasi`);
    await expect(shell(page)).toBeVisible();
    await expect(page.getByText('E2E Notifikasi Satu').first()).toBeVisible({ timeout: 20000 });

    await page.getByRole('button', { name: 'Tandai semua dibaca' }).click();

    // Backend truth: this user's inbox drains to zero unread.
    await expect.poll(unreadCount, { timeout: 20000 }).toBe(0);

    // …and operator_b's own unread row is untouched (mark_all_read is per-user).
    const tokenB = await tokenFor(request, 'operator_b');
    const resB = await request.get(`${API}/notifikasi/unread-count`, {
      headers: { Authorization: `Bearer ${tokenB}` },
    });
    expect(resB.ok(), `operator_b unread-count -> ${resB.status()}`).toBeTruthy();
    const unreadB = (await resB.json()).data.count as number;
    expect(unreadB, "operator_b's unread survives operator_a's mark-all").toBeGreaterThan(0);
  });

  test('a user cannot mark another user notification read (ownership enforced)', async ({ request }) => {
    // mark_as_read is keyed on (id, claims.user_id) — operator_b patching
    // operator_a's N1 must not flip it.
    const tokenB = await tokenFor(request, 'operator_b');
    const res = await request.patch(`${API}/notifikasi/${N1}/read`, {
      headers: { Authorization: `Bearer ${tokenB}` },
    });
    // Either an explicit rejection or a no-op — never a cross-user write.
    if (res.ok()) {
      const tokenA = await tokenFor(request, 'operator_a');
      const listed = await request.get(`${API}/notifikasi?limit=100&offset=0&unread_only=false`, {
        headers: { Authorization: `Bearer ${tokenA}` },
      });
      const rows: Array<{ id: string; read: boolean }> = (await listed.json()).data ?? [];
      const n1 = rows.find((r) => r.id === N1);
      // If N1 is still in operator_a's inbox it must NOT have been read by B.
      if (n1) expect(n1.read, 'N1 read-state after cross-user PATCH').toBe(false);
    } else {
      expect(res.status()).toBeGreaterThanOrEqual(400);
    }
  });
});

test.describe('Notifikasi — operator_b sees only its own row', () => {
  test.use({ storageState: storageStatePath('operator_b') });

  test('inbox is scoped to operator_b', async ({ page }) => {
    await page.goto(`${BASE}/notifikasi`);
    await expect(shell(page)).toBeVisible();
    await expect(page.getByText('E2E Notifikasi Operator B').first()).toBeVisible({ timeout: 20000 });
    await expect(page.getByText('E2E Notifikasi Satu')).toHaveCount(0);
    await expect(page.getByText('E2E Notifikasi Dua')).toHaveCount(0);
  });
});

// Reference the seeded ids that are asserted indirectly, so the constants stay
// honest if the fixture changes.
test('seeded notifikasi ids are the ones the fixture defines', async ({ request }) => {
  const token = await tokenFor(request, 'operator_a');
  const res = await request.get(`${API}/notifikasi?limit=100&offset=0&unread_only=false`, {
    headers: { Authorization: `Bearer ${token}` },
  });
  expect(res.ok()).toBeTruthy();
  const ids: string[] = ((await res.json()).data ?? []).map((r: { id: string }) => r.id);
  expect(ids).toEqual(expect.arrayContaining([N1, N2, N3]));
  expect(ids, "operator_a's inbox must not contain operator_b's row").not.toContain(N4);
});
