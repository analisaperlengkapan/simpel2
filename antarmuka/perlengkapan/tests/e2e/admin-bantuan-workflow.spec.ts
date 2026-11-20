/**
 * F-E2E E-5 — Bantuan (panduan/FAQ/helpdesk), admin workflow pages,
 * kebutuhan buat + laporan (exports), pakaian ukuran, and negative/RBAC
 * edges. Clears the last 9 route-coverage debt entries so the perlengkapan
 * gate can flip to blocking.
 *
 * Real-auth per-role storageState; asserts real UI state AND the backend
 * behind it. Unlike earlier specs, navigation here is CLICK-DRIVEN through
 * the sidebar wherever a link exists — the FE audit (commit 38015487) found
 * 12 broken links that goto()-only specs could never catch.
 *
 * Preconditions: seed-perlengkapan-workflow.sql (kebutuhan campaign 2026 +
 * seeded barang rows for laporan) + V002 baseline seed (ms_ukuran sizes).
 * The helpdesk tests create their own tickets (unique subjects) so every
 * test stays independent under fullyParallel.
 */
import { test, expect, type Page, type APIRequestContext } from '@playwright/test';
import {
  credsFor,
  apiLogin,
  storageStatePath,
  tokenFor,
  TEST_USERS,
  PERLENGKAPAN_API_URL,
} from './helpers/real-auth';

const BASE = '/perlengkapan/simpel';
const API = `${PERLENGKAPAN_API_URL}/api/v1/perlengkapan`;

/** Both route guards render this on a role mismatch (see guards-rbac.spec.ts). */
const FORBIDDEN = /Akses Ditolak/i;

/**
 * The authenticated app shell has mounted (not bounced to the portal login).
 *
 * `app_chrome.rs:46` renders the brand inside a sticky <header> as a <span>, and
 * `sidebar.rs:144` as a <div> — there is NO heading element anywhere in the
 * chrome, so `getByRole('heading', { name: 'SIMPEL' })` could never match. The
 * <header> scope is what keeps this off the login page, which mentions "Portal
 * SIMPEL" in body copy (`login.rs:39`) but renders no header. Same locator and
 * timeout as the specs that already pass — nav-access.spec.ts:46 and
 * pengelolaan-workflow.spec.ts:54.
 */
const shell = (page: Page) => page.locator('header').getByText('SIMPEL').first();

/** Expand a collapsed sidebar group, then click the real nav link by href. */
async function clickSidebarLink(page: Page, group: string, href: string) {
  const nav = page.locator('nav');
  await nav.getByRole('button', { name: group, exact: true }).click();
  await nav.locator(`a[href="${href}"]`).click();
}

// ---------------------------------------------------------------------------
// Bantuan — panduan + FAQ (static pages, reached by clicking the sidebar)
// ---------------------------------------------------------------------------
test.describe('Bantuan — panduan & FAQ', () => {
  test.use({ storageState: storageStatePath('operator_a') });

  test('panduan is reachable via sidebar and renders content', async ({ page }) => {
    await page.goto(`${BASE}/dashboard`);
    await expect(shell(page)).toBeVisible({ timeout: 20000 });
    await clickSidebarLink(page, 'Bantuan', `${BASE}/bantuan/panduan`);
    await expect(page).toHaveURL(new RegExp(`${BASE}/bantuan/panduan$`));
    await expect(page.getByText('Panduan Pengguna').first()).toBeVisible();
  });

  test('faq is reachable via sidebar and renders content', async ({ page }) => {
    await page.goto(`${BASE}/dashboard`);
    await expect(shell(page)).toBeVisible({ timeout: 20000 });
    await clickSidebarLink(page, 'Bantuan', `${BASE}/bantuan/faq`);
    await expect(page).toHaveURL(new RegExp(`${BASE}/bantuan/faq$`));
    await expect(
      page.getByText('Pertanyaan yang sering diajukan tentang SIMPEL').first(),
    ).toBeVisible();
    await expect(page.getByText('Bagaimana cara mengajukan kebutuhan BMN?').first()).toBeVisible();
  });
});

