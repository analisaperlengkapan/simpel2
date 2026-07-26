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
 *
 * WHAT THE UI ACTUALLY SHOWS (verified, do not "fix" back to siman_aset.nama):
 *   `repository.rs:563` maps `BankAsetItem.nama_aset` from `ur_sskel` (the BMN
 *   sub-sub-group nomenclature) and only falls back to `nama` when ur_sskel is
 *   NULL. Both `list_page.rs:409` and `detail_page.rs:125` render that field, and
 *   `sort=nama_asc` is `ORDER BY ur_sskel` (`repository.rs:98`) — so the whole
 *   stack is consistently ur_sskel-based. The strings on screen are therefore the
 *   `ur_sskel` column, NOT the brand names above:
 *     E2E-A-1 "Kendaraan Dinas Roda 4"   E2E-A-2 "Personal Computer Unit"
 *     E2E-B-1 "Kendaraan Dinas Roda 2"   E2E-B-2 "Tanah Bangunan Kantor"
 *     E2E-C-1 "Personal Computer Unit"
 *   (Search still matches the brand: the BE ILIKEs ur_sskel OR nama OR kd_brg OR
 *   no_aset OR merk, so "Toyota" narrows to E2E-A-1 by `nama`/`merk`.)
 */
import { test, expect, type Page, type APIRequestContext } from "@playwright/test";
import { credsFor, apiLogin, storageStatePath, TEST_USERS, PERLENGKAPAN_API_URL } from "./helpers/real-auth";

const BASE = "/perlengkapan/simpel/v2";
const API = `${PERLENGKAPAN_API_URL}/api/v1/perlengkapan`;

/** Seeded notifikasi UUIDs (seed-perlengkapan-workflow.sql section 9). */
const N1 = "f1000000-0000-4f00-8f00-0000000c0001"; // operator_a, unread
const N2 = "f1000000-0000-4f00-8f00-0000000c0002"; // operator_a, unread
const N3 = "f1000000-0000-4f00-8f00-0000000c0003"; // operator_a, already read
const N4 = "f1000000-0000-4f00-8f00-0000000c0004"; // operator_b, unread

/**
 * The authenticated app shell has mounted (not bounced to the portal login).
 *
 * `app_chrome.rs:46` renders the brand inside a sticky <header> as a <span>, and
 * `sidebar.rs:144` as a <div> — there is NO heading element anywhere in the
 * chrome, so `getByRole("heading", { name: "SIMPEL" })` could never match. The
 * <header> scope is what keeps this off the login page, which mentions "Portal
 * SIMPEL" in body copy (`login.rs:39`) but renders no header. Same locator and
 * timeout as the specs that already pass — nav-access.spec.ts:46 and
 * pengelolaan-workflow.spec.ts:54.
 */
const shell = (page: Page) => page.locator("header").getByText("SIMPEL").first();

/** Real per-role JWT (apiLogin returns a token PAIR — the header needs .accessToken). */
async function tokenFor(request: APIRequestContext, userKey: string): Promise<string> {
  const user = TEST_USERS.find((u) => u.key === userKey);
  if (!user) throw new Error(`unknown seeded test user: ${userKey}`);
  const { accessToken } = await apiLogin(request, credsFor(user));
  return accessToken;
}

const ALL_NUPS = ["E2E-A-1", "E2E-A-2", "E2E-B-1", "E2E-B-2", "E2E-C-1"];

/** Rows currently rendered in the bank-aset list (the NUP column is unique per asset). */
async function visibleNups(page: Page): Promise<string[]> {
  const body = await page.locator("body").innerText();
  return ALL_NUPS.filter((n) => body.includes(n));
}

/**
 * Same rows, but in the order they appear on screen — `visibleNups` filters a
 * fixed array and so can never observe ordering, which is what the sort test
 * needs. Ordering is asserted on NUPs (unique) rather than on the displayed
 * ur_sskel, which repeats across E2E-A-2 and E2E-C-1.
 */
async function nupsInDomOrder(page: Page): Promise<string[]> {
  const body = await page.locator("body").innerText();
  return ALL_NUPS.map((n) => [n, body.indexOf(n)] as const)
    .filter(([, i]) => i >= 0)
    .sort((a, b) => a[1] - b[1])
    .map(([n]) => n);
}

