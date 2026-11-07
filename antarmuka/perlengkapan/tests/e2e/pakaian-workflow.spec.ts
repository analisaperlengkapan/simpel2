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
import { readFileSync } from "node:fs";
import { test, expect, type APIRequestContext } from "@playwright/test";
import { tokenFor, storageStatePath, PERLENGKAPAN_API_URL } from "./helpers/real-auth";
import { reachable } from "./helpers/page-load";

const BASE = "/perlengkapan/simpel/v2";
const PD_API = `${PERLENGKAPAN_API_URL}/api/v1/perlengkapan/pakaian-dinas`;

// Tokens come from the shared `tokenFor` helper rather than a spec-local
// lookup. Two things follow from that, and this spec got both wrong when it
// carried its own copy:
//   - `admin` is deliberately NOT in `TEST_USERS` (it is the all-role seed
//     user, not a satker-scoped one), so a local `TEST_USERS.find` throws on
//     it — the same break the helper's docstring already records happening
//     twice before,
//   - `apiLogin` resolves to `{ accessToken, refreshToken }`, so interpolating
//     its result into `Bearer ${...}` sends the literal `Bearer [object
//     Object]` and the backend answers 401. `tokenFor` returns the string.

/**
 * Rows out of a `PaginatedResponse`, with the status asserted first.
 *
 * Reading `.data` off an unchecked response is why this spec's first CI run
 * reported `Cannot read properties of undefined (reading 'find')` three times
 * and never once printed the 401 that caused it.
 */
/**
 * The `data` object out of an `ApiResponse`, with the status asserted first.
 * The list-shaped sibling is [`fetchRows`]; both exist so that no call site
 * reaches for `.data` before knowing the request succeeded.
 */
async function fetchObject<T>(
  request: APIRequestContext,
  url: string,
  token: string,
  what: string,
): Promise<T> {
  const res = await request.get(url, { headers: { Authorization: `Bearer ${token}` } });
  expect(res.ok(), `${what} (${res.status()}): ${await res.text()}`).toBeTruthy();
  const body = await res.json();
  expect(body.data, `${what} must carry a data object`).toBeTruthy();
  return body.data as T;
}

async function fetchRows<T>(
  request: APIRequestContext,
  url: string,
  token: string,
  what: string,
): Promise<T[]> {
  const res = await request.get(url, { headers: { Authorization: `Bearer ${token}` } });
  expect(res.ok(), `${what} (${res.status()}): ${await res.text()}`).toBeTruthy();
  const body = await res.json();
  expect(Array.isArray(body.data), `${what} must return a paginated envelope`).toBeTruthy();
  return body.data as T[];
}

/** The subset of `RosterPengisian` these tests assert on. */
interface Roster {
  satker_kode: string;
  dapat_diubah: boolean;
  pegawai: Array<{ nip: string }>;
}

// ── Master + campaign + report pages render real data ───────────────────────
test.describe("Pakaian Dinas — master, campaign & report pages", () => {
  test.use({ storageState: storageStatePath("admin") });

  // These three assert the page WORKS: the shell mounts, there is no auth
  // bounce, and every API call it makes on load succeeds. The last clause is
  // the one that matters — see helpers/page-load.ts for what it caught. The
  // seeded-row assertions for #94 live in the satker describe below, which is
  // where the mysimkari join actually runs.
  // (Full literal paths so the route-coverage gate detects these as visited.)

  test("jenis master page is reachable", async ({ page }) => {
    await reachable(page, `${BASE}/pakaian-dinas/jenis`);
  });
  test("pengajuan (campaign) page is reachable", async ({ page }) => {
    await reachable(page, `${BASE}/pakaian-dinas/pengajuan`);
  });
  test("laporan page is reachable", async ({ page }) => {
    await reachable(page, `${BASE}/pakaian-dinas/laporan`);
  });
});

// ── Master spesifikasi: the step that made the whole feature unusable ───────
// A campaign cannot be created without at least one spesifikasi, and until this
// change nothing in either frontend could create one — the backend's
// POST/PUT/DELETE for spesifikasi and subspesifikasi had zero callers, and
// staging showed the result: 8 jenis, 0 spesifikasi, 0 subspesifikasi. These
// tests drive the round trip so a regression takes the feature's entry point
// away again and is caught here rather than by an operator.
test.describe("Pakaian Dinas — master spesifikasi & subspesifikasi", () => {
  test.use({ storageState: storageStatePath("admin") });

  /** The seeded jenis carrying the reference rows migrated from simpelv1. */
  async function jenisPdh(request: APIRequestContext): Promise<string> {
    const token = await tokenFor(request, "admin");
    const list = await fetchRows<{ id: string; nama: string }>(
      request,
      `${PD_API}/jenis?page=1&per_page=100`,
      token,
      "jenis list",
    );
    const pdh = list.find((j) => j.nama.trim().toUpperCase() === "PDH");
    expect(pdh, "V011 seeds PDH; without it no campaign can be created").toBeTruthy();
    return pdh!.id;
  }

  test("the simpelv1 reference rows survived the migration", async ({ request }) => {
    const token = await tokenFor(request, "admin");
    const id = await jenisPdh(request);
    // `jenis_pakaian_dinas_id` is the name the backend reads; it used to read
    // `jenis_id`, which nothing sent, so this list came back unfiltered.
    const names = await fetchRows<{ nama: string; ukuran_group: string }>(
      request,
      `${PD_API}/spesifikasi?page=1&per_page=100&jenis_pakaian_dinas_id=${id}`,
      token,
      "spesifikasi list for PDH",
    );
    // The three PDH rows simpelv1 has carried since 2023. Asserted by name AND
    // size family: a seed that lands the right label under the wrong family
    // would offer shoe sizes for a shirt.
    for (const [nama, grup] of [
      ["Pakaian Dinas", "BAJU"],
      ["Celana", "CELANA"],
      ["Sepatu Dinas", "SEPATU"],
    ]) {
      const hit = names.find((s) => s.nama.trim().toUpperCase() === nama.toUpperCase());
      expect(hit, `spesifikasi "${nama}" missing from PDH`).toBeTruthy();
      expect(hit!.ukuran_group).toBe(grup);
    }
  });

  test("admin creates then deletes a spesifikasi through the page", async ({ page, request }) => {
    const id = await jenisPdh(request);
    await page.goto(`${BASE}/pakaian-dinas/jenis/${id}/spesifikasi`, {
      waitUntil: "domcontentloaded",
    });

    const nama = `E2E Spesifikasi ${Date.now()}`;
    await page.getByTestId("tambah-spesifikasi").click();
    await page.getByLabel("Nama Spesifikasi").fill(nama);
    // getByLabel resolves only because FormField now emits `for` — before this
    // change no label in perlengkapan was associated with its control.
    await page.getByLabel("Grup Ukuran").selectOption("CELANA");
    await page.getByRole("button", { name: "Simpan" }).click();

    const row = page.getByTestId("spesifikasi-tabel").locator("tr", { hasText: nama });
    await expect(row, "the new spesifikasi must appear without a reload").toHaveCount(1, {
      timeout: 20000,
    });

    // And it must be gone again — a create with no delete leaves master data
    // that only a DBA can correct.
    await row.getByRole("button", { name: "Hapus spesifikasi" }).click();
    await expect(row).toHaveCount(0, { timeout: 20000 });
  });

});