// ---------------------------------------------------------------------------
// Bantuan — helpdesk ticket lifecycle (reporter side)
// ---------------------------------------------------------------------------
test.describe('Helpdesk — reporter files and tracks a ticket', () => {
  test.use({ storageState: storageStatePath('operator_a') });

  test('empty subject is rejected client-side (server enforces too)', async ({ page }) => {
    await page.goto(`${BASE}/bantuan/helpdesk`);
    await expect(shell(page)).toBeVisible({ timeout: 20000 });
    await page.getByTestId('helpdesk-message').fill('pesan tanpa subjek');
    await page.getByTestId('helpdesk-submit').click();
    await expect(page.getByTestId('helpdesk-error')).toContainText('Subjek wajib diisi');
  });

  test('create ticket → appears open in Tiket Saya → comment thread works → scoped per user', async ({
    page,
    request,
  }) => {
    const subject = `E2E tiket operator ${Date.now()}`;
    await page.goto(`${BASE}/bantuan/helpdesk`);
    await expect(shell(page)).toBeVisible({ timeout: 20000 });

    await page.getByTestId('helpdesk-subject').fill(subject);
    await page.getByTestId('helpdesk-priority').selectOption('high');
    await page.getByTestId('helpdesk-message').fill('Deskripsi kendala E2E.');
    await page.getByTestId('helpdesk-submit').click();
    await expect(page.getByTestId('helpdesk-success')).toContainText(subject);

    // The new ticket lands in the list as `open`.
    const row = page
      .getByTestId('helpdesk-ticket-row')
      .filter({ hasText: subject })
      .first();
    await expect(row).toBeVisible();
    await expect(row).toHaveAttribute('data-status', 'open');
    const ticketId = await row.getAttribute('data-ticket-id');
    expect(ticketId, 'ticket row carries its id').toBeTruthy();

    // Comment thread: expand, see detail, post a comment, see it render.
    await row.getByTestId('ticket-comments-toggle').click();
    await expect(page.getByTestId('ticket-comments')).toBeVisible();
    await expect(page.getByTestId('ticket-detail')).toContainText(subject);
    await expect(page.getByTestId('comments-empty')).toBeVisible();
    await page.getByTestId('comment-input').fill('Komentar pertama dari pelapor.');
    await page.getByTestId('comment-submit').click();
    await expect(page.getByTestId('comment-row').first()).toContainText(
      'Komentar pertama dari pelapor.',
    );

    // Backend truth: the reporter owns it; another operator cannot see it.
    const tokenA = await tokenFor(request, 'operator_a');
    const mine = await request.get(`${API}/bantuan/tiket?limit=50&offset=0`, {
      headers: { Authorization: `Bearer ${tokenA}` },
    });
    expect(mine.ok()).toBeTruthy();
    const mineIds = (await mine.json()).data.map((t: { id: string }) => t.id);
    expect(mineIds).toContain(ticketId);

    const tokenB = await tokenFor(request, 'operator_b');
    const theirs = await request.get(`${API}/bantuan/tiket/${ticketId}`, {
      headers: { Authorization: `Bearer ${tokenB}` },
    });
    // Unreadable id → 404 (never 403, so existence is not confirmable).
    expect(theirs.status(), 'other operator must not read the ticket').toBe(404);
  });
});