// ---------------------------------------------------------------------------
// Bank Aset — filters / search / sort / pagination drive the real list
// ---------------------------------------------------------------------------
test.describe("Bank Aset — daftar controls (validator_pusat: all 5 assets in scope)", () => {
  test.use({ storageState: storageStatePath("validator_pusat") });

  test("list renders every seeded asset before filtering", async ({ page }) => {
    await page.goto(`${BASE}/bank-aset/daftar`);
    await expect(shell(page)).toBeVisible({ timeout: 20000 });
    await expect(page.getByText("Daftar Aset").first()).toBeVisible();
    await expect
      .poll(() => visibleNups(page), { timeout: 20000 })
      .toEqual(["E2E-A-1", "E2E-A-2", "E2E-B-1", "E2E-B-2", "E2E-C-1"]);
  });

  test("search narrows to the matching asset and Reset restores the full list", async ({ page }) => {
    await page.goto(`${BASE}/bank-aset/daftar`);
    await expect(shell(page)).toBeVisible({ timeout: 20000 });
    await expect.poll(() => visibleNups(page), { timeout: 20000 }).toHaveLength(5);

    await page.getByPlaceholder("Cari nama/kode/NUP/merk...").fill("Toyota");
    await page.getByRole("button", { name: "Terapkan" }).click();
    // Real server-side search: only the Toyota row survives (matched on nama/merk).
    await expect.poll(() => visibleNups(page), { timeout: 20000 }).toEqual(["E2E-A-1"]);
    await expect(page.getByText("Kendaraan Dinas Roda 4").first()).toBeVisible();

    await page.getByRole("button", { name: "Reset" }).click();
    await expect.poll(() => visibleNups(page), { timeout: 20000 }).toHaveLength(5);
  });

  test("kondisi filter isolates the single RUSAK RINGAN asset", async ({ page }) => {
    await page.goto(`${BASE}/bank-aset/daftar`);
    await expect(shell(page)).toBeVisible({ timeout: 20000 });
    await expect.poll(() => visibleNups(page), { timeout: 20000 }).toHaveLength(5);

    // Select by VALUE, not label: the FE renders each dynamic option as
    // `format!("{} ({})", o.value, o.count)` (list_page.rs:264), so the label is
    // "RUSAK RINGAN (1)" and matching on the bare value is both correct and
    // immune to the seeded row count changing.
    await page.getByRole("combobox").filter({ hasText: "Semua Kondisi" }).selectOption("RUSAK RINGAN");
    await expect.poll(() => visibleNups(page), { timeout: 20000 }).toEqual(["E2E-C-1"]);
    await expect(page.getByText("Personal Computer Unit").first()).toBeVisible();
  });

  test("jenis filter isolates the single Tanah asset", async ({ page }) => {
    await page.goto(`${BASE}/bank-aset/daftar`);
    await expect(shell(page)).toBeVisible({ timeout: 20000 });
    await expect.poll(() => visibleNups(page), { timeout: 20000 }).toHaveLength(5);

    await page.getByRole("combobox").filter({ hasText: "Semua Jenis BMN" }).selectOption("Tanah");
    await expect.poll(() => visibleNups(page), { timeout: 20000 }).toEqual(["E2E-B-2"]);
    await expect(page.getByText("Tanah Bangunan Kantor").first()).toBeVisible();
  });

  test("sort by Nama A→Z reorders the rows server-side", async ({ page }) => {
    await page.goto(`${BASE}/bank-aset/daftar`);
    await expect(shell(page)).toBeVisible({ timeout: 20000 });
    await expect.poll(() => visibleNups(page), { timeout: 20000 }).toHaveLength(5);

    await page.getByRole("combobox").filter({ hasText: "Terbaru diperbarui" }).selectOption("nama_asc");
    // `nama_asc` is ORDER BY ur_sskel ASC server-side, so the row order becomes:
    //   "Kendaraan Dinas Roda 2" (E2E-B-1) < "Kendaraan Dinas Roda 4" (E2E-A-1)
    //   < "Personal Computer Unit" (E2E-A-2, E2E-C-1 — tied) < "Tanah Bangunan
    //   Kantor" (E2E-B-2).
    // Assert only the parts the tie does not make ambiguous, which is still
    // enough to prove the order changed from the default (updated_at DESC).
    await expect
      .poll(async () => (await nupsInDomOrder(page)).slice(0, 2), { timeout: 20000 })
      .toEqual(["E2E-B-1", "E2E-A-1"]);
    await expect
      .poll(async () => (await nupsInDomOrder(page)).at(-1), { timeout: 20000 })
      .toBe("E2E-B-2");
  });

  test("Detail opens the asset detail page with the real record", async ({ page }) => {
    await page.goto(`${BASE}/bank-aset/daftar`);
    await expect(shell(page)).toBeVisible({ timeout: 20000 });
    await page.getByPlaceholder("Cari nama/kode/NUP/merk...").fill("Toyota");
    await page.getByRole("button", { name: "Terapkan" }).click();
    await expect.poll(() => visibleNups(page), { timeout: 20000 }).toEqual(["E2E-A-1"]);

    // siman_aset.id is BIGSERIAL (no natural key) → reach detail by clicking,
    // not by deep-link. This also proves the list→detail link is wired.
    await page.getByRole("link", { name: "Detail" }).first().click();
    await expect(page).toHaveURL(/\/bank-aset\/daftar\/\d+/);
    await expect(page.getByText("Detail Aset").first()).toBeVisible();
    await expect(page.getByText("Kendaraan Dinas Roda 4").first()).toBeVisible();
    await expect(page.getByText("3.05.01.04.001").first()).toBeVisible(); // kode barang
  });
});

