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
    const token = await apiLogin(request, credsFor(userFor("operator_a")));
    const res = await request.get(
      `${PD_API}/pengajuan/${PENGAJUAN}/satker/${SATKER}/pegawai`,
      { headers: { Authorization: `Bearer ${token}` } },
    );
    expect(res.ok(), "roster endpoint must answer").toBeTruthy();
    const data = (await res.json()).data;
    // Every person in the satker appears, whether or not they have been filled
    // in yet — a roster that only lists people already saved cannot be used to
    // add the ones who are missing.
    expect(Array.isArray(data.pegawai)).toBeTruthy();
    expect(data.satker_kode).toBe(SATKER);
    expect(typeof data.dapat_diubah).toBe("boolean");

    await page.goto(ISI, { waitUntil: "domcontentloaded" });
    await expect(page.getByTestId("pengisian-tabel")).toBeVisible({ timeout: 20000 });
  });

  test("another satker's roster is not reachable", async ({ request }) => {
    // 0200020 belongs to operator_b. Out of scope answers NotFound rather than
    // Forbidden: a 403 would confirm the row exists under another satker.
    const token = await apiLogin(request, credsFor(userFor("operator_a")));
    const res = await request.get(
      `${PD_API}/pengajuan/${PENGAJUAN}/satker/0200020/pegawai`,
      { headers: { Authorization: `Bearer ${token}` } },
    );
    expect(res.status(), "cross-satker roster must not be readable").toBe(404);
  });

  test("a satker past its own stage cannot have sizes written", async ({ request }) => {
    // Jakpus on this campaign sits at 1004 (Diajukan ke Pusat). Writing must be
    // refused by the SERVER, not merely by a disabled button.
    const token = await apiLogin(request, credsFor(userFor("operator_a")));
    const roster = await request.get(
      `${PD_API}/pengajuan/${PENGAJUAN}/satker/${SATKER}/pegawai`,
      { headers: { Authorization: `Bearer ${token}` } },
    );
    const data = (await roster.json()).data;
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
    const token = await apiLogin(request, credsFor(userFor("operator_a")));
    const roster = await request.get(
      `${PD_API}/pengajuan/${PENGAJUAN}/satker/${SATKER}/pegawai`,
      { headers: { Authorization: `Bearer ${token}` } },
    );
    const data = (await roster.json()).data;
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
    const bySatker = page.waitForResponse((r) => r.url().includes("/laporan/rekap-ukuran"));
    await page.getByTestId("laporan-satker").fill(SATKER_JAKPUS);
    await page.getByTestId("laporan-satker").blur();
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