// ---------------------------------------------------------------------------
// Bantuan — helpdesk staff flow (admin manages every ticket)
// ---------------------------------------------------------------------------
test.describe('Helpdesk — staff status management', () => {
  test.use({ storageState: storageStatePath('admin') });

  test('staff sees every ticket and drives a server-enforced status transition', async ({
    page,
    request,
  }) => {
    // Independent fixture: file a ticket as operator_b via the API.
    const subject = `E2E tiket staf ${Date.now()}`;
    const tokenB = await tokenFor(request, 'operator_b');
    const created = await request.post(`${API}/bantuan/tiket`, {
      headers: { Authorization: `Bearer ${tokenB}` },
      data: { subject, description: 'Perlu penanganan staf.', priority: 'urgent' },
    });
    expect(created.ok()).toBeTruthy();
    const ticketId = (await created.json()).data.id as string;

    // Staff UI: the admin's list is server-widened to ALL tickets.
    await page.goto(`${BASE}/bantuan/helpdesk`);
    await expect(shell(page)).toBeVisible({ timeout: 20000 });
    const row = page.locator(`[data-testid="helpdesk-ticket-row"][data-ticket-id="${ticketId}"]`);
    await expect(row).toBeVisible();
    await expect(row).toHaveAttribute('data-status', 'open');

    // Transition open → in_progress from the row's staff control.
    await row.getByTestId('staff-status-select').selectOption('in_progress');
    await expect(
      page.locator(`[data-testid="helpdesk-ticket-row"][data-ticket-id="${ticketId}"]`),
    ).toHaveAttribute('data-status', 'in_progress', { timeout: 15000 });

    // Backend truth: the reporter sees the new status too.
    const asReporter = await request.get(`${API}/bantuan/tiket/${ticketId}`, {
      headers: { Authorization: `Bearer ${tokenB}` },
    });
    expect(asReporter.ok()).toBeTruthy();
    expect((await asReporter.json()).data.status).toBe('in_progress');

    // Negative: a non-staff caller cannot transition (403) …
    const denied = await request.put(`${API}/bantuan/tiket/${ticketId}/status`, {
      headers: { Authorization: `Bearer ${tokenB}` },
      data: { status: 'closed' },
    });
    expect(denied.status()).toBe(403);

    // … and even staff cannot make an illegal transition (in_progress → in_progress).
    const tokenAdmin = await tokenFor(request, 'admin');
    const illegal = await request.put(`${API}/bantuan/tiket/${ticketId}/status`, {
      headers: { Authorization: `Bearer ${tokenAdmin}` },
      data: { status: 'in_progress' },
    });
    expect(illegal.status(), 'illegal transition must be rejected').toBeGreaterThanOrEqual(400);
  });
});

// ---------------------------------------------------------------------------
// Admin workflow pages — render real data for admin (reached by clicking)
// ---------------------------------------------------------------------------
test.describe('Admin — workflow config and monitoring', () => {
  test.use({ storageState: storageStatePath('admin') });

  test('konfigurasi workflow lists real definitions', async ({ page, request }) => {
    await page.goto(`${BASE}/dashboard`);
    await expect(shell(page)).toBeVisible({ timeout: 20000 });
    await clickSidebarLink(page, 'Admin', `${BASE}/admin/workflow`);
    await expect(page).toHaveURL(new RegExp(`${BASE}/admin/workflow$`));
    await expect(page.getByText('Konfigurasi Workflow').first()).toBeVisible();
    await expect(page.getByText('Definisi Workflow').first()).toBeVisible();

    const token = await tokenFor(request, 'admin');
    const defs = await request.get(`${API}/workflow/definitions`, {
      headers: { Authorization: `Bearer ${token}` },
    });
    expect(defs.ok(), 'workflow definitions endpoint backs the page').toBeTruthy();
  });

  test('monitoring workflow renders the metrics summary', async ({ page }) => {
    // #118 (monitoring slice) FIXED — the `test.fail` marker is gone because the
    // behaviour asserted below now holds. What was wrong:
    //   `get_metrics` queried `perlengkapan.kebutuhan_bmn`, a table NO migration creates, and
    //   `..._satker_aktivitas.aktivitas_id` / `.pengajuan_id`, two columns that table does not
    //   have. Every call 500'd, so `MonitoringContent` — which holds "Ringkasan Metrics" — never
    //   rendered while the outer page title did. That asymmetry is what this test observed.
    //   `/monitoring/active` was equally broken AND its caller looped over six hard-coded English
    //   state names that match nothing; the page requires BOTH calls to succeed.
    //
    // The page renders both blocks, so assert both: a metrics summary AND the active-workflow
    // panel. Asserting only the first would let a regression in `/monitoring/active` pass
    // unnoticed — and that endpoint is exactly the one that was silently dead.
    await page.goto(`${BASE}/dashboard`);
    await expect(shell(page)).toBeVisible({ timeout: 20000 });
    await clickSidebarLink(page, 'Admin', `${BASE}/admin/workflow-monitoring`);
    await expect(page).toHaveURL(new RegExp(`${BASE}/admin/workflow-monitoring$`));
    await expect(page.getByText('Monitoring Workflow').first()).toBeVisible();
    await expect(page.getByText('Ringkasan Metrics').first()).toBeVisible({ timeout: 20000 });

    // The stat cards carry real numbers, not blanks: a card rendering an empty
    // value would still satisfy a text-only check on its label.
    for (const label of ['Total Aktif', 'SLA Breaches', 'Mendekati SLA']) {
      await expect(page.getByText(label).first(), `${label} card`).toBeVisible();
    }
  });
});

