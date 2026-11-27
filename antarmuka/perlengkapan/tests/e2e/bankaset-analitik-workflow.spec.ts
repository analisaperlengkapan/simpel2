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
 *   create→list and the notifikasi read lifecycle.
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
 * Seeded assets (integrasi.siman_aset, shared seed section 3). `jenis_aset` now
 * uses the REAL SIMAN taxonomy — the seed previously invented "Peralatan dan
 * Mesin"/"Alat Kantor", which existed nowhere in the 624 533-row snapshot and
 * showed up as a 16th phantom category next to the 15 real ones:
 *   E2E-A-1 Toyota Avanza  Alat Angkutan Bermotor     / Baik         0200010
 *   E2E-A-2 Laptop Dell    Peralatan Mesin Khusus TIK / Baik         0200010
 *   E2E-B-1 Honda Vario    Alat Angkutan Bermotor     / Baik         0200020
 *   E2E-B-2 Tanah Kantor   Tanah                      / Baik         0200020
 *   E2E-C-1 Printer Epson  Peralatan Mesin Khusus TIK / Rusak Ringan 0300010
 * The NUP markers live in `no_aset` (where real SIMAN NUPs live), so every test
 * below narrows the list with `gotoSeededList` and works in ANY environment
 * rather than only against an empty CI database.
 *
 * WHAT THE UI ACTUALLY SHOWS (verified, do not "fix" back to siman_aset.nama):
 *   `repository.rs:563` maps `BankAsetItem.nama_aset` from `ur_sskel` (the BMN
 *   sub-sub-group nomenclature) and only falls back to `nama` when ur_sskel is
 *   NULL. Both `list_page.rs:409` and `detail_page.rs:125` render that field, and
 *   `sort=nama_asc` is `ORDER BY ur_sskel` (`repository.rs:98`) — so the whole
 *   stack is consistently ur_sskel-based. The strings on screen are therefore the
 *   `ur_sskel` column, NOT the brand names above:
 *     E2E-A-1 "Station Wagon"    E2E-A-2 "Note Book"
 *     E2E-B-1 "Sepeda Motor"     E2E-B-2 "Tanah Bangunan Kantor Pemerintah"
 *     E2E-C-1 "Printer (Peralatan Personal Komputer)"
 *   Those are the names SIMAN itself gives those five barang codes. They used
 *   to be invented, which mattered here more than anywhere: this file asserts
 *   the strings, so it was certifying the fiction.
 *   (Search still matches the brand: the BE ILIKEs ur_sskel OR nama OR kd_brg OR
 *   no_aset OR merk, so "Toyota" narrows to E2E-A-1 by `nama`/`merk`.)
 */
import { test, expect, type Page, type APIRequestContext } from "@playwright/test";
import { readFile } from "node:fs/promises";
import {
  credsFor,
  apiLogin,
  storageStatePath,
  tokenFor,
  TEST_USERS,
  PERLENGKAPAN_API_URL,
  deriveScope,
  formatThousands,
} from "./helpers/real-auth";

const BASE = "/perlengkapan/simpel";
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

/**
 * Shared marker on every seeded asset's NUP. The seed puts it in `no_aset`
 * (which is where a real SIMAN NUP lives — see ASSET_NUP_SQL in
 * layanan/perlengkapan/src/bank_aset/repository.rs), and the backend's search
 * ILIKEs `no_aset`, so searching this prefix isolates exactly the seeded rows.
 */
const SEED_MARKER = "E2E-";

/**
 * Open the asset list narrowed to the seeded rows, in ANY environment.
 *
 * Without this, the filter/sort maths below is only valid on a fresh CI database
 * that contains nothing but the 5 seeded assets. Against staging the list is
 * PER_PAGE=25 rows out of 624 533 ordered by `updated_at DESC`, so the seeded
 * rows are usually not on page 1 at all — `visibleNups` (which filters a fixed
 * array against the page text) would return [] and every `toHaveLength(5)`
 * would fail, while the negative assertions passed for the wrong reason.
 *
 * Searching the seed marker makes the working set deterministic and equal in
 * both environments, so the same spec certifies CI and staging. Real SIMAN rows
 * cannot collide: their `no_aset` is numeric (624 528 of 624 533 rows).
 */