// ── Pengisian: the head the workflow never had ─────────────────────────────
// Nothing in the backend wrote `pengajuan_pakaian_dinas_satker_pegawai` or its
// sizes table before this — both were only ever read (report, dashboard) and
// cascade-deleted. The rows staging holds came from the e2e seed, not from the
// application, so the approval chain had nothing an operator could have filled.
test.describe("Pakaian Dinas — pengisian ukuran per satker", () => {
  test.use({ storageState: storageStatePath("operator_a") });

  const PENGAJUAN = "d1000000-0000-4d00-8d00-0000000000c1";
  const SATKER = "0200010";
  const ISI = `${BASE}/pakaian-dinas/pengajuan/${PENGAJUAN}/satker/${SATKER}/isi`;

  test("the roster comes from kepegawaian, not from a table anyone types into", async ({
    page,
    request,
  }) => {
    const token = await tokenFor(request, "operator_a");
    const data = await fetchObject<Roster>(
      request,
      `${PD_API}/pengajuan/${PENGAJUAN}/satker/${SATKER}/pegawai`,
      token,
      "roster for the caller's own satker",
    );
    // Every person in the satker appears, whether or not they have been filled
    // in yet — a roster that only lists people already saved cannot be used to
    // add the ones who are missing.
    expect(Array.isArray(data.pegawai)).toBeTruthy();
    expect(data.satker_kode).toBe(SATKER);
    expect(typeof data.dapat_diubah).toBe("boolean");

    await page.goto(ISI, { waitUntil: "domcontentloaded" });

    // Name the state before asserting the table. A missing `pengisian-tabel`
    // has three different causes that all report as "element(s) not found":
    // the route does not resolve (404), the campaign carries no clothing types
    // so the page refuses to draw a size table, or the table is genuinely
    // broken. Only the last is this test's subject, and the first run of this
    // test hit the second — the fixture gave …c1 no items, so the page was
    // right and the failure message pointed at the wrong thing.
    const kosong = page.getByTestId("pengisian-tanpa-jenis-pakaian");
    const tabel = page.getByTestId("pengisian-tabel");
    await expect
      .poll(
        async () => {
          if (await tabel.count()) return "tabel";
          if (await kosong.count()) return "kampanye-tanpa-jenis-pakaian";
          if (await page.getByTestId("halaman-404").count()) return "404";
          return "belum-termuat";
        },
        {
          timeout: 20000,
          message:
            "halaman pengisian tidak menampilkan tabel ukuran — nilai di bawah menamai sebabnya",
        },
      )
      .toBe("tabel");
    await expect(tabel).toBeVisible();
  });

  test("another satker's roster is not reachable", async ({ request }) => {
    // 0200020 belongs to operator_b. Out of scope answers NotFound rather than
    // Forbidden: a 403 would confirm the row exists under another satker.
    const token = await tokenFor(request, "operator_a");
    const res = await request.get(
      `${PD_API}/pengajuan/${PENGAJUAN}/satker/0200020/pegawai`,
      { headers: { Authorization: `Bearer ${token}` } },
    );
    expect(res.status(), "cross-satker roster must not be readable").toBe(404);
  });

  test("a satker past its own stage cannot have sizes written", async ({ request }) => {
    // Jakpus on this campaign sits at 1004 (Diajukan ke Pusat). Writing must be
    // refused by the SERVER, not merely by a disabled button.
    const token = await tokenFor(request, "operator_a");
    const data = await fetchObject<Roster>(
      request,
      `${PD_API}/pengajuan/${PENGAJUAN}/satker/${SATKER}/pegawai`,
      token,
      "roster before asserting it is locked",
    );
    test.skip(data.dapat_diubah, "this campaign is still editable; nothing to assert here");

    const nip = data.pegawai[0]?.nip;
    expect(nip, "the satker must have at least one person").toBeTruthy();
    const res = await request.put(
      `${PD_API}/pengajuan/${PENGAJUAN}/satker/${SATKER}/pegawai/${nip}`,
      {
        headers: { Authorization: `Bearer ${token}` },
        data: { with_hijab: false, ukuran: [] },
      },
    );
    expect(res.status(), "a submitted satker must refuse edits").toBe(400);
  });

  test("the page says why it is read-only instead of silently doing nothing", async ({
    page,
    request,
  }) => {
    const token = await tokenFor(request, "operator_a");
    const data = await fetchObject<Roster>(
      request,
      `${PD_API}/pengajuan/${PENGAJUAN}/satker/${SATKER}/pegawai`,
      token,
      "roster before asserting the locked notice",
    );
    test.skip(data.dapat_diubah, "campaign still editable; the locked notice is not shown");

    await page.goto(ISI, { waitUntil: "domcontentloaded" });
    await expect(page.getByTestId("pengisian-terkunci")).toBeVisible({ timeout: 20000 });
    await expect(page.getByTestId("ajukan-pengisian")).toHaveCount(0);
  });
});

// Same page, a role that may not write it. Split into its own describe because
// `test.use` binds a storage state per describe, not per test.
test.describe("Pakaian Dinas — master spesifikasi is read-only for non-admins", () => {
  test.use({ storageState: storageStatePath("operator_a") });

  test("an operator sees the list but is not offered the write controls", async ({
    page,
    request,
  }) => {
    // The server gates these with require_admin. Offering the button anyway
    // lets an operator fill the whole form and collect a 403.
    const token = await tokenFor(request, "operator_a");
    const list = await fetchRows<{ id: string; nama: string }>(
      request,
      `${PD_API}/jenis?page=1&per_page=100`,
      token,
      "jenis list as operator",
    );
    const pdh = list.find((j) => j.nama.trim().toUpperCase() === "PDH");
    expect(pdh, "V011 seeds PDH").toBeTruthy();

    await page.goto(`${BASE}/pakaian-dinas/jenis/${pdh!.id}/spesifikasi`, {
      waitUntil: "domcontentloaded",
    });
    await expect(page.getByTestId("spesifikasi-tabel")).toBeVisible({ timeout: 20000 });
    await expect(page.getByTestId("tambah-spesifikasi")).toHaveCount(0);
  });
});