// ---------------------------------------------------------------------------
// Negative/RBAC edges — operator must not see or reach admin surfaces
// ---------------------------------------------------------------------------
test.describe('Admin surfaces denied for operator', () => {
  test.use({ storageState: storageStatePath('operator_a') });

  test('sidebar hides the Admin group for a non-admin active role', async ({ page }) => {
    await page.goto(`${BASE}/dashboard`);
    await expect(shell(page)).toBeVisible({ timeout: 20000 });
    const nav = page.locator('nav');
    // Bantuan (same ADMINISTRASI section) stays visible…
    await expect(nav.getByRole('button', { name: 'Bantuan', exact: true })).toBeVisible();
    // …but the Admin group and its links are not rendered at all.
    await expect(nav.getByRole('button', { name: 'Admin', exact: true })).toHaveCount(0);
    await expect(nav.locator(`a[href="${BASE}/admin/workflow"]`)).toHaveCount(0);
  });

  test('direct URLs to the admin workflow pages are forbidden', async ({ page }) => {
    for (const path of [
      `${BASE}/admin/workflow`,
      `${BASE}/admin/workflow-monitoring`,
    ]) {
      await page.goto(path, { waitUntil: 'domcontentloaded' });
      await expect(page.getByText(FORBIDDEN).first()).toBeVisible({ timeout: 15000 });
    }
  });
});

// ---------------------------------------------------------------------------
// Kebutuhan BMN — buat (validator_pusat campaign) + laporan with exports
// ---------------------------------------------------------------------------
test.describe('Kebutuhan BMN — buat & laporan', () => {
  test.use({ storageState: storageStatePath('validator_pusat') });

  test('validator_pusat creates a campaign through the form; operator is 403', async ({
    page,
    request,
  }) => {
    const nama = `E2E Kampanye Kebutuhan ${Date.now()}`;
    await page.goto(`${BASE}/dashboard`);
    await expect(shell(page)).toBeVisible({ timeout: 20000 });
    await clickSidebarLink(page, 'Kebutuhan BMN', `${BASE}/kebutuhan-bmn/buat`);
    await expect(page).toHaveURL(new RegExp(`${BASE}/kebutuhan-bmn/buat$`));

    await page.getByPlaceholder('Contoh: Pengajuan Kebutuhan BMN Tahun 2025').fill(nama);
    await page.locator('input[type="date"]').nth(0).fill('2026-08-01');
    await page.locator('input[type="date"]').nth(1).fill('2026-12-31');
    await page.getByRole('button', { name: 'Simpan' }).click();

    // Post-create redirect exercises the navigate() base fix from the FE audit.
    await expect(page).toHaveURL(new RegExp(`${BASE}/kebutuhan-bmn/daftar$`), {
      timeout: 20000,
    });
    await expect(page.getByText(nama).first()).toBeVisible({ timeout: 20000 });

    // Backend truth + RBAC negative.
    const tokenPusat = await tokenFor(request, 'validator_pusat');
    const list = await request.get(
      `${API}/kebutuhan-bmn/pengajuan?search=${encodeURIComponent(nama)}`,
      { headers: { Authorization: `Bearer ${tokenPusat}` } },
    );
    expect(list.ok()).toBeTruthy();
    expect(JSON.stringify(await list.json())).toContain(nama);

    const tokenOp = await tokenFor(request, 'operator_a');
    const deniedCreate = await request.post(`${API}/kebutuhan-bmn/pengajuan`, {
      headers: { Authorization: `Bearer ${tokenOp}` },
      data: {
        nama: 'E2E harus ditolak',
        tahun: 2026,
        tgl_mulai: '2026-08-01',
        tgl_selesai: '2026-12-31',
        pilihan_satker: 'semua',
      },
    });
    expect(deniedCreate.status(), 'operator must not create a campaign').toBe(403);
  });

  test('laporan shows seeded recap rows and both exports download real bytes', async ({
    page,
  }) => {
    await page.goto(`${BASE}/dashboard`);
    await expect(shell(page)).toBeVisible({ timeout: 20000 });
    await clickSidebarLink(page, 'Kebutuhan BMN', `${BASE}/kebutuhan-bmn/laporan`);
    await expect(page).toHaveURL(new RegExp(`${BASE}/kebutuhan-bmn/laporan$`));

    // validator_pusat sees all satkers → the seeded barang rows must render.
    await expect(page.getByText('E2E Barang Seed A').first()).toBeVisible({ timeout: 20000 });

    for (const [label, ext] of [
      ['Export XLSX', 'xlsx'],
      ['Export PDF', 'pdf'],
    ] as const) {
      const downloadPromise = page.waitForEvent('download');
      await page.getByRole('button', { name: label }).click();
      const download = await downloadPromise;
      expect(download.suggestedFilename()).toMatch(new RegExp(`\\.${ext}$`));
      const stream = await download.createReadStream();
      let bytes = 0;
      for await (const chunk of stream) bytes += (chunk as Buffer).length;
      expect(bytes, `${label} must not be empty`).toBeGreaterThan(0);
    }
  });
});