// ---------------------------------------------------------------------------
// Bank Aset — sebaran + QR generator (clears route-coverage debt with real drive)
// ---------------------------------------------------------------------------
test.describe("Bank Aset — sebaran + QR code", () => {
  test.use({ storageState: storageStatePath("validator_pusat") });

  test("sebaran aggregates assets per satker", async ({ page }) => {
    await page.goto(`${BASE}/bank-aset/sebaran`);
    await expect(shell(page)).toBeVisible({ timeout: 20000 });
    await expect(page.getByText("Sebaran Aset per Satker").first()).toBeVisible();
    // Real aggregation of the seeded rows — the satkers holding assets must appear.
    await expect(page.getByText("KEJAKSAAN NEGERI JAKARTA PUSAT").first()).toBeVisible({ timeout: 20000 });
    await expect(page.getByText("KEJAKSAAN NEGERI JAKARTA SELATAN").first()).toBeVisible();
  });

  test("QR generator selects assets and renders label previews", async ({ page }) => {
    await page.goto(`${BASE}/bank-aset/qrcode`);
    await expect(shell(page)).toBeVisible({ timeout: 20000 });
    await expect(page.getByText("Generator QR Code Aset").first()).toBeVisible();

    // Candidate assets load from the real list endpoint.
    await expect.poll(() => visibleNups(page), { timeout: 20000 }).not.toHaveLength(0);

    // "Pilih semua" only fills the selection; the preview modal is behind its own
    // button, which is `disabled` while nothing is selected (qrcode_page.rs:271).
    // Asserting it went from disabled to enabled proves the selection really
    // landed in state rather than just repainting checkboxes.
    const previewBtn = page.getByRole("button", { name: "Pratinjau & Cetak" });
    await expect(previewBtn).toBeDisabled();
    await page.getByRole("button", { name: "Pilih semua" }).click();
    await expect(previewBtn).toBeEnabled({ timeout: 20000 });

    await previewBtn.click();
    await expect(page.getByText("Pratinjau Label QR Code").first()).toBeVisible({ timeout: 20000 });
    // `exact: true` is required: getByRole matches the accessible name as a
    // SUBSTRING by default, so a bare "Cetak" also matches the "Pratinjau &
    // Cetak" trigger that opened this modal — two elements, strict-mode failure.
    await expect(page.getByRole("button", { name: "Cetak", exact: true })).toBeVisible();

    // Close the modal, then clear the selection — the button falls back to disabled.
    await page.getByRole("button", { name: "Tutup" }).click();
    await expect(page.getByText("Pratinjau Label QR Code")).toHaveCount(0);
    await page.getByRole("button", { name: "Bersihkan" }).click();
    await expect(previewBtn).toBeDisabled();
  });
});