// ── Laporan: both tabs and both exports, driven with a real pengajuan ───────
// Everything here 400'd before this spec existed, and `laporan page is
// reachable` stayed green through all of it because the shell mounts fine
// while every resource lands in its Err arm:
//   1. both tab endpoints take `pengajuan_id` as a REQUIRED Uuid, and the page
//      opens on the rekap tab with none selected — so the FIRST request the
//      page ever made was a 400,
//   2. the exports send the same required param the same optional way,
//   3. the exports sent the TAB id as `jenis_laporan`, so every export from the
//      pegawai tab sent `pegawai` where `cetak_laporan` matches `daftar`.
// The campaign is the one seeded in seed-perlengkapan-workflow.sql section 6.
test.describe("Pakaian Dinas — laporan tabs & exports", () => {
  test.use({ storageState: storageStatePath("admin") });

  const LAPORAN = `${BASE}/pakaian-dinas/laporan`;

  // Seed section 6b — a campaign built for reporting, deliberately NOT the one
  // the workflow tests drive. The reports only count satker rows at aktivitas
  // 1008, and the workflow tests move a satker to 1008 mid-run; reading the
  // same campaign would make these assertions depend on test order.
  //   2 satker × 2 jenis pakaian × 1 pegawai each
  //   0200010 → one L employee, 0200020 → one P employee
  const LAPORAN_CAMPAIGN = "d1000000-0000-4d00-8d00-0000000000c2";
  const JENIS_PDH = "d1000000-0000-4d00-8d00-000000000001";
  const SATKER_JAKPUS = "0200010";

  test("no pengajuan selected: prompts instead of requesting", async ({ page }) => {
    const calls: string[] = [];
    page.on("request", (r) => {
      if (r.url().includes("/pakaian-dinas/laporan/")) calls.push(r.url());
    });

    await reachable(page, LAPORAN);

    await expect(page.getByText("Pilih Pengajuan", { exact: true })).toBeVisible();
    expect(calls, "report endpoints must not be called without a pengajuan").toEqual([]);
    await expect(page.getByTestId("laporan-cetak-excel")).toBeDisabled();
    await expect(page.getByTestId("laporan-cetak-pdf")).toBeDisabled();

    // The dropdown must carry real campaigns, not just its placeholder. The
    // list endpoint answered 200 while the frontend DTO named five fields the
    // backend never sends, so serde rejected the body and this select rendered
    // its error arm — a failure `reachable` cannot see, because it watches HTTP
    // status and the status was fine. Assert on what the user can act on.
    const options = page.getByTestId("laporan-pengajuan").locator("option");
    await expect
      .poll(() => options.count(), {
        message: "period dropdown has no campaigns — check the FE↔BE contract for /pakaian-dinas/pengajuan",
      })
      .toBeGreaterThan(1);
  });

  test("selecting a pengajuan loads both tabs without a rejected request", async ({ page }) => {
    await reachable(page, LAPORAN);

    const rekap = page.waitForResponse((r) => r.url().includes("/laporan/rekap-ukuran"));
    await page.getByTestId("laporan-pengajuan").selectOption(LAPORAN_CAMPAIGN);
    const rekapResp = await rekap;
    expect(rekapResp.ok(), `GET rekap-ukuran -> ${rekapResp.status()} ${new URL(rekapResp.url()).search}`).toBeTruthy();

    const daftar = page.waitForResponse((r) => r.url().includes("/laporan/daftar-pegawai"));
    await page.getByTestId("laporan-tab-pegawai").click();
    const daftarResp = await daftar;
    expect(
      daftarResp.ok(),
      `GET daftar-pegawai -> ${daftarResp.status()} ${new URL(daftarResp.url()).search}`,
    ).toBeTruthy();
  });

  // Both filters were declared by no query type, so axum's `Query` dropped them
  // and the report came back unchanged — with a 200, so nothing looked wrong.
  // `satker_id` was worse: three of the four query builders applied it and the
  // one behind this tab did not, so the table and the spreadsheet exported from
  // the button beside it disagreed. Asserting "the request succeeded" would
  // still pass on the broken build; these assert the row count DROPS.
  test("satker and jenis pakaian filters narrow the rekap", async ({ page }) => {
    await reachable(page, LAPORAN);

    const groups = page.getByTestId("laporan-rekap-group");

    const unfiltered = page.waitForResponse((r) => r.url().includes("/laporan/rekap-ukuran"));
    await page.getByTestId("laporan-pengajuan").selectOption(LAPORAN_CAMPAIGN);
    expect((await unfiltered).ok()).toBeTruthy();
    await expect(groups, "seed 6b configures two clothing types").toHaveCount(2);

    // Clothing type: drops one of the two groups entirely.
    const byJenis = page.waitForResponse((r) => r.url().includes("/laporan/rekap-ukuran"));
    await page.getByTestId("laporan-jenis").selectOption(JENIS_PDH);
    const jenisResp = await byJenis;
    expect(
      new URL(jenisResp.url()).search,
      "jenis_pakaian_id must reach the server, not be dropped as an unknown key",
    ).toContain(`jenis_pakaian_id=${JENIS_PDH}`);
    expect(jenisResp.ok(), `rekap-ukuran -> ${jenisResp.status()}`).toBeTruthy();
    await expect(groups, "PDL group must disappear").toHaveCount(1);
    await expect(page.getByRole("heading", { name: "Kemeja PDH E2E (BAJU)" })).toBeVisible();

    // Satker: keeps the group but leaves only the Jakarta Pusat employee, who
    // is male — so the Perempuan total goes to 0. Counting rows alone would not
    // catch a filter applied to the wrong column; the gender split does.
    //
    // `selectOption`, not `fill`: this control is the campaign's own satker
    // list now. That change is why this test failed the moment the box became
    // a dropdown — the element type IS part of the contract, and a test that
    // types into it is asserting the old one.
    const bySatker = page.waitForResponse((r) => r.url().includes("/laporan/rekap-ukuran"));
    await page.getByTestId("laporan-satker").selectOption(SATKER_JAKPUS);
    const satkerResp = await bySatker;
    expect(new URL(satkerResp.url()).search).toContain(`satker_id=${SATKER_JAKPUS}`);
    expect(satkerResp.ok(), `rekap-ukuran -> ${satkerResp.status()}`).toBeTruthy();

    const group = groups.first();
    await expect(group.getByRole("row").filter({ hasText: "Laki-laki (L)" })).toContainText("1");
    await expect(
      group.getByRole("row").filter({ hasText: "Perempuan (P)" }),
      "the only remaining employee is male",
    ).toContainText("0");
  });

  // The dropdown and the coverage line are the two halves of one answer: which
  // satker this report can be narrowed to, and how much of the campaign the
  // numbers already cover. The filter used to be a free-text `kode_satker` box
  // with no coverage line beside it, so a recap over 3 of 238 satker rendered
  // identically to a complete one.
  test("the satker filter offers the campaign's own satker and states the coverage", async ({
    page,
  }) => {
    await reachable(page, LAPORAN);

    const satker = page.getByTestId("laporan-satker");
    const cakupan = page.getByTestId("laporan-cakupan");

    // Disabled rather than empty before a campaign is picked: an empty dropdown
    // reads as "this campaign has no satker", which is a different fact.
    await expect(satker).toBeDisabled();
    await expect(cakupan, "nothing to state the coverage of yet").toHaveCount(0);

    // 6b: both satker at 1008, so this report covers the campaign whole.
    const loaded = page.waitForResponse((r) => r.url().includes("/laporan/rekap-ukuran"));
    await page.getByTestId("laporan-pengajuan").selectOption(LAPORAN_CAMPAIGN);
    await loaded;

    await expect(satker).toBeEnabled();
    // Count, not order — the list is whatever the campaign returns. Three
    // options = the "Semua Satker" placeholder plus 6b's two satker, so a list
    // leaking another campaign's satker fails here.
    await expect(satker.locator("option")).toHaveCount(3);
    await expect(satker.locator("option[value='0200010']")).toHaveText(
      "KEJAKSAAN NEGERI JAKARTA PUSAT",
    );
    await expect(satker.locator("option[value='0200020']")).toHaveText(
      "KEJAKSAAN NEGERI JAKARTA SELATAN",
    );

    await expect(cakupan).toHaveAttribute("data-lengkap", "true");
    await expect(cakupan).toContainText("Seluruh 2 satker sudah Selesai");

    // 6a is the other direction, and it is stable in both orders: its two
    // satker never BOTH reach 1008 (the workflow tests move at most one), so
    // whenever this runs the line must say the numbers are partial. Without
    // this half, a banner hard-coded to "complete" passes the assertion above.
    const reload = page.waitForResponse((r) => r.url().includes("/laporan/rekap-ukuran"));
    await page.getByTestId("laporan-pengajuan").selectOption(CAMPAIGN);
    await reload;

    await expect(cakupan).toHaveAttribute("data-lengkap", "false");
    await expect(cakupan).toContainText("dari 2 satker");
  });

  // The export must carry the same filters as the table above it, or the
  // spreadsheet silently covers rows the screen excluded.
  test("exports carry the active filters", async ({ page }) => {
    await reachable(page, LAPORAN);

    const loaded = page.waitForResponse((r) => r.url().includes("/laporan/rekap-ukuran"));
    await page.getByTestId("laporan-pengajuan").selectOption(LAPORAN_CAMPAIGN);
    await loaded;

    const filtered = page.waitForResponse((r) => r.url().includes("/laporan/rekap-ukuran"));
    await page.getByTestId("laporan-jenis").selectOption(JENIS_PDH);
    await filtered;

    const cetak = page.waitForResponse((r) => r.url().includes("/laporan/cetak"));
    const downloadPromise = page.waitForEvent("download");
    await page.getByTestId("laporan-cetak-excel").click();

    const search = new URL((await cetak).url()).search;
    expect(search, "export URL must repeat the on-screen filter").toContain(
      `jenis_pakaian_id=${JENIS_PDH}`,
    );
    await downloadPromise;
  });

  // Driven per (tab, format) because the tab is what decides `jenis_laporan`,
  // and it was exactly the pegawai half that was broken. Same download
  // assertion as the kebutuhan-bmn export in admin-bantuan-workflow.spec.ts —
  // both now go through the same fetch-with-JWT → blob → anchor path.
  for (const [tab, jenisLaporan] of [
    ["rekap", "rekap"],
    ["pegawai", "daftar"],
  ] as const) {
    for (const [format, ext] of [
      ["excel", "xlsx"],
      ["pdf", "pdf"],
    ] as const) {
      test(`export ${format} from the ${tab} tab downloads real bytes`, async ({ page }) => {
        await reachable(page, LAPORAN);
        await page.getByTestId("laporan-pengajuan").selectOption(LAPORAN_CAMPAIGN);
        await page.getByTestId(`laporan-tab-${tab}`).click();

        const cetak = page.waitForResponse((r) => r.url().includes("/laporan/cetak"));
        const downloadPromise = page.waitForEvent("download");
        await page.getByTestId(`laporan-cetak-${format}`).click();

        // The request is asserted separately from the download because the two
        // defects were separable: a 401 (no Authorization header on a
        // window.open navigation) and a wrong `jenis_laporan` both end as "no
        // file appears", and only the query string tells them apart.
        const resp = await cetak;
        const search = new URL(resp.url()).search;
        expect(resp.ok(), `GET cetak -> ${resp.status()} ${search}`).toBeTruthy();
        expect(search).toContain(`jenis_laporan=${jenisLaporan}`);
        expect(search).toContain(`pengajuan_id=${LAPORAN_CAMPAIGN}`);

        const download = await downloadPromise;
        expect(download.suggestedFilename()).toMatch(new RegExp(`\\.${ext}$`));
        const stream = await download.createReadStream();
        let bytes = 0;
        for await (const chunk of stream) bytes += (chunk as Buffer).length;
        expect(bytes, `${tab}/${format} export must not be empty`).toBeGreaterThan(0);
      });
    }
  }
});

