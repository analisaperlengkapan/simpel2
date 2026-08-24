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
import { test, expect } from "@playwright/test";
import { apiLogin, credsFor, storageStatePath, TEST_USERS, PERLENGKAPAN_API_URL } from "./helpers/real-auth";
import { reachable } from "./helpers/page-load";

const BASE = "/perlengkapan/simpel/v2";
const PD_API = `${PERLENGKAPAN_API_URL}/api/v1/perlengkapan/pakaian-dinas`;

const userFor = (key: string) => {
  const u = TEST_USERS.find((t) => t.key === key);
  if (!u) throw new Error(`unknown test user ${key}`);
  return u;
};

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
    await page.getByTestId("laporan-pengajuan").selectOption(CAMPAIGN);
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
        await page.getByTestId("laporan-pengajuan").selectOption(CAMPAIGN);
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
        expect(search).toContain(`pengajuan_id=${CAMPAIGN}`);

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
  const { accessToken } = await apiLogin(request, credsFor(userFor("validator_pusat")));
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
    const { accessToken } = await apiLogin(request, credsFor(userFor("validator_pusat")));
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

    const wil = await apiLogin(request, credsFor(userFor("validator_wilayah")));
    const resp = await request.post(`${PD_API}/validator-action`, {
      headers: { Authorization: `Bearer ${wil.accessToken}` },
      data: { pengajuan_satker_id: P1_WILAYAH, aksi: "approve", komentar: "e2e teruskan" },
    });
    expect(resp.status(), "validator_wilayah may approve at 1001").toBe(200);

    await expect.poll(() => beAktivitas(request, P1_WILAYAH), { timeout: 15000 }).toBe(1004);
  });
});

test.describe("Pakaian Dinas — validator pusat decides", () => {
  test("validator_pusat approves P2 (1004 → 1008 Selesai)", async ({ request }) => {
    expect(await beAktivitas(request, P2_PUSAT)).toBe(1004);

    const pusat = await apiLogin(request, credsFor(userFor("validator_pusat")));
    const resp = await request.post(`${PD_API}/validator-action`, {
      headers: { Authorization: `Bearer ${pusat.accessToken}` },
      data: { pengajuan_satker_id: P2_PUSAT, aksi: "approve", komentar: "e2e setujui" },
    });
    expect(resp.status(), "validator_pusat may approve at 1004").toBe(200);

    await expect.poll(() => beAktivitas(request, P2_PUSAT), { timeout: 15000 }).toBe(1008);
  });
});

// ── RBAC: the validator-action endpoint is role-gated ───────────────────────
test.describe("Pakaian Dinas — role-gated validator action", () => {
  test("an operator cannot drive a validator action", async ({ request }) => {
    const op = await apiLogin(request, credsFor(userFor("operator_a")));
    const resp = await request.post(`${PD_API}/validator-action`, {
      headers: { Authorization: `Bearer ${op.accessToken}` },
      data: {
        pengajuan_satker_id: "d1000000-0000-4d00-8d00-0000000a0001",
        aksi: "approve",
        komentar: "e2e role-gate probe",
      },
    });
    expect(resp.status(), "operator blocked from validator-action").toBeGreaterThanOrEqual(400);
  });
});