async function gotoSeededList(page: Page): Promise<void> {
  await page.goto(`${BASE}/bank-aset/daftar`);
  await expect(shell(page)).toBeVisible({ timeout: 20000 });
  await page.getByPlaceholder("Cari nama/kode/NUP/merk...").fill(SEED_MARKER);
  await page.getByRole("button", { name: "Terapkan" }).click();
  // Against the 624 533-row snapshot one search takes 6-7 s on staging, and this page asks for
  // four heavy things at once when it opens (the list, the filter options, and the search twice:
  // once for the typing, once for "Terapkan"), so the answer arrived after 17 s in a measured run
  // and after more than 20 s in rc54's pipeline run. 20 s left 3 s of margin on a shared host. (The
  // bound stays under the 60 s a whole test is allowed, so a slow answer is reported as one.)
  await expect.poll(() => visibleNups(page), { timeout: 45000 }).toHaveLength(5);
}

/**
 * The SIMAN finance codes of the two satkers the e2e seed creates (seed-multisatker.sql, section 2).
 *
 * Their NAMES are not theirs alone: the 624 533-row snapshot on staging has real satkers called
 * "KEJAKSAAN NEGERI JAKARTA PUSAT" and "KEJAKSAAN NEGERI JAKARTA SELATAN" too, in a different
 * wilayah, so anything that picks a satker by name picks the real one on staging and the seeded one
 * on an empty CI database. The code is unique in both.
 */
const SEED_SATKER = {
  pusat: "006019999010001KD",
  selatan: "006019999020001KD",
};