// ---------------------------------------------------------------------------
// Pakaian Dinas — ukuran (current-user sizes persist round-trip)
// ---------------------------------------------------------------------------
test.describe('Pakaian Dinas — ukuran pegawai', () => {
  test.use({ storageState: storageStatePath('operator_a') });

  // Locator note (#117): `getByText('Isi Ukuran')` is NOT usable as proof that the form card
  // rendered. Playwright's default text matching is substring + case-insensitive, so it also
  // matches the PageLayout description "Isi ukuran pakaian dinas Anda untuk keperluan
  // pengajuan", which renders unconditionally, OUTSIDE the <Suspense>. Assert on the form itself.
  const sizeForm = (page: Page) => page.getByTestId('ukuran-profil-form');
  const kolom = (page: Page) => page.getByTestId('ukuran-profil-kolom');

  // Since V019 the page offers one field per active specification (PDH shoes and PDL shoes are
  // different fields), so nothing below counts "three selects".
  interface ProfilItem {
    spesifikasi_id: string;
    jenis_pakaian_nama: string;
    spesifikasi_nama: string;
    ukuran_group: string;
    tipe: string;
    ukuran: string | null;
  }
  async function profilServer(request: APIRequestContext, token: string): Promise<ProfilItem[]> {
    const res = await request.get(`${API}/pakaian-dinas/ukuran-profil`, {
      headers: { Authorization: `Bearer ${token}` },
    });
    expect(res.ok(), await res.text()).toBeTruthy();
    return (await res.json()).data as ProfilItem[];
  }

  test('operator saves personal sizes and they persist across reload', async ({ page, request }) => {
    await page.goto(`${BASE}/dashboard`);
    await expect(shell(page)).toBeVisible({ timeout: 20000 });
    await clickSidebarLink(page, 'Pakaian Dinas', `${BASE}/pakaian-dinas/ukuran`);
    await expect(page).toHaveURL(new RegExp(`${BASE}/pakaian-dinas/ukuran$`));
    await expect(page.getByText('Ukuran Pakaian Dinas').first()).toBeVisible();
    await expect(sizeForm(page)).toBeVisible({ timeout: 20000 });

    // The PDH garments simpelv1 has always carried are there, each as its own field.
    const pdh = page.locator('[data-testid="ukuran-profil-jenis"][data-jenis="PDH"]');
    for (const nama of ['Pakaian Dinas', 'Celana', 'Sepatu Dinas']) {
      await expect(pdh.locator(`[data-spesifikasi="${nama}"]`), `PDH ${nama}`).toHaveCount(1);
    }

    // Pick the first real size in every dropdown (options come from ms_ukuran) and fill the text
    // field (nama dada) of the PDL group.
    const selects = sizeForm(page).locator('select');
    const total = await selects.count();
    expect(total, 'one dropdown per choice-type specification').toBeGreaterThanOrEqual(3);
    const chosen: string[] = [];
    for (let i = 0; i < total; i++) {
      const values = await selects
        .nth(i)
        .locator('option')
        .evaluateAll((opts) =>
          (opts as HTMLOptionElement[]).map((o) => o.value).filter((v) => v !== ''),
        );
      expect(values.length, `select ${i} offers seeded sizes`).toBeGreaterThan(0);
      await selects.nth(i).selectOption(values[0]);
      chosen.push(values[0]);
    }
    const teks = sizeForm(page).locator('input[type="text"]');
    const jumlahTeks = await teks.count();
    for (let i = 0; i < jumlahTeks; i++) await teks.nth(i).fill(`E2E NAMA ${i + 1}`);
    // `with_hijab` belongs to the same record and had its own bug (saving sizes used to
    // reset it), so drive it here instead of leaving it at the default.
    await page.getByTestId('ukuran-profil-hijab').check();
    await page.getByTestId('simpan-ukuran-profil').click();
    await expect(page.getByTestId('ukuran-profil-sukses')).toContainText('Ukuran berhasil disimpan!', {
      timeout: 15000,
    });

    // The server holds exactly what the screen showed.
    const token = await tokenFor(request, 'operator_a');
    const server = await profilServer(request, token);
    // Compared sorted: the DOM groups by jenis, the server lists by its own order.
    expect(
      server
        .filter((s) => s.tipe === 'PILIHAN')
        .map((s) => s.ukuran)
        .sort(),
    ).toEqual([...chosen].sort());

    // Round-trip: a fresh load must show the persisted values.
    await page.reload({ waitUntil: 'domcontentloaded' });
    await expect(sizeForm(page)).toBeVisible({ timeout: 20000 });
    for (let i = 0; i < total; i++) {
      await expect(sizeForm(page).locator('select').nth(i)).toHaveValue(chosen[i], { timeout: 15000 });
    }
    for (let i = 0; i < jumlahTeks; i++) {
      await expect(sizeForm(page).locator('input[type="text"]').nth(i)).toHaveValue(`E2E NAMA ${i + 1}`);
    }
    await expect(page.getByTestId('ukuran-profil-hijab')).toBeChecked({ timeout: 15000 });
  });

  // Every field is optional since V019 (a person may have no hat size, or no chest name), so the old
  // "wajib diisi semua" refusal is gone. What replaces it: only what changed is sent, saving
  // nothing says so without a request, and clearing a field removes the stored value.
  test('an untouched form sends nothing; clearing a field removes the stored size', async ({
    page,
    request,
  }) => {
    const token = await tokenFor(request, 'operator_a');
    // Known starting point: one PDH baju size stored.
    const awal = await profilServer(request, token);
    const baju = awal.find((s) => s.jenis_pakaian_nama === 'PDH' && s.spesifikasi_nama === 'Pakaian Dinas');
    expect(baju, 'PDH / Pakaian Dinas is seeded').toBeTruthy();
    const seeded = await request.put(`${API}/pakaian-dinas/ukuran-profil`, {
      headers: { Authorization: `Bearer ${token}` },
      data: { ukuran: [{ spesifikasi_id: baju!.spesifikasi_id, ukuran: 'L' }] },
    });
    expect(seeded.ok(), await seeded.text()).toBeTruthy();

    const puts: string[] = [];
    await page.route('**/pakaian-dinas/ukuran-profil', async (route) => {
      if (route.request().method() === 'PUT') puts.push(route.request().postData() ?? '');
      await route.continue();
    });
    await page.goto(`${BASE}/pakaian-dinas/ukuran`);
    await expect(sizeForm(page)).toBeVisible({ timeout: 20000 });
    const kolomBaju = page.locator(
      '[data-testid="ukuran-profil-jenis"][data-jenis="PDH"] [data-spesifikasi="Pakaian Dinas"]',
    );
    await expect(kolomBaju.locator('select')).toHaveValue('L');

    await page.getByTestId('simpan-ukuran-profil').click();
    await expect(page.getByTestId('ukuran-profil-sukses')).toContainText('Tidak ada perubahan');
    expect(puts, 'an untouched form must not send a request').toHaveLength(0);

    await kolomBaju.locator('select').selectOption('');
    await page.getByTestId('simpan-ukuran-profil').click();
    await expect(page.getByTestId('ukuran-profil-sukses')).toContainText('Ukuran berhasil disimpan!', {
      timeout: 15000,
    });
    expect(puts, 'exactly one request, carrying only the changed field').toHaveLength(1);
    expect(JSON.parse(puts[0]).ukuran).toEqual([{ spesifikasi_id: baju!.spesifikasi_id, ukuran: '' }]);

    const sesudah = await profilServer(request, token);
    expect(
      sesudah.find((s) => s.spesifikasi_id === baju!.spesifikasi_id)?.ukuran,
      'the empty value deleted the stored size',
    ).toBeNull();
  });

  // The form used to be drawn as soon as the size LISTS arrived, while the employee's STORED sizes
  // were still on their way. When that second answer came in after the operator had already chosen,
  // it replaced the choice with the stored value, and "Simpan" then saved the old sizes under a
  // success message. The form is now not drawn until ALL answers are in; this holds the stored-sizes
  // answer back so the order of arrival is fixed instead of lucky.
  test('the form is not offered until the stored sizes arrive, then shows them and saves what is picked', async ({
    page,
    request,
  }) => {
    const token = await tokenFor(request, 'operator_a');
    const auth = { Authorization: `Bearer ${token}` };
    const awal = await profilServer(request, token);
    const baju = awal.find((s) => s.jenis_pakaian_nama === 'PDH' && s.spesifikasi_nama === 'Pakaian Dinas')!;
    const celana = awal.find((s) => s.jenis_pakaian_nama === 'PDH' && s.spesifikasi_nama === 'Celana')!;
    const seeded = await request.put(`${API}/pakaian-dinas/ukuran-profil`, {
      headers: auth,
      data: {
        ukuran: [
          { spesifikasi_id: baju.spesifikasi_id, ukuran: 'L' },
          { spesifikasi_id: celana.spesifikasi_id, ukuran: '30' },
        ],
      },
    });
    expect(seeded.ok(), await seeded.text()).toBeTruthy();

    // Hold ONLY the stored-sizes read for 4 s (a slow link); the size lists are not held.
    await page.route('**/pakaian-dinas/ukuran-profil', async (route) => {
      if (route.request().method() !== 'GET') return route.continue();
      await new Promise((resolve) => setTimeout(resolve, 4000));
      await route.continue();
    });
    const storedAnswered = page.waitForResponse(
      (r) => r.url().endsWith('/pakaian-dinas/ukuran-profil') && r.request().method() === 'GET',
    );
    await page.goto(`${BASE}/pakaian-dinas/ukuran`);

    // While the answer is held there is no form to choose in (and so nothing to be overwritten).
    await page.waitForTimeout(1500);
    await expect(sizeForm(page), 'no form before the stored sizes are known').toHaveCount(0);
    await storedAnswered;
    await expect(sizeForm(page)).toBeVisible({ timeout: 20000 });

    const kolomBaju = page.locator(
      '[data-testid="ukuran-profil-jenis"][data-jenis="PDH"] [data-spesifikasi="Pakaian Dinas"] select',
    );
    const kolomCelana = page.locator(
      '[data-testid="ukuran-profil-jenis"][data-jenis="PDH"] [data-spesifikasi="Celana"] select',
    );
    await expect(kolomBaju).toHaveValue('L');
    await expect(kolomCelana).toHaveValue('30');

    // And what the screen shows is what gets saved.
    await kolomBaju.selectOption('XS');
    await page.getByTestId('simpan-ukuran-profil').click();
    await expect(page.getByTestId('ukuran-profil-sukses')).toContainText('Ukuran berhasil disimpan!', {
      timeout: 15000,
    });
    const sesudah = await profilServer(request, token);
    expect(sesudah.find((s) => s.spesifikasi_id === baju.spesifikasi_id)?.ukuran).toBe('XS');
    expect(sesudah.find((s) => s.spesifikasi_id === celana.spesifikasi_id)?.ukuran).toBe('30');
  });
});