// ── Per-satker list + workflow (restored by V006/#94) ───────────────────────
// Seeded by tests/fixtures/e2e/seed-perlengkapan-workflow.sql section 6:
//   P1 satker 0200010 @1001 SUBMIT_TO_VALIDATOR → validator_wilayah approves → 1004
//   P2 satker 0200020 @1004 SUBMIT_TO_PUSAT     → validator_pusat  approves → 1008
const CAMPAIGN = "d1000000-0000-4d00-8d00-0000000000c1";
const P1_WILAYAH = "d1000000-0000-4d00-8d00-0000000a0001";
const P2_PUSAT = "d1000000-0000-4d00-8d00-0000000a0002";

/**
 * Backend source-of-truth for a satker row's aktivitas_id. Reads the per-campaign
 * satker list as validator_pusat (cross-satker, sees every row) — the SAME endpoint
 * whose `LEFT JOIN integrasi.mysimkari_satker` was invalid SQL before #94, so a
 * successful read here is itself part of the regression assertion.
 */
async function beAktivitas(
  request: import("@playwright/test").APIRequestContext,
  satkerRowId: string,
): Promise<number> {
  const accessToken = await tokenFor(request, "validator_pusat");
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

test.describe("Pakaian Dinas — per-satker list resolves the mysimkari join", () => {
  test("satker list carries names resolved from integrasi.mysimkari_satker", async ({ request }) => {
    const accessToken = await tokenFor(request, "validator_pusat");
    const resp = await request.get(`${PD_API}/pengajuan/${CAMPAIGN}/satker?page=1&per_page=100`, {
      headers: { Authorization: `Bearer ${accessToken}` },
    });
    expect(resp.status(), "satker list no longer errors on uuid=bigint").toBe(200);

    const body = await resp.json();
    const rows: Array<{ satker_id: string; satker_nama: string | null }> = body.data ?? [];
    const codes = rows.map((r) => r.satker_id);
    expect(codes, "both seeded satkers present, keyed by kode_satker").toEqual(
      expect.arrayContaining(["0200010", "0200020"]),
    );

    // satker_nama comes ONLY from the join, so a non-null value proves it resolved.
    const jakpus = rows.find((r) => r.satker_id === "0200010");
    expect(jakpus?.satker_nama, "name resolved through mysimkari_satker").toContain("JAKARTA PUSAT");
  });
});

test.describe("Pakaian Dinas — validator wilayah forwards a satker", () => {
  test("validator_wilayah advances P1 (1001 → 1004)", async ({ request }) => {
    expect(await beAktivitas(request, P1_WILAYAH)).toBe(1001);

    const wil = await tokenFor(request, "validator_wilayah");
    const resp = await request.post(`${PD_API}/validator-action`, {
      headers: { Authorization: `Bearer ${wil}` },
      data: { pengajuan_satker_id: P1_WILAYAH, aksi: "approve", komentar: "e2e teruskan" },
    });
    expect(resp.status(), "validator_wilayah may approve at 1001").toBe(200);

    await expect.poll(() => beAktivitas(request, P1_WILAYAH), { timeout: 15000 }).toBe(1004);
  });
});

test.describe("Pakaian Dinas — validator pusat decides", () => {
  test("validator_pusat approves P2 (1004 → 1008 Selesai)", async ({ request }) => {
    expect(await beAktivitas(request, P2_PUSAT)).toBe(1004);

    const pusat = await tokenFor(request, "validator_pusat");
    const resp = await request.post(`${PD_API}/validator-action`, {
      headers: { Authorization: `Bearer ${pusat}` },
      data: { pengajuan_satker_id: P2_PUSAT, aksi: "approve", komentar: "e2e setujui" },
    });
    expect(resp.status(), "validator_pusat may approve at 1004").toBe(200);

    await expect.poll(() => beAktivitas(request, P2_PUSAT), { timeout: 15000 }).toBe(1008);
  });
});

// ── Returns must have a way home, and say why ───────────────────────────────
// "Kembalikan ke Wilayah" (pusat, 1004 → 1007) used to park the satker for good:
// no transition left 1007, the sizes stay locked, and the satker screen offered
// nobody a button ("mode pantau"), so the campaign could never reach 100% Selesai.
// And an operator whose satker was sent back (1003) was not told why: the
// validator's note lived only in the history panel one page away.
// Both describes build their own campaign, so neither depends on which of the
// fixture rows above earlier tests already moved.
type Role = "operator_a" | "operator_b" | "validator_wilayah" | "validator_pusat";
const SATKER_REVISI = "0200010";

async function aksiPakaian(
  request: APIRequestContext,
  role: Role,
  rowId: string,
  action: "submit" | "approve" | "reject",
  komentar = `e2e ${role} ${action}`,
): Promise<{ status: number; aktivitas?: number }> {
  const token = await tokenFor(request, role);
  const res = await request.post(`${PD_API}/validator-action`, {
    headers: { Authorization: `Bearer ${token}` },
    data: { pengajuan_satker_id: rowId, aksi: action, komentar },
  });
  const body = res.ok() ? await res.json() : null;
  return { status: res.status(), aktivitas: body?.data?.aktivitas_id };
}

/** A campaign of our own aimed at one satker; returns its id and that satker's row (at 1000). */
/**
 * A calendar day, `selisih` days from today, as `YYYY-MM-DD`. Today is Jakarta's
 * (UTC+7): that is the calendar the server judges a campaign's dates by, so UTC
 * would be a day behind from 17:00 to midnight.
 */
const hari = (selisih: number) =>
  new Date(Date.now() + 7 * 3_600_000 + selisih * 86_400_000).toISOString().slice(0, 10);

async function kampanyeBaru(
  request: APIRequestContext,
  periode: { mulai: string; selesai: string } = { mulai: hari(-60), selesai: hari(300) },
): Promise<{ pengajuanId: string; rowId: string; pusat: string }> {
  const pusat = await tokenFor(request, "validator_pusat");
  const jenis = await fetchRows<{ id: string; nama: string }>(
    request,
    `${PD_API}/jenis?page=1&per_page=100`,
    pusat,
    "jenis list",
  );
  const pdh = jenis.find((j) => j.nama.trim().toUpperCase() === "PDH");
  expect(pdh, "V011 seeds PDH").toBeTruthy();
  const spesifikasi = await fetchRows<{ id: string }>(
    request,
    `${PD_API}/spesifikasi?page=1&per_page=100&jenis_pakaian_dinas_id=${pdh!.id}`,
    pusat,
    "spesifikasi list",
  );
  expect(spesifikasi.length, "PDH carries reference rows").toBeGreaterThan(0);

  const created = await request.post(`${PD_API}/pengajuan`, {
    headers: { Authorization: `Bearer ${pusat}` },
    data: {
      nama: `E2E revisi ${Date.now()}`,
      tahun: 2026,
      // Relative to today: the dates are enforced, so a fixed year would turn this red in January.
      tgl_mulai: periode.mulai,
      tgl_selesai: periode.selesai,
      is_reguler: true,
      pilihan_satker: "sebagian",
      jenis_pakaian_dinas_id: pdh!.id,
      spesifikasi_ids: [spesifikasi[0].id],
      satker_ids: [SATKER_REVISI],
    },
  });
  expect(created.status(), await created.text()).toBe(201);
  const pengajuanId = (await created.json()).data.id as string;

  const rows = await fetchRows<{ id: string; satker_id: string; aktivitas_id: number }>(
    request,
    `${PD_API}/pengajuan/${pengajuanId}/satker?page=1&per_page=100`,
    pusat,
    "satker rows of the new campaign",
  );
  const row = rows.find((r) => r.satker_id === SATKER_REVISI);
  expect(row, `the campaign materialises a row for ${SATKER_REVISI}`).toBeTruthy();
  expect(row!.aktivitas_id).toBe(1000);
  return { pengajuanId, rowId: row!.id, pusat };
}

test.describe("Pakaian Dinas — pusat returns a satker to wilayah, wilayah answers", () => {
  test.use({ storageState: storageStatePath("validator_wilayah") });

  test("1007 shows wilayah its two moves, and the loop reaches Selesai", async ({
    page,
    request,
  }) => {
    const { pengajuanId, rowId, pusat } = await kampanyeBaru(request);

    // 1000 → 1001 → 1004 → pusat sends it back → 1007.
    expect((await aksiPakaian(request, "operator_a", rowId, "submit")).aktivitas).toBe(1001);
    expect((await aksiPakaian(request, "validator_wilayah", rowId, "approve")).aktivitas).toBe(1004);
    expect((await aksiPakaian(request, "validator_pusat", rowId, "reject")).aktivitas).toBe(1007);

    // The screen: wilayah is offered both moves, not "mode pantau".
    await page.goto(`${BASE}/pakaian-dinas/pengajuan/${pengajuanId}/satker`, {
      waitUntil: "domcontentloaded",
    });
    await page.getByRole("button", { name: "Lihat Riwayat & Aksi" }).first().click({ timeout: 20000 });
    await expect(
      page.getByRole("button", { name: "Teruskan ke Pusat", exact: true }),
    ).toBeVisible({ timeout: 20000 });
    await expect(page.getByRole("button", { name: "Kembalikan untuk Revisi" })).toBeVisible();
    await expect(page.getByText("mode pantau")).toHaveCount(0);

    // Only wilayah may answer: not the operator (would skip wilayah), not pusat.
    for (const [role, action] of [
      ["operator_a", "submit"],
      ["validator_pusat", "approve"],
      ["validator_pusat", "reject"],
    ] as const) {
      expect(
        (await aksiPakaian(request, role, rowId, action)).status,
        `${role} ${action} at 1007`,
      ).toBe(400);
    }

    // Answer it THROUGH THE SCREEN: forward it again. The panel must then stop
    // offering the moves the row has just left — it used to keep its copy of the
    // row from when it was opened, so "Teruskan ke Pusat" stayed on a satker that
    // was already at Pusat.
    await page.getByRole("button", { name: "Teruskan ke Pusat", exact: true }).click();
    await page.getByRole("button", { name: "Kirim", exact: true }).click();
    await expect(
      page.getByRole("button", { name: "Teruskan ke Pusat", exact: true }),
    ).toHaveCount(0, { timeout: 20000 });
    await expect(page.getByRole("button", { name: "Kembalikan untuk Revisi" })).toHaveCount(0);
    await expect(page.getByText("mode pantau")).toBeVisible();
    await expect
      .poll(async () => {
        const r = await fetchRows<{ id: string; aktivitas_id: number }>(
          request,
          `${PD_API}/pengajuan/${pengajuanId}/satker?page=1&per_page=100`,
          pusat,
          "satker rows after wilayah forwards",
        );
        return r.find((x) => x.id === rowId)?.aktivitas_id;
      })
      .toBe(1004);

    // And Pusat can now close it.
    expect((await aksiPakaian(request, "validator_pusat", rowId, "approve")).aktivitas).toBe(1008);
  });
});

test.describe("Pakaian Dinas — a returned satker tells its operator why", () => {
  test.use({ storageState: storageStatePath("operator_a") });

  test("the pengisian page quotes the validator's note, and only while it is returned", async ({
    page,
    request,
  }) => {
    const { pengajuanId, rowId } = await kampanyeBaru(request);
    const isi = `${BASE}/pakaian-dinas/pengajuan/${pengajuanId}/satker/${SATKER_REVISI}/isi`;
    const note = `Ukuran celana tidak sesuai ${Date.now()}`;

    // A fresh satker was never returned: no banner.
    await page.goto(isi, { waitUntil: "domcontentloaded" });
    await expect(page.getByTestId("pengisian-tabel")).toBeVisible({ timeout: 20000 });
    await expect(page.getByTestId("pengisian-catatan-revisi")).toHaveCount(0);

    expect((await aksiPakaian(request, "operator_a", rowId, "submit")).aktivitas).toBe(1001);
    expect(
      (await aksiPakaian(request, "validator_wilayah", rowId, "reject", note)).aktivitas,
    ).toBe(1003);

    await page.goto(isi, { waitUntil: "domcontentloaded" });
    const banner = page.getByTestId("pengisian-catatan-revisi");
    await expect(banner).toBeVisible({ timeout: 20000 });
    await expect(banner, "the reviewer's own words").toContainText(note);
    await expect(banner, "and who said it").toContainText("validator_wilayah");

    // Fixed and resubmitted: the banner goes away with the state that earned it.
    expect((await aksiPakaian(request, "operator_a", rowId, "submit")).aktivitas).toBe(1001);
    await page.goto(isi, { waitUntil: "domcontentloaded" });
    await expect(page.getByTestId("pengisian-terkunci")).toBeVisible({ timeout: 20000 });
    await expect(page.getByTestId("pengisian-catatan-revisi")).toHaveCount(0);
  });
});

// ── Bulk approval ───────────────────────────────────────────────────────────
// Nationally a campaign has a few hundred satker, and the wilayah and pusat
// validators approved them one dialog at a time. The list now lets a validator
// tick the satker waiting on them and approve the lot in one request; each row
// still goes through the single approval's own checks.
async function kampanyeDuaSatker(
  request: APIRequestContext,
): Promise<{ pengajuanId: string; rows: string[] }> {
  // A window around today: whatever the calendar says when this runs.
  const tanggal = (selisih: number) =>
    new Date(Date.now() + selisih * 86_400_000).toISOString().slice(0, 10);
  const pusat = await tokenFor(request, "validator_pusat");
  const jenis = await fetchRows<{ id: string; nama: string }>(
    request, `${PD_API}/jenis?page=1&per_page=100`, pusat, "jenis list");
  const pdh = jenis.find((j) => j.nama.trim().toUpperCase() === "PDH");
  expect(pdh, "V011 seeds PDH").toBeTruthy();
  const spesifikasi = await fetchRows<{ id: string }>(
    request, `${PD_API}/spesifikasi?page=1&per_page=100&jenis_pakaian_dinas_id=${pdh!.id}`, pusat, "spesifikasi list");
  const created = await request.post(`${PD_API}/pengajuan`, {
    headers: { Authorization: `Bearer ${pusat}` },
    data: {
      nama: `E2E massal ${Date.now()}`, tahun: 2026,
      tgl_mulai: tanggal(-60), tgl_selesai: tanggal(300),
      is_reguler: true, pilihan_satker: "sebagian",
      jenis_pakaian_dinas_id: pdh!.id, spesifikasi_ids: [spesifikasi[0].id],
      satker_ids: ["0200010", "0200020"],
    },
  });
  expect(created.status(), await created.text()).toBe(201);
  const pengajuanId = (await created.json()).data.id as string;
  const rows = await fetchRows<{ id: string; satker_id: string }>(
    request, `${PD_API}/pengajuan/${pengajuanId}/satker?page=1&per_page=100`, pusat, "rows");
  expect(rows.length).toBe(2);
  return { pengajuanId, rows: rows.map((r) => r.id) };
}

test.describe("Pakaian Dinas — bulk approval", () => {
  test("wilayah forwards two satker with one click, then pusat approves both", async ({
    browser,
    request,
  }) => {
    const { pengajuanId, rows } = await kampanyeDuaSatker(request);
    expect((await aksiPakaian(request, "operator_a", rows[0], "submit")).aktivitas).toBe(1001);
    expect((await aksiPakaian(request, "operator_b", rows[1], "submit")).aktivitas).toBe(1001);
    const list = `${BASE}/pakaian-dinas/pengajuan/${pengajuanId}/satker`;
    const statuses = async () => {
      const pusat = await tokenFor(request, "validator_pusat");
      const r = await fetchRows<{ id: string; aktivitas_id: number }>(
        request, `${PD_API}/pengajuan/${pengajuanId}/satker?page=1&per_page=100`, pusat, "rows");
      return rows.map((id) => r.find((x) => x.id === id)!.aktivitas_id);
    };

    // The satker's operator is offered no bulk controls.
    const operator = await browser.newContext({ storageState: storageStatePath("operator_a") });
    const po = await operator.newPage();
    await po.goto(list, { waitUntil: "domcontentloaded" });
    await expect(po.getByRole("link", { name: "Isi Ukuran Pegawai" })).toBeVisible({ timeout: 20000 });
    await expect(po.getByTestId("bilah-massal")).toHaveCount(0);
    await expect(po.getByTestId("pilih-satker")).toHaveCount(0);
    await operator.close();

    // Wilayah ticks everything waiting on it and forwards the lot.
    const wilayah = await browser.newContext({ storageState: storageStatePath("validator_wilayah") });
    const pw = await wilayah.newPage();
    await pw.goto(list, { waitUntil: "domcontentloaded" });
    await expect(pw.getByTestId("bilah-massal")).toBeVisible({ timeout: 20000 });
    await expect(pw.getByTestId("setujui-massal")).toBeDisabled();
    await pw.getByTestId("pilih-semua").check();
    await expect(pw.getByText("2 dipilih")).toBeVisible();
    await pw.getByTestId("setujui-massal").click();
    await expect(pw.getByTestId("hasil-massal")).toContainText("2 satker diproses", { timeout: 15000 });
    expect(await statuses()).toEqual([1004, 1004]);
    await wilayah.close();

    // Pusat approves both the same way.
    const pusatCtx = await browser.newContext({ storageState: storageStatePath("validator_pusat") });
    const pp = await pusatCtx.newPage();
    await pp.goto(list, { waitUntil: "domcontentloaded" });
    await expect(pp.getByTestId("bilah-massal")).toBeVisible({ timeout: 20000 });
    await pp.getByTestId("pilih-semua").check();
    await pp.getByTestId("setujui-massal").click();
    await expect(pp.getByTestId("hasil-massal")).toContainText("2 satker diproses", { timeout: 15000 });
    expect(await statuses()).toEqual([1008, 1008]);
    await pusatCtx.close();
  });
});

// ── Handovers tell the next role ────────────────────────────────────────────
// A satker moving to the next role used to tell nobody: the work sat in a list
// until the next role happened to open it, and the dashboard's "Perlu Tindakan"
// did not count it. The next role is now notified, and the dashboard counts what
// waits on the caller.
test.describe("Pakaian Dinas — a handover tells the next role", () => {
  const NOTIF = `${PD_API.replace("/pakaian-dinas", "")}/notifikasi?page=1&per_page=50`;
  const inbox = async (request: APIRequestContext, role: Role) => {
    const token = await tokenFor(request, role);
    const res = await request.get(NOTIF, { headers: { Authorization: `Bearer ${token}` } });
    expect(res.ok(), await res.text()).toBeTruthy();
    return ((await res.json()).data ?? []) as { title: string; message: string }[];
  };
  const dikenal = async (request: APIRequestContext, role: Role) => {
    // Opening the campaign list is how a user becomes reachable.
    const token = await tokenFor(request, role);
    const res = await request.get(`${PD_API}/pengajuan?page=1&per_page=1`, {
      headers: { Authorization: `Bearer ${token}` },
    });
    expect(res.ok(), await res.text()).toBeTruthy();
  };

  test("wilayah is told when an operator submits, and the operator when it is returned", async ({
    request,
  }) => {
    await dikenal(request, "validator_wilayah");
    await dikenal(request, "operator_a");
    const { rowId } = await kampanyeBaru(request);
    const note = `Ukuran celana tidak sesuai ${Date.now()}`;

    expect((await aksiPakaian(request, "operator_a", rowId, "submit")).aktivitas).toBe(1001);
    const wilayah = await inbox(request, "validator_wilayah");
    expect(
      wilayah.some((n) => n.title === "Pakaian dinas menunggu pemeriksaan Anda"),
      "wilayah is told something is waiting",
    ).toBe(true);

    expect(
      (await aksiPakaian(request, "validator_wilayah", rowId, "reject", note)).aktivitas,
    ).toBe(1003);
    const operator = await inbox(request, "operator_a");
    expect(
      operator.some((n) => n.title.includes("dikembalikan") && n.message.includes(note)),
      "the operator is told, with the validator's own words",
    ).toBe(true);
  });

  test("the wilayah dashboard counts the satker waiting on it", async ({ browser, request }) => {
    await dikenal(request, "validator_wilayah");
    const { rowId } = await kampanyeBaru(request);
    expect((await aksiPakaian(request, "operator_a", rowId, "submit")).aktivitas).toBe(1001);

    const ctx = await browser.newContext({ storageState: storageStatePath("validator_wilayah") });
    const page = await ctx.newPage();
    await page.goto(`${BASE}/dashboard`, { waitUntil: "domcontentloaded" });
    const tile = page.getByTestId("menunggu-pakaian-dinas");
    await expect(tile).toBeVisible({ timeout: 30000 });
    await expect(tile).toContainText("Pakaian Dinas Menunggu");
    await expect(tile).toContainText("menunggu pemeriksaan Anda");
    await ctx.close();

    // A role with no turn in the flow gets no tile. approver_satker holds that one
    // role alone; the e2e "admin" is the base seed user in CI and holds every role
    // (validator_wilayah among them), so it rightly has a turn and a tile.
    const luar = await browser.newContext({ storageState: storageStatePath("approver_satker") });
    const pl = await luar.newPage();
    await pl.goto(`${BASE}/dashboard`, { waitUntil: "domcontentloaded" });
    await expect(pl.getByText("Perlu Tindakan")).toBeVisible({ timeout: 30000 });
    await expect(pl.getByTestId("menunggu-pakaian-dinas")).toHaveCount(0);
    await luar.close();
  });
});

// ── What the system already knows is not retyped ────────────────────────────
// A pegawai's sizes are on their own profile (the Ukuran page). The operator
// filling the satker's campaign was shown an empty dialog and typed them again.
test.describe("Pakaian Dinas — the dialog starts from the sizes already on record", () => {
  test.use({ storageState: storageStatePath("operator_a") });

  test("a pegawai with a profile is pre-filled, and saving keeps what the operator checked", async ({
    page,
    request,
  }) => {
    // operator_a is also the employee on the roster of 0200010: set their own profile.
    const token = await tokenFor(request, "operator_a");
    const auth = { Authorization: `Bearer ${token}` };
    const profil = await request.post(`${PD_API}/ukuran-pakaian-pegawai`, {
      headers: auth,
      data: { ukuran_baju: "L", ukuran_celana: "32", ukuran_sepatu: "41", with_hijab: false },
    });
    expect(profil.ok(), await profil.text()).toBeTruthy();

    const { pengajuanId } = await kampanyeBaru(request);
    await page.goto(
      `${BASE}/pakaian-dinas/pengajuan/${pengajuanId}/satker/${SATKER_REVISI}/isi`,
      { waitUntil: "domcontentloaded" },
    );
    await expect(page.getByTestId("pengisian-tabel")).toBeVisible({ timeout: 20000 });
    await page.getByRole("button", { name: /^Isi ukuran /i }).first().click();

    const dialog = page.getByTestId("dialog-ukuran");
    await expect(dialog.getByTestId("ukuran-diisi-otomatis")).toBeVisible();
    const selects = dialog.locator("select");
    expect(await selects.count()).toBeGreaterThan(0);
    for (let i = 0; i < (await selects.count()); i++) {
      await expect(selects.nth(i), "no column is left for the operator to retype").not.toHaveValue("");
    }

    // The operator checks and saves; what is stored is what the dialog showed.
    await page.getByTestId("simpan-ukuran").click();
    // A size is a cell of its own. Matching the table's flattened text with \b
    // missed it: "Penata Muda" + "32" run together as "Muda32", which has no boundary.
    await expect(
      page.getByTestId("pengisian-tabel").locator("td").filter({ hasText: /^\s*(L|32|41)\s*$/ }).first(),
    ).toBeVisible({ timeout: 15000 });
    const pusat = await tokenFor(request, "validator_pusat");
    const roster = await request.get(
      `${PD_API}/pengajuan/${pengajuanId}/satker/${SATKER_REVISI}/pegawai`,
      { headers: { Authorization: `Bearer ${pusat}` } },
    );
    const saved = (await roster.json()).data.pegawai.find(
      (p: { nip: string }) => p.nip === "200000000000000001",
    );
    expect(saved.sudah_diisi).toBe(true);
    expect(["L", "32", "41"]).toContain(saved.ukuran[0].ukuran);
  });
});

// ── Failures that used to be silent ─────────────────────────────────────────
// The two delete handlers and the export download ended their error arm in a
// console log. The server had answered with the reason (a campaign with work in
// it is not deleted; a jenis a campaign uses is not deleted) and the screen said
// nothing, so the operator pressed the button, confirmed, and watched nothing
// happen. An export that failed in the background did the same.
test.describe("Pakaian Dinas — a refused campaign delete says why", () => {
  test.use({ storageState: storageStatePath("validator_pusat") });

  test("a campaign with work in it is kept, and the screen gives the reason", async ({
    page,
    request,
  }) => {
    const { pengajuanId, rowId, pusat } = await kampanyeBaru(request);
    expect((await aksiPakaian(request, "operator_a", rowId, "submit")).aktivitas).toBe(1001);
    const detail = await request.get(`${PD_API}/pengajuan/${pengajuanId}`, {
      headers: { Authorization: `Bearer ${pusat}` },
    });
    const nama = (await detail.json()).data.nama as string;

    page.on("dialog", (d) => d.accept());
    // A short window, so the card's button is a scroll away from the banner above
    // the list — the case on any long list. The reason must be where the operator is
    // looking, not only somewhere on the page.
    await page.setViewportSize({ width: 1280, height: 300 });
    await page.goto(`${BASE}/pakaian-dinas/pengajuan`, { waitUntil: "domcontentloaded" });
    const card = page.locator("div.rounded-2xl", { has: page.locator("h3", { hasText: nama }) });
    await expect(card).toBeVisible({ timeout: 20000 });
    await card.first().locator("button").last().click();

    const banner = page.getByTestId("action-error");
    await expect(banner).toBeVisible({ timeout: 15000 });
    await expect(banner, "on screen, not above it").toBeInViewport({ ratio: 1 });
    await expect(banner).toContainText("tidak dapat dihapus");
    await expect(banner, "names the cause").toContainText("sudah mengajukan");

    // Nothing was thrown away: the card is still listed and the campaign still exists.
    await expect(card).toBeVisible();
    const kept = await request.get(`${PD_API}/pengajuan/${pengajuanId}`, {
      headers: { Authorization: `Bearer ${pusat}` },
    });
    expect(kept.status()).toBe(200);
  });
});

test.describe("Pakaian Dinas — a refused jenis delete says why", () => {
  test.use({ storageState: storageStatePath("admin") });

  test("a jenis that a campaign uses is kept, and the screen gives the reason", async ({
    page,
    request,
  }) => {
    // A campaign on PDH, so PDH is in use whatever ran before this test.
    await kampanyeBaru(request);

    page.on("dialog", (d) => d.accept());
    await page.setViewportSize({ width: 1280, height: 300 });
    await page.goto(`${BASE}/pakaian-dinas/jenis`, { waitUntil: "domcontentloaded" });
    const row = page.locator("tr", { has: page.getByText("PDH", { exact: true }) });
    await expect(row).toBeVisible({ timeout: 20000 });
    await row.getByTitle("Hapus").click();

    const banner = page.getByTestId("action-error");
    await expect(banner).toBeVisible({ timeout: 15000 });
    await expect(banner, "on screen, not above it").toBeInViewport({ ratio: 1 });
    await expect(banner, "the reason, not the database's wording").toContainText(
      "masih dipakai oleh pengajuan",
    );
    await expect(banner).not.toContainText("db error");
    await expect(row, "and the row is still there").toBeVisible();
  });
});

test.describe("Pakaian Dinas — a failed export is reported", () => {
  test.use({ storageState: storageStatePath("admin") });

  test("the print button says so when the server cannot produce the file", async ({ page }) => {
    // Hold the file back with a server error; the page must not just go quiet.
    await page.route("**/pakaian-dinas/laporan/cetak**", (route) =>
      route.fulfill({
        status: 500,
        contentType: "application/json",
        body: JSON.stringify({ success: false, message: "boom" }),
      }),
    );
    await page.goto(`${BASE}/pakaian-dinas/laporan`, { waitUntil: "domcontentloaded" });
    const periode = page.getByTestId("laporan-pengajuan");
    await expect
      .poll(() => periode.locator("option").count(), { timeout: 20000 })
      .toBeGreaterThan(1);
    await periode.selectOption({ index: 1 });
    await expect(page.getByTestId("laporan-cetak-pdf")).toBeEnabled();
    await expect(page.getByTestId("action-error")).toHaveCount(0);

    await page.getByTestId("laporan-cetak-pdf").click();
    const banner = page.getByTestId("action-error");
    await expect(banner).toBeVisible({ timeout: 15000 });
    await expect(banner).toContainText("Cetak PDF gagal");
  });
});

// ── Dates are enforced, and each role sees only its own controls ────────────
// A campaign outside its dates takes no entry and no decision; the pusat
// validator extends it. Until now the dates only drove a badge, so an operator
// could fill and submit a campaign that said "Ditutup". And every role was
// offered every button (an operator saw Tambah Jenis and the trash can of every
// campaign), each answering 403.
test.describe("Pakaian Dinas — pusat extends a closed campaign", () => {
  test.use({ storageState: storageStatePath("validator_pusat") });

  test("the end date is changed from the card and the campaign opens again", async ({
    page,
    request,
  }) => {
    const { pengajuanId, pusat } = await kampanyeBaru(request, {
      mulai: hari(-60),
      selesai: hari(-1),
    });
    const auth = { Authorization: `Bearer ${pusat}` };
    const nama = (await (await request.get(`${PD_API}/pengajuan/${pengajuanId}`, { headers: auth })).json())
      .data.nama as string;

    await page.goto(`${BASE}/pakaian-dinas/pengajuan`, { waitUntil: "domcontentloaded" });
    const card = page.locator("div.rounded-2xl", { has: page.locator("h3", { hasText: nama }) });
    await expect(card).toBeVisible({ timeout: 20000 });
    await expect(card, "a campaign past its end date says so").toContainText("Ditutup");

    await card.getByTestId("ubah-kegiatan").click();
    await card.getByTestId("ubah-selesai").fill(hari(30));
    await card.getByTestId("simpan-ubah").click();

    await expect(card).toContainText("Dibuka", { timeout: 15000 });
    await expect(card).toContainText(`Tutup: ${hari(30)}`);
    const body = (await (await request.get(`${PD_API}/pengajuan/${pengajuanId}`, { headers: auth })).json())
      .data;
    expect(body.tgl_selesai).toBe(hari(30));
    expect(body.is_open).toBe(true);
  });
});

test.describe("Pakaian Dinas — a closed campaign tells the operator why", () => {
  test.use({ storageState: storageStatePath("operator_a") });

  test("saving sizes after the end date shows the reason", async ({ page, request }) => {
    const { pengajuanId } = await kampanyeBaru(request, { mulai: hari(-60), selesai: hari(-1) });
    await page.goto(
      `${BASE}/pakaian-dinas/pengajuan/${pengajuanId}/satker/${SATKER_REVISI}/isi`,
      { waitUntil: "domcontentloaded" },
    );
    await expect(page.getByTestId("pengisian-tabel")).toBeVisible({ timeout: 20000 });
    await page.getByRole("button", { name: /^Isi ukuran /i }).first().click();
    const selects = page.getByTestId("dialog-ukuran").locator("select");
    for (let i = 0; i < (await selects.count()); i++) await selects.nth(i).selectOption({ index: 1 });
    await page.getByTestId("simpan-ukuran").click();
    await expect(page.getByText(/sudah ditutup/i).first()).toBeVisible({ timeout: 15000 });
  });
});

type Kontrol = { jenisTambah: number; kegiatanBuat: number; kegiatanUbah: number };

/**
 * The realm roles the session of `peran` really carries, read from the access
 * token its login stored. The screen offers a control for a ROLE, not for a
 * fixture name, and the "admin" session is not the same person everywhere: in CI
 * it is the base seed user, who holds every role (admin, validator_pusat, …),
 * while the staging fixture `…005` holds `admin` alone. Hard-coding what "admin"
 * sees made the spec right in one place and red in the other.
 */
function rolesOfSession(peran: string): string[] {
  const state = JSON.parse(readFileSync(storageStatePath(peran), "utf8")) as {
    origins: { localStorage: { name: string; value: string }[] }[];
  };
  for (const origin of state.origins) {
    const token = origin.localStorage.find((item) => item.name === "auth_token")?.value;
    if (!token) continue;
    const payload = JSON.parse(Buffer.from(token.split(".")[1], "base64url").toString("utf8")) as {
      realm_access?: { roles?: string[] };
    };
    return payload.realm_access?.roles ?? [];
  }
  throw new Error(`no stored access token for "${peran}" in ${storageStatePath(peran)}`);
}

/** Roles that hold exactly one of the two powers keep a fixed expectation; "admin" is derived from its token. */
function kontrolDari(peran: string): Kontrol {
  const roles = rolesOfSession(peran);
  const pusat = roles.includes("validator_pusat") ? 1 : 0;
  return { jenisTambah: roles.includes("admin") ? 1 : 0, kegiatanBuat: pusat, kegiatanUbah: pusat };
}

const KONTROL_PER_PERAN: [Role | "admin", () => Kontrol][] = [
  ["operator_a", () => ({ jenisTambah: 0, kegiatanBuat: 0, kegiatanUbah: 0 })],
  ["validator_wilayah", () => ({ jenisTambah: 0, kegiatanBuat: 0, kegiatanUbah: 0 })],
  ["validator_pusat", () => ({ jenisTambah: 0, kegiatanBuat: 1, kegiatanUbah: 1 })],
  ["admin", () => kontrolDari("admin")],
];
for (const [peran, kontrol] of KONTROL_PER_PERAN) {
  test.describe(`Pakaian Dinas — ${peran} sees only its own controls`, () => {
    test.use({ storageState: storageStatePath(peran) });

    test("the master and campaign pages offer what the role may do", async ({ page }) => {
      const harap = kontrol();
      await page.goto(`${BASE}/pakaian-dinas/jenis`, { waitUntil: "domcontentloaded" });
      await expect(page.getByRole("table")).toBeVisible({ timeout: 20000 });
      await expect(page.getByRole("button", { name: "Tambah Jenis", exact: true })).toHaveCount(
        harap.jenisTambah,
      );
      const hapusJenis = page.getByTitle("Hapus");
      if (harap.jenisTambah) expect(await hapusJenis.count()).toBeGreaterThan(0);
      else await expect(hapusJenis).toHaveCount(0);

      await page.goto(`${BASE}/pakaian-dinas/pengajuan`, { waitUntil: "domcontentloaded" });
      await expect(page.locator("div.rounded-2xl h3").first()).toBeVisible({ timeout: 20000 });
      await expect(page.getByRole("button", { name: "Buat Pengajuan", exact: true })).toHaveCount(
        harap.kegiatanBuat,
      );
      const ubah = page.getByTestId("ubah-kegiatan");
      if (harap.kegiatanUbah) expect(await ubah.count()).toBeGreaterThan(0);
      else await expect(ubah).toHaveCount(0);
      const hapusKegiatan = page.getByLabel("Hapus kegiatan");
      if (harap.kegiatanUbah) expect(await hapusKegiatan.count()).toBeGreaterThan(0);
      else await expect(hapusKegiatan).toHaveCount(0);
    });
  });
}

test.describe("Pakaian Dinas — the satker's sizes are filled by its operator and read by the rest", () => {
  test("the link and the page say which one you are", async ({ browser, request }) => {
    const { pengajuanId } = await kampanyeBaru(request);
    const list = `${BASE}/pakaian-dinas/pengajuan/${pengajuanId}/satker`;
    const isi = `${list}/${SATKER_REVISI}/isi`;

    const operator = await browser.newContext({ storageState: storageStatePath("operator_a") });
    const po = await operator.newPage();
    await po.goto(list, { waitUntil: "domcontentloaded" });
    await expect(po.getByRole("link", { name: "Isi Ukuran Pegawai" })).toBeVisible({ timeout: 20000 });
    await po.goto(isi, { waitUntil: "domcontentloaded" });
    await expect(po.getByTestId("pengisian-tabel")).toBeVisible({ timeout: 20000 });
    await expect(po.getByTestId("pengisian-pemantau")).toHaveCount(0);
    await operator.close();

    const wilayah = await browser.newContext({ storageState: storageStatePath("validator_wilayah") });
    const pw = await wilayah.newPage();
    await pw.goto(list, { waitUntil: "domcontentloaded" });
    await expect(pw.getByRole("link", { name: "Lihat Ukuran Pegawai" })).toBeVisible({ timeout: 20000 });
    await expect(pw.getByRole("link", { name: "Isi Ukuran Pegawai" })).toHaveCount(0);
    await pw.goto(isi, { waitUntil: "domcontentloaded" });
    await expect(pw.getByTestId("pengisian-tabel")).toBeVisible({ timeout: 20000 });
    await expect(pw.getByTestId("pengisian-pemantau")).toBeVisible();
    await expect(pw.getByTestId("ajukan-pengisian")).toHaveCount(0);
    await expect(pw.getByRole("button", { name: /^Isi ukuran /i }).first()).toBeDisabled();
    await wilayah.close();
  });
});

// ── RBAC: the validator-action endpoint is role-gated ───────────────────────
test.describe("Pakaian Dinas — role-gated validator action", () => {
  test("an operator cannot drive a validator action", async ({ request }) => {
    const op = await tokenFor(request, "operator_a");
    const resp = await request.post(`${PD_API}/validator-action`, {
      headers: { Authorization: `Bearer ${op}` },
      data: {
        pengajuan_satker_id: "d1000000-0000-4d00-8d00-0000000a0001",
        aksi: "approve",
        komentar: "e2e role-gate probe",
      },
    });
    expect(resp.status(), "operator blocked from validator-action").toBeGreaterThanOrEqual(400);
  });
});