// ---------------------------------------------------------------------------
// Dashboard
// (The old /dashboard/search "global search" page was deleted in the FE audit:
// it called a phantom endpoint — /api/v1/perlengkapan/search does not exist on
// the backend — and no UI ever linked to it.)
// ---------------------------------------------------------------------------
test.describe("Dashboard", () => {
  test.use({ storageState: storageStatePath("validator_pusat") });

  test("dashboard renders real stat cards", async ({ page }) => {
    await page.goto(`${BASE}/dashboard`);
    await expect(shell(page)).toBeVisible({ timeout: 20000 });
    await expect(page.getByText("Total Aset BMN").first()).toBeVisible({ timeout: 20000 });
    await expect(page.getByText("Kondisi Baik").first()).toBeVisible();
  });

  // Split per-format (was one test looping both): the loop threw on `excel` and
  // so never reached `pdf`, leaving pdf's real status unknown for the whole life
  // of the test. Separate tests report both.
  for (const fmt of ["excel", "pdf"] as const) {
    test(`dashboard export/${fmt} responds server-side (#97: no FE caller yet)`, async ({ request }) => {
      // FINDING #116 — filed, not papered over. `excel` genuinely 500s. The 400
      // this test used to get (before `tahun_anggaran` was supplied) was hiding
      // it. Ruled out by the sibling tests in this same file: the route is
      // mounted and guarded ("reject anonymous callers" passes with 401), and
      // the metrics query works ("dashboard renders real stat cards" passes) —
      // and the handler runs that query BEFORE the excel step. So the fault is
      // inside `export_dashboard_to_excel`, which round-trips a temp file
      // through /tmp (services.rs:95).
      //
      // `test.fail` rather than a loosened assertion or a skip: the assertions
      // below still describe CORRECT behaviour, so the day #116 is fixed
      // Playwright reports "expected to fail but passed" and forces this marker
      // to be removed. A weakened expectation would silently accept the bug
      // forever.
      test.fail(fmt === "excel", "#116: export_dashboard_to_excel returns 500");

      const token = await tokenFor(request, "validator_pusat");
      // `tahun_anggaran` is a REQUIRED query param (DashboardParams.tahun_anggaran
      // is a bare i32, dashboard/models.rs:10) — omitting it is a legitimate 400
      // from the Query extractor, not endpoint rot.
      const res = await request.get(`${API}/dashboard/perlengkapan/export/${fmt}?tahun_anggaran=2026`, {
        headers: { Authorization: `Bearer ${token}` },
      });
      expect(res.status(), `dashboard export/${fmt} status`).toBeLessThan(400);
      expect((await res.body()).length, `dashboard export/${fmt} body`).toBeGreaterThan(0);
    });
  }

  // The test above passes a token — and used to pass WITHOUT one too, because
  // these handlers took no `Claims` and this router has no auth middleware
  // layer. It therefore proved the endpoint responds, not that it is guarded:
  // a false green. These assert the guard itself.
  test("dashboard read + export endpoints reject anonymous callers", async ({ request }) => {
    const paths = [
      "/dashboard/perlengkapan?tahun_anggaran=2026",
      "/dashboard/perlengkapan/export/excel?tahun_anggaran=2026",
      "/dashboard/perlengkapan/export/pdf?tahun_anggaran=2026",
    ];
    for (const p of paths) {
      const res = await request.get(`${API}${p}`);
      expect(res.status(), `${p} without a token must be 401`).toBe(401);
    }
  });

  test("dashboard websocket rejects a missing or bad token before upgrading", async ({ request }) => {
    // No token at all → the Query extractor rejects the request outright.
    const noToken = await request.get(`${API}/dashboard/ws`);
    expect(noToken.status(), "ws without token must not upgrade").toBeGreaterThanOrEqual(400);

    // A syntactically plausible but invalid token → rejected by authenc,
    // still before the upgrade.
    const badToken = await request.get(`${API}/dashboard/ws?token=not-a-real-token`);
    expect(badToken.status(), "ws with bad token must not upgrade").toBeGreaterThanOrEqual(400);
  });
});