// ---------------------------------------------------------------------------
// Bank Aset — filters / search / sort / pagination drive the real list
// ---------------------------------------------------------------------------
test.describe("Bank Aset — daftar controls (validator_pusat, narrowed to the seeded rows)", () => {
  test.use({ storageState: storageStatePath("validator_pusat") });

  test("list renders every seeded asset before filtering", async ({ page }) => {
    await gotoSeededList(page);
    await expect(page.getByText("Daftar Aset").first()).toBeVisible();
    await expect
      .poll(() => visibleNups(page), { timeout: 20000 })
      .toEqual(["E2E-A-1", "E2E-A-2", "E2E-B-1", "E2E-B-2", "E2E-C-1"]);
  });

  test("search narrows to the matching asset and Reset restores the full list", async ({ page, request }) => {
    await gotoSeededList(page);

    await page.getByPlaceholder("Cari nama/kode/NUP/merk...").fill("Toyota");
    await page.getByRole("button", { name: "Terapkan" }).click();
    // Real server-side search: only the Toyota row survives (matched on nama/merk).
    await expect.poll(() => visibleNups(page), { timeout: 20000 }).toEqual(["E2E-A-1"]);
    await expect(page.getByText("Station Wagon").first()).toBeVisible();

    // Reset clears the search too, so the list returns to the caller's FULL
    // scope — which is 5 only on an empty CI database. Assert the real claim
    // ("all filters cleared") against the total the backend reports for this
    // token, so it holds against staging's 624 533-row snapshot as well.
    const token = await tokenFor(request, "validator_pusat");
    const full = await deriveScope(request, token);
    await page.getByRole("button", { name: "Reset" }).click();
    await expect(
      page.getByText(`Total: ${formatThousands(full.total)} aset`),
      "Reset must restore the unfiltered scope total",
    ).toBeVisible({ timeout: 20000 });
  });

  test("kondisi filter isolates the single Rusak Ringan asset", async ({ page }) => {
    await gotoSeededList(page);

    // Select by VALUE, not label: the FE renders each dynamic option as
    // `format!("{} ({})", o.value, o.count)` (list_page.rs:264), so the label is
    // "Rusak Ringan (1)" and matching on the bare value is both correct and
    // immune to the seeded row count changing.
    await page.getByRole("combobox").filter({ hasText: "Semua Kondisi" }).selectOption("Rusak Ringan");
    await expect.poll(() => visibleNups(page), { timeout: 20000 }).toEqual(["E2E-C-1"]);
    await expect(page.getByText("Printer (Peralatan Personal Komputer)").first()).toBeVisible();
  });

  /**
   * The list re-fetches on every filter change, and HTTP answers do not arrive in
   * the order the questions were asked. Staging, 2026-11-06: searching `E2E-` over
   * 624 533 rows took ~10 s and the page sent that search TWICE (the input event and
   * "Terapkan"); the second answer landed after the "Rusak Ringan" answer and replaced
   * it, so the dropdown said Rusak Ringan while the table listed five Baik rows
   * (4 of 12 runs). The existing "kondisi filter" test only hit this by luck.
   *
   * To make the bad ordering certain at ANY server speed, every search-only answer is
   * held back 3 s and the kondisi answer is not. A correct page ends on the kondisi
   * result; a page without a newest-request-wins guard falls back to the five search
   * hits once the held answers land.
   */
  test("a slow answer to an older search cannot overwrite the newer kondisi filter", async ({ page }) => {
    let held = 0;
    let released = 0;
    await page.route("**/api/v1/perlengkapan/bank-aset?*", async (route) => {
      const query = new URL(route.request().url()).searchParams;
      const searchOnly = Boolean(query.get("search")) && !query.get("kondisi");
      // Staging answers a search over 624 533 rows in ~10 s, which is Playwright's
      // default `fetch` timeout; the hold below is what this test is about, not that.
      // Count the hold when the request STARTS, not when the server answers: against
      // a slow server the answer comes seconds later, and counting then would let
      // "everything was released" be true before anything had even been held.
      if (searchOnly) held += 1;
      const response = await route.fetch({ timeout: 90000 });
      if (searchOnly) await new Promise((resolve) => setTimeout(resolve, 3000));
      await route.fulfill({ response });
      if (searchOnly) released += 1;
    });

    await page.goto(`${BASE}/bank-aset/daftar`);
    await expect(shell(page)).toBeVisible({ timeout: 20000 });
    await page.getByPlaceholder("Cari nama/kode/NUP/merk...").fill(SEED_MARKER);
    await page.getByRole("button", { name: "Terapkan" }).click();
    // Do NOT wait for the search answer: pick the kondisi while it is still in flight.
    const kondisi = page.getByRole("combobox").filter({ hasText: "Semua Kondisi" });
    await expect(kondisi).toBeVisible({ timeout: 30000 });
    await kondisi.selectOption("Rusak Ringan", { timeout: 30000 });
    await expect.poll(() => visibleNups(page), { timeout: 60000 }).toEqual(["E2E-C-1"]);

    // Now let every held (stale) answer land, and look again.
    expect(held, "the search was actually held back").toBeGreaterThan(0);
    await expect.poll(() => released, { timeout: 90000, message: "held search answers must be released" }).toBe(held);
    await page.waitForTimeout(1500); // the page applies the late answers after they land
    expect(await visibleNups(page), "a stale search answer overwrote the newer kondisi filter").toEqual(["E2E-C-1"]);
    await expect(page.getByText("Total: 1 aset")).toBeVisible();
  });

  test("jenis filter isolates the single Tanah asset", async ({ page }) => {
    await gotoSeededList(page);

    await page.getByRole("combobox").filter({ hasText: "Semua Jenis BMN" }).selectOption("Tanah");
    await expect.poll(() => visibleNups(page), { timeout: 20000 }).toEqual(["E2E-B-2"]);
    await expect(page.getByText("Tanah Bangunan Kantor Pemerintah").first()).toBeVisible();
  });

  test("sort by Nama A→Z reorders the rows server-side", async ({ page }) => {
    await gotoSeededList(page);

    await page.getByRole("combobox").filter({ hasText: "Terbaru diperbarui" }).selectOption("nama_asc");
    // `nama_asc` is ORDER BY ur_sskel ASC server-side, so the row order becomes:
    //   "Note Book" (E2E-A-2) < "Printer (Peralatan Personal Komputer)" (E2E-C-1)
    //   < "Sepeda Motor" (E2E-B-1) < "Station Wagon" (E2E-A-1)
    //   < "Tanah Bangunan Kantor Pemerintah" (E2E-B-2)
    // The five real names differ at the first letter (N/P/Se/St/T), so the order
    // is the same under any collation AND there is no longer a tie — the whole
    // sequence is assertable, where the invented names shared a string and left
    // two positions ambiguous.
    await expect
      .poll(() => nupsInDomOrder(page), { timeout: 20000 })
      .toEqual(["E2E-A-2", "E2E-C-1", "E2E-B-1", "E2E-A-1", "E2E-B-2"]);
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
    await expect(page.getByText("Station Wagon").first()).toBeVisible();
    await expect(page.getByText("3020101003").first()).toBeVisible(); // kode barang
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
    // Real aggregation of the seeded rows — the satkers holding assets must appear. The table is
    // paged (25 of 556 on staging, ordered by asset count) so the two seeded satkers, which hold
    // two assets each, are not on its first page there: find each by its code, as a person would.
    for (const [kode, nama] of [
      [SEED_SATKER.pusat, "KEJAKSAAN NEGERI JAKARTA PUSAT"],
      [SEED_SATKER.selatan, "KEJAKSAAN NEGERI JAKARTA SELATAN"],
    ]) {
      await page.getByTestId("cari-sebaran").fill(kode);
      await expect(page.getByTestId("baris-sebaran").getByText(nama)).toBeVisible({ timeout: 20000 });
    }
  });

  test("the map drills from the whole country to a wilayah to one satker", async ({ page, request }) => {
    // Truth from the API, not from the seed's own comments.
    const token = await tokenFor(request, "validator_pusat");
    const res = await request.get(`${API}/bank-aset/sebaran`, { headers: { Authorization: `Bearer ${token}` } });
    expect(res.ok()).toBeTruthy();
    type Row = { kode_satker: string | null; nama_satker: string; lintang: number | null; bujur: number | null; wilayah_kode: string | null; wilayah_nama: string | null; total_aset: number };
    const satker: Row[] = (await res.json()).data.satker;
    const placed = (r: Row) => r.lintang !== null && r.bujur !== null;
    const wilayah = new Map<string, Row[]>();
    for (const r of satker) wilayah.set(r.wilayah_kode ?? "", [...(wilayah.get(r.wilayah_kode ?? "") ?? []), r]);
    // The seeded Kejari, found by its code (its name also belongs to a real satker on staging), and
    // the wilayah it sits in: the seed's own two Kejari, under "KEJAKSAAN TINGGI DKI JAKARTA".
    const seeded = satker.find((r) => r.kode_satker === SEED_SATKER.selatan);
    expect(seeded, "the seeded Jakarta Selatan Kejari is in the answer").toBeTruthy();
    const dki = wilayah.get(seeded?.wilayah_kode ?? "") ?? [];
    expect(dki.filter(placed).length, "the seed gives the DKI Kejari a place").toBeGreaterThanOrEqual(2);

    await page.goto(`${BASE}/bank-aset/sebaran`);
    await expect(page.getByTestId("peta-sebaran")).toBeVisible({ timeout: 20000 });

    // Level 1, the country: one dot per wilayah that has a placed satker, and the national summary.
    const withPlace = [...wilayah.values()].filter((rows) => rows.some(placed)).length;
    await expect(page.getByTestId("titik-peta")).toHaveCount(withPlace);
    await expect(page.getByTestId("panel-nasional")).toBeVisible();
    await expect(page.getByTestId("panel-nasional")).toContainText("Seluruh Indonesia");

    // Level 2: pick the wilayah. The trail follows, the dots become that wilayah's satkers.
    const nama = dki.find((r) => r.wilayah_nama)?.wilayah_nama ?? "";
    expect(nama).not.toBe("");
    // By keyboard: on the national map the dots of neighbouring wilayah overlap (here, the seeded
    // DKI group sits on top of the real Jakarta one, a few km away), and a pointer click lands on the
    // centre pixel, which belongs to whichever dot is drawn last. Enter on the focused dot is what
    // a keyboard user does, and does not depend on who is on top.
    const titik = page.getByRole("button", { name: new RegExp(`^${nama}:`) });
    await expect(titik).toHaveCount(1);
    await titik.focus();
    await titik.press("Enter");
    await expect(page.getByTestId("panel-wilayah")).toBeVisible();
    await expect(page.getByTestId("peta-remah")).toContainText(nama);
    await expect(page.getByTestId("titik-peta")).toHaveCount(dki.filter(placed).length, { timeout: 10000 });
    // The satker table narrows to the same wilayah.
    await expect(page.getByTestId("hitungan-sebaran")).toContainText(`${dki.length} dari`);

    // Level 3: pick a satker. Its card, its BMN by jenis, and what GPS the assets have.
    await page.getByRole("button", { name: /KEJAKSAAN NEGERI JAKARTA SELATAN/ }).first().click();
    const kartu = page.getByTestId("kartu-satker");
    await expect(kartu).toContainText("KEJAKSAAN NEGERI JAKARTA SELATAN");
    await expect(page.getByTestId("rincian-satker")).toContainText("Jenis BMN");
    await expect(page.getByTestId("rincian-satker")).not.toContainText("Memuat rincian", { timeout: 20000 });
    await expect(page.getByTestId("gps-satker")).toBeVisible();
    await expect(kartu.getByRole("link", { name: "Lihat aset satker ini" })).toHaveAttribute("href", /\?kode_siman=/);

    // Back up through the trail.
    await page.getByTestId("peta-remah").getByRole("button", { name: "Indonesia" }).click();
    await expect(page.getByTestId("panel-nasional")).toBeVisible();
    await expect(page.getByTestId("titik-peta")).toHaveCount(withPlace);
    await expect(page.getByTestId("hitungan-sebaran")).toContainText(`${satker.length} dari`);
    await expect(page.getByTestId("peta-cakupan")).toContainText(`${satker.filter(placed).length} satker ada di peta`);
  });

  test("the table narrows by search and the satker link lands on its own asset list", async ({ page }) => {
    // Two searches over 624 533 rows in a row, each allowed up to a minute below.
    test.setTimeout(180000);
    await page.goto(`${BASE}/bank-aset/sebaran`);
    await expect(page.getByTestId("baris-sebaran")).toBeVisible({ timeout: 20000 });
    // By code: the NAME "jakarta selatan" is also a real satker's on staging (and its kode, wilayah and
    // 1 410 assets are not the seed's), so a name would narrow to two rows there and to one in CI.
    await page.getByTestId("cari-sebaran").fill(SEED_SATKER.selatan);
    await expect(page.getByTestId("baris-sebaran").locator("tr")).toHaveCount(1);
    await expect(page.getByTestId("hitungan-sebaran")).toContainText("1 dari");
    const link = page.getByTestId("baris-sebaran").getByRole("link", { name: /JAKARTA SELATAN/ });
    await link.click();
    // The list opens already filtered to that satker, and shows only its two assets (identified by
    // their NUP, which is unique; the displayed names are SIMAN's, not the seed's brand names). The
    // answer takes as long as any search over 624 533 rows.
    await expect.poll(() => visibleNups(page), { timeout: 60000 }).toEqual(["E2E-B-1", "E2E-B-2"]);

    // Picking another satker from the dropdown replaces the code filter, instead of asking for rows
    // that carry BOTH satkers' identities (which would be nothing). The dropdown names a satker by
    // NAME, and on staging that name also belongs to a real satker with 1 679 assets, so the seed
    // marker narrows the answer to the seeded two.
    await page.locator("select").last().selectOption("KEJAKSAAN NEGERI JAKARTA PUSAT");
    await expect(page.getByTestId("saring-kode-satker")).toHaveCount(0);
    await page.getByPlaceholder("Cari nama/kode/NUP/merk...").fill(SEED_MARKER);
    await page.getByRole("button", { name: "Terapkan" }).click();
    await expect.poll(() => visibleNups(page), { timeout: 60000 }).toEqual(["E2E-A-1", "E2E-A-2"]);
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

  // The trend card has no value axis, so it says the two numbers that give it a scale; and a year
  // that is not over is set apart instead of being drawn as a collapse at the right edge.
  test("the trend card names its peak and marks the unfinished current year", async ({ page, request }) => {
    const token = await tokenFor(request, "validator_pusat");
    const res = await request.get(`${API}/bank-aset/dashboard`, { headers: { Authorization: `Bearer ${token}` } });
    expect(res.ok(), `GET /bank-aset/dashboard -> ${res.status()}`).toBeTruthy();
    const years: Array<{ tahun: number }> = (await res.json()).data.per_tahun ?? [];
    test.skip(years.length < 2, "the seeded SIMAN rows carry fewer than two acquisition years: no trend to draw");

    await page.goto(`${BASE}/dashboard`);
    await expect(page.getByText(/Tertinggi [\d.]+ \(\d{4}\)/).first()).toBeVisible({ timeout: 20000 });
    const sekarang = new Date().getFullYear();
    if (years.length >= 3 && years.some((y) => y.tahun === sekarang)) {
      await expect(page.getByText(`${sekarang} masih berjalan`, { exact: false }).first()).toBeVisible();
    }
  });

  // The yearly bars read like a time axis: oldest on the left, the most recent fifteen years, one
  // row (they used to run newest first and wrap, with the current year at the far left).
  test("bank aset yearly bars run oldest to newest, the most recent fifteen", async ({ page, request }) => {
    const token = await tokenFor(request, "validator_pusat");
    const res = await request.get(`${API}/bank-aset/dashboard`, { headers: { Authorization: `Bearer ${token}` } });
    const api: number[] = ((await res.json()).data.per_tahun ?? []).map((y: { tahun: number }) => y.tahun);
    test.skip(api.length === 0, "no acquisition years in the seeded SIMAN rows");

    await page.goto(`${BASE}/bank-aset/dashboard`);
    const bars = page.getByTestId("perolehan-per-tahun").locator("[data-tahun]");
    await expect(bars.first()).toBeVisible({ timeout: 20000 });
    const shown = (await bars.evaluateAll((els) => els.map((e) => Number(e.getAttribute("data-tahun")))));
    const expected = [...api].sort((a, b) => a - b).slice(-15);
    expect(shown, "oldest to newest, the most recent fifteen").toEqual(expected);
  });

  // Split per-format (was one test looping both): the loop threw on `excel` and
  // so never reached `pdf`, leaving pdf's real status unknown for the whole life
  // of the test. Separate tests report both.
  for (const fmt of ["excel", "pdf"] as const) {
    test(`dashboard export/${fmt} responds server-side`, async ({ request }) => {
      // #116 FIXED. The 500 was never in the export layer: both handlers call
      // `get_perlengkapan_dashboard_metrics` FIRST, and its sub-queries targeted
      // two relations that exist in NO environment (`perlengkapan.kebutuhan_bmn`,
      // `perlengkapan.pakaian_dinas` — verified absent from V001__baseline) plus
      // two columns that do not exist on relations that DO (`authenc.satkers.nama`,
      // `..._satker_aktivitas.aktivitas_id`) — the same never-built data model as
      // #118. `tokio::try_join!` propagates the first error, so BOTH formats
      // failed identically; that is why pdf failed too despite never touching the
      // filesystem.
      //
      // A second, distinct fault survived that rewrite and only CI caught it: the
      // rebuilt workflow query read `AVG(EXTRACT(EPOCH ...))` into an f64. That
      // expression is NUMERIC on PostgreSQL 14+, tokio_postgres has no
      // FromSql<f64> for NUMERIC, and the release profile sets `panic = "abort"` —
      // so the backend PROCESS died on the first export request and every later
      // test in this file failed with ENOTFOUND. Hence the `::FLOAT8` casts in
      // dashboard/repository.rs.
      //
      // The earlier `test.fail(true, …)` marker did its job: the assertions were
      // left describing CORRECT behaviour, so they now simply pass.
      const token = await tokenFor(request, "validator_pusat");
      // `tahun_anggaran` is a REQUIRED query param (DashboardParams.tahun_anggaran
      // is a bare i32, dashboard/models.rs:10) — omitting it is a legitimate 400
      // from the Query extractor, not endpoint rot.
      const res = await request.get(`${API}/dashboard/perlengkapan/export/${fmt}?tahun_anggaran=2026`, {
        headers: { Authorization: `Bearer ${token}` },
      });
      expect(res.status(), `dashboard export/${fmt} status`).toBeLessThan(400);
      const body = await res.body();
      expect(body.length, `dashboard export/${fmt} body`).toBeGreaterThan(0);
      // Assert the file IS what the Content-Type claims, by magic bytes.
      // "status 200 + non-empty body" passed while export/pdf returned an HTML
      // document under `Content-Type: application/pdf` — a .pdf no reader can
      // open. A format assertion is the only thing that can catch that.
      const magic = { pdf: "%PDF-", excel: "PK" } as const;
      expect(
        body.subarray(0, magic[fmt].length).toString("latin1"),
        `dashboard export/${fmt} must really be ${fmt}, not something mislabelled`,
      ).toBe(magic[fmt]);
    });
  }

  // #97: the exports now have a real FE caller. Asserting the BUTTON drives a
  // download closes the gap that let a 500 endpoint sit unnoticed — a
  // server-side probe alone keeps passing with no UI behind it.
  // Labels are "Excel" / "PDF", not "Export Excel" / "Export PDF": the section
  // header beside them already reads "Unduh rekap", so the old labels stated
  // the verb twice. `getByRole("button", { name })` is an accessible-name
  // match, so these are exact — deliberately, since `getByText` would match a
  // substring case-insensitively and pass while reading something else
  // entirely.
  for (const [label, ext] of [
    ["Excel", "xlsx"],
    ["PDF", "pdf"],
  ] as const) {
    test(`dashboard "${label}" button downloads a file`, async ({ page }) => {
      await page.goto(`${BASE}/dashboard`, { waitUntil: "domcontentloaded" });
      const btn = page.getByRole("button", { name: label });
      await expect(btn, `${label} button is rendered`).toBeVisible({ timeout: 20000 });

      const [download] = await Promise.all([
        page.waitForEvent("download", { timeout: 20000 }),
        btn.click(),
      ]);
      expect(download.suggestedFilename(), `${label} filename`).toMatch(
        new RegExp(`\\.${ext}$`),
      );
      // The extension alone proves nothing about the bytes — the PDF export
      // shipped HTML named .pdf and this assertion passed. Read the saved file
      // and check its signature.
      const path = await download.path();
      const head = (await readFile(path)).subarray(0, 5).toString("latin1");
      const expected = ext === "pdf" ? "%PDF-" : "PK";
      expect(
        head.startsWith(expected),
        `${label} downloaded a file whose contents are not ${ext} (starts with ${JSON.stringify(head)})`,
      ).toBe(true);
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
// Analitik — roadmap create→list (real CRUD)
// ---------------------------------------------------------------------------
test.describe("Analitik — roadmap", () => {
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
    // The cost is money, read as money: thousands grouped, not the raw "125000000.00".
    await expect(page.locator("tr", { hasText: judul }).first()).toContainText("Rp 125.000.000");

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

// The inbox showed ONE page of 50 and stopped: a user with more notifications than
// that could never reach the older ones (the fixture row of "operator_b" sank below the
// first page on staging for exactly this reason). approver_satker has 55, newest
// first, so the 55th is reachable only through "Muat lebih banyak".
test.describe("Notifikasi — the inbox pages past the first page", () => {
  test.use({ storageState: storageStatePath("approver_satker") });

  test("older notifications are reachable with 'Muat lebih banyak'", async ({ page }) => {
    await page.goto(`${BASE}/notifikasi`);
    await expect(shell(page)).toBeVisible({ timeout: 20000 });

    // A full first page, and the oldest fixture row is NOT on it.
    await expect(page.getByTestId("notifikasi-jumlah")).toHaveText("Menampilkan 50 notifikasi", {
      timeout: 20000,
    });
    await expect(page.getByText("E2E Paginasi 55", { exact: true })).toHaveCount(0);

    const more = page.getByTestId("notifikasi-muat-lebih");
    await expect(more, "a full page must offer the next one").toBeVisible();
    await more.click();

    // The next page is APPENDED: the oldest row appears and the count grows past 50.
    await expect(page.getByText("E2E Paginasi 55", { exact: true })).toBeVisible({ timeout: 20000 });
    await expect(page.getByTestId("notifikasi-jumlah")).not.toHaveText("Menampilkan 50 notifikasi");
    // ...and what was already there is still there, once.
    await expect(page.getByText("E2E Paginasi 01", { exact: true })).toHaveCount(1);
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