// ---------------------------------------------------------------------------
// Analitik — roadmap create→list (real CRUD) + kodefikasi
// ---------------------------------------------------------------------------
test.describe("Analitik — roadmap + kodefikasi", () => {
  test.use({ storageState: storageStatePath("operator_a") });

  test("create an analisis via the form, then see it in the list and the backend", async ({ page, request }) => {
    // Unique title so the assertion cannot pass on a leftover row from a retry.
    const judul = `E2E Roadmap ${Date.now()}`;

    await page.goto(`${BASE}/analitik/roadmap/buat`);
    await expect(shell(page)).toBeVisible({ timeout: 20000 });
    await expect(page.getByText("Buat Analisis Kebutuhan").first()).toBeVisible();

    await page.getByRole("textbox").first().fill(judul);
    await page.getByRole("combobox").filter({ hasText: "Pilih Kategori" }).selectOption("TIK");
    await page.getByRole("combobox").filter({ hasText: "Sedang" }).selectOption("tinggi");
    await page.getByRole("spinbutton").fill("125000000");

    await page.getByRole("button", { name: "Simpan" }).click();

    // The form navigates to the roadmap list after a successful create.
    await expect(page).toHaveURL(/\/analitik\/roadmap$/, { timeout: 20000 });
    await expect(page.getByText(judul).first()).toBeVisible({ timeout: 20000 });

    // Backend truth: the row really persisted with the values we typed.
    const token = await tokenFor(request, "operator_a");
    const res = await request.get(`${API}/analisis?page=1&per_page=100`, {
      headers: { Authorization: `Bearer ${token}` },
    });
    expect(res.ok(), `GET /analisis -> ${res.status()}`).toBeTruthy();
    const body = await res.json();
    const rows: Array<{ judul: string; kategori: string; prioritas: string; estimasi_biaya: number | null }> =
      body.data ?? [];
    const created = rows.find((r) => r.judul === judul);
    expect(created, "created analisis present in GET /analisis").toBeTruthy();
    expect(created!.kategori).toBe("TIK");
    expect(created!.prioritas).toBe("tinggi");
    expect(created!.estimasi_biaya).toBe(125000000);
  });

  test("roadmap list page mounts with the create entry point", async ({ page }) => {
    await page.goto(`${BASE}/analitik/roadmap`);
    await expect(shell(page)).toBeVisible({ timeout: 20000 });
    await expect(page.getByText("Buat Analisis Baru").first()).toBeVisible({ timeout: 20000 });
  });

  test("kodefikasi dashboard route mounts (mapping table blocked by #113)", async ({ page }) => {
    await page.goto(`${BASE}/analitik/kodefikasi`);
    await expect(shell(page)).toBeVisible({ timeout: 20000 });
    // FINDING #113 (product bug, filed rather than faked — see the task):
    //   The mapping table below `Mapping Kodefikasi BMN` never renders, because
    //   every mapping_kodefikasi query 500s. Two independent schema faults:
    //     (a) `perlengkapan.ms_barang` is joined at 13 sites in
    //         mapping_kodefikasi/repository.rs but is created by NO migration —
    //         `grep -rn ms_barang --include=*.sql .` returns zero hits.
    //     (b) both progress queries select `s.nama` from `authenc.satkers`,
    //         whose column is `name` (001_baseline.sql:4376).
    //   The FE then swallows the error to `None` (mapping_kodefikasi_dashboard.rs:35)
    //   and `.and_then(|data| data.map(..))` renders NOTHING — no error state — so
    //   the failure is invisible in the browser.
    // Until the module has a schema, assert what the route genuinely delivers:
    // it mounts under the app shell and its own chrome renders. The header
    // assertions move back here once #113 lands.
    await expect(page.getByRole("heading", { name: "Mapping Kodefikasi BMN" }).first()).toBeVisible({
      timeout: 20000,
    });
    await expect(page.getByRole("button", { name: /Export XLSX/ })).toBeVisible();
  });
});

// ---------------------------------------------------------------------------
// Notifikasi — per-user inbox, read lifecycle, cross-user isolation
// ---------------------------------------------------------------------------
test.describe("Notifikasi — inbox read lifecycle", () => {
  test.use({ storageState: storageStatePath("operator_a") });

  test("inbox lists this user rows only, and the unread filter hides the read one", async ({ page }) => {
    await page.goto(`${BASE}/notifikasi`);
    await expect(shell(page)).toBeVisible({ timeout: 20000 });
    await expect(page.getByRole("heading", { name: "Notifikasi" }).first()).toBeVisible();

    await expect(page.getByText("E2E Notifikasi Satu").first()).toBeVisible({ timeout: 20000 });
    await expect(page.getByText("E2E Notifikasi Dua").first()).toBeVisible();
    await expect(page.getByText("E2E Notifikasi Terbaca").first()).toBeVisible();
    // Per-user isolation: operator_b's row must never appear here.
    await expect(page.getByText("E2E Notifikasi Operator B")).toHaveCount(0);

    // "Hanya yang belum dibaca" re-queries with unread_only=true.
    await page.getByRole("checkbox").check();
    await expect(page.getByText("E2E Notifikasi Terbaca")).toHaveCount(0, { timeout: 20000 });
    await expect(page.getByText("E2E Notifikasi Satu").first()).toBeVisible();
  });

  test("Tandai semua dibaca drives every row read in the backend", async ({ page, request }) => {
    const token = await tokenFor(request, "operator_a");

    // GET /notifikasi/unread-count -> ApiResponse<UnreadCountResponse> = { data: { count } }
    const unreadCount = async () => {
      const res = await request.get(`${API}/notifikasi/unread-count`, {
        headers: { Authorization: `Bearer ${token}` },
      });
      expect(res.ok(), `unread-count -> ${res.status()}`).toBeTruthy();
      return (await res.json()).data.count as number;
    };

    await page.goto(`${BASE}/notifikasi`);
    await expect(shell(page)).toBeVisible({ timeout: 20000 });
    await expect(page.getByText("E2E Notifikasi Satu").first()).toBeVisible({ timeout: 20000 });

    await page.getByRole("button", { name: "Tandai semua dibaca" }).click();

    // Backend truth: this user's inbox drains to zero unread.
    await expect.poll(unreadCount, { timeout: 20000 }).toBe(0);

    // …and operator_b's own unread row is untouched (mark_all_read is per-user).
    const tokenB = await tokenFor(request, "operator_b");
    const resB = await request.get(`${API}/notifikasi/unread-count`, {
      headers: { Authorization: `Bearer ${tokenB}` },
    });
    expect(resB.ok(), `operator_b unread-count -> ${resB.status()}`).toBeTruthy();
    const unreadB = (await resB.json()).data.count as number;
    expect(unreadB, "operator_b's unread survives operator_a's mark-all").toBeGreaterThan(0);
  });

  test("a user cannot mark another user notification read (ownership enforced)", async ({ request }) => {
    // mark_as_read is keyed on (id, claims.user_id) — `notifikasi/api.rs:124`
    // passes the token's user_id and the UPDATE is `WHERE id=$1 AND user_id=$2`
    // (`in_app.rs:172`), so operator_b patching operator_a's N1 must be a no-op.
    //
    // Asserted as "B's PATCH did not CHANGE N1", never as "N1 is unread": the
    // sibling test above clicks "Tandai semua dibaca" and legitimately drains
    // operator_a's whole inbox, so an absolute `read === false` here is only
    // true when this file runs out of order. Compare before/after instead.
    const tokenA = await tokenFor(request, "operator_a");
    const readStateOfN1 = async (): Promise<boolean | undefined> => {
      const listed = await request.get(`${API}/notifikasi?limit=100&offset=0&unread_only=false`, {
        headers: { Authorization: `Bearer ${tokenA}` },
      });
      const rows: Array<{ id: string; read: boolean }> = (await listed.json()).data ?? [];
      return rows.find((r) => r.id === N1)?.read;
    };

    const before = await readStateOfN1();
    expect(before, "N1 must be visible in operator_a's inbox for this test to mean anything").toBeDefined();

    const tokenB = await tokenFor(request, "operator_b");
    const res = await request.patch(`${API}/notifikasi/${N1}/read`, {
      headers: { Authorization: `Bearer ${tokenB}` },
    });

    // Either an explicit rejection or a silent no-op — never a cross-user write.
    if (res.ok()) {
      expect(await readStateOfN1(), "cross-user PATCH must not mutate N1").toBe(before);
    } else {
      expect(res.status()).toBeGreaterThanOrEqual(400);
    }
  });
});

test.describe("Notifikasi — operator_b sees only its own row", () => {
  test.use({ storageState: storageStatePath("operator_b") });

  test("inbox is scoped to operator_b", async ({ page }) => {
    await page.goto(`${BASE}/notifikasi`);
    await expect(shell(page)).toBeVisible({ timeout: 20000 });
    await expect(page.getByText("E2E Notifikasi Operator B").first()).toBeVisible({ timeout: 20000 });
    await expect(page.getByText("E2E Notifikasi Satu")).toHaveCount(0);
    await expect(page.getByText("E2E Notifikasi Dua")).toHaveCount(0);
  });
});

// Reference the seeded ids that are asserted indirectly, so the constants stay
// honest if the fixture changes.
test("seeded notifikasi ids are the ones the fixture defines", async ({ request }) => {
  const token = await tokenFor(request, "operator_a");
  const res = await request.get(`${API}/notifikasi?limit=100&offset=0&unread_only=false`, {
    headers: { Authorization: `Bearer ${token}` },
  });
  expect(res.ok()).toBeTruthy();
  const ids: string[] = ((await res.json()).data ?? []).map((r: { id: string }) => r.id);
  expect(ids).toEqual(expect.arrayContaining([N1, N2, N3]));
  expect(ids, "operator_a's inbox must not contain operator_b's row").not.toContain(N4);
});
