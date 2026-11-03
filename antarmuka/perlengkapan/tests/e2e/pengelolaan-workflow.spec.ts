/**
 * Pengelolaan BMN (Pemakaian + Penghapusan) — e2e (F-E2E E-3). Reuses the E-1
 * template: drive the wired UI through a real multi-role workflow and assert
 * the resulting state in BOTH the browser and the backend.
 *
 * Penghapusan is the flagship here — its per-action RBAC is the cleanest in
 * perlengkapan (require_role per transition) and its RBAC data-visibility is
 * the authoritative MySIMKARI `satker_code` (V003 + SatkerScope), with NO
 * integrasi.mysimkari_satker join → #94-safe. Status model (4000-series):
 *   4000 DRAFT → 4001 SUBMIT_WILAYAH → 4003 SUBMIT_PUSAT → 4004
 *   VERIFIKASI_PUSAT → 4005 KONSEP_SK_GENERATED → 4006 SK_SIGNED → 4007 DONE.
 * Seed rows at distinct statuses (deep-linked by UUID, seed-multisatker.sql):
 *   H1 0200010 @4000 → operator_a clicks "Ajukan ke Validator Wilayah"
 *   H2 0200010 @4001 → validator_wilayah clicks "Teruskan ke Validator Pusat"
 *   H3 0200020 @4003 → validator_pusat verifies (API) + "Generate Konsep SK" (UI)
 *
 * Both FE gaps this spec used to document are now CLOSED, and the tests that
 * recorded them have become tests that drive them:
 *   - #95 gave penghapusan a UI for every transition, so verifikasi-pusat is
 *     no longer an API-only step.
 *   - #96 seeded the `validator_satker` / `approver_satker` roles (authenc
 *     migration 004) and wired the pemakaian approval buttons.
 * In both cases the buttons are rendered from the backend's own
 * `allowed_transitions`, already narrowed to the caller's role — so a button
 * appearing IS the assertion that the server would accept it.
 *
 * Pemakaian (izin_pemakaian_bmn) coverage: the satker-internal chain
 * (Submitted→SubmittedApproverSatker→Approved) driven through the UI by
 * validator_satker then approver_satker; plus the list + detail render of the
 * seeded ACTIVE izin, list scoping by satker_code (#70), and the revoke policy
 * rejections (approver-only; admin explicitly BLOCKED by stakeholder mandate
 * via enforce_no_admin_revoke).
 */
import { test, expect, type APIRequestContext, type Page } from "@playwright/test";
import { apiLogin, credsFor, storageStatePath, TEST_USERS, PERLENGKAPAN_API_URL } from "./helpers/real-auth";
import { clickAction } from "./helpers/workflow";
import { reachable } from "./helpers/page-load";

const BASE = "/perlengkapan/simpel/v2";
const PH_API = `${PERLENGKAPAN_API_URL}/api/v1/perlengkapan/penghapusan-bmn`;
const PM_API = `${PERLENGKAPAN_API_URL}/api/v1/perlengkapan/pemakaian-bmn`;

const H1_OPERATOR = "e1000000-0000-4e00-8e00-0000000a0001"; // 0200010 @4000
const H2_WILAYAH = "e1000000-0000-4e00-8e00-0000000a0002"; //  0200010 @4001
const H3_PUSAT = "e1000000-0000-4e00-8e00-0000000a0003"; //    0200020 @4003
const I1_IZIN = "e2000000-0000-4e00-8e00-0000000b0001"; //     0200010 ACTIVE
const I2_IZIN = "e2000000-0000-4e00-8e00-0000000b0002"; //     0200010 @3001

const userFor = (key: string) => {
  const u = TEST_USERS.find((t) => t.key === key);
  if (!u) throw new Error(`unknown test user ${key}`);
  return u;
};

/** The authenticated app shell mounts a sticky <header> containing "SIMPEL". */
const shell = (page: import("@playwright/test").Page) => page.locator("header").getByText("SIMPEL").first();

async function openDetail(page: import("@playwright/test").Page, id: string) {
  await page.goto(`${BASE}/pengelolaan/penghapusan/detail/${id}`, {
    waitUntil: "domcontentloaded",
  });
  await expect(shell(page), "app shell mounts").toBeVisible({ timeout: 20000 });
}

/**
 * Backend source-of-truth: a penghapusan row's status_kode via
 * GET /penghapusan-bmn/{id} as validator_pusat (cross-satker, sees every row).
 */
async function phRow(
  request: APIRequestContext,
  id: string,
): Promise<{ status_kode: number; konsep_sk_url: string | null }> {
  const { accessToken } = await apiLogin(request, credsFor(userFor("validator_pusat")));
  const resp = await request.get(`${PH_API}/${id}`, {
    headers: { Authorization: `Bearer ${accessToken}` },
  });
  expect(resp.ok(), `GET penghapusan-bmn/${id} (${resp.status()})`).toBeTruthy();
  const body = await resp.json();
  return body.data;
}

const phStatusKode = async (request: APIRequestContext, id: string) => (await phRow(request, id)).status_kode;

/** IDs visible to a caller through the scoped penghapusan list (#70). */
async function phListIds(request: APIRequestContext, userKey: string): Promise<string[]> {
  const { accessToken } = await apiLogin(request, credsFor(userFor(userKey)));
  const resp = await request.get(`${PH_API}?page=1&per_page=100`, {
    headers: { Authorization: `Bearer ${accessToken}` },
  });
  expect(resp.ok(), `GET penghapusan-bmn list as ${userKey} (${resp.status()})`).toBeTruthy();
  const body = await resp.json();
  return (body.data ?? []).map((r: { id: string }) => r.id);
}

// ── Penghapusan: operator submits the DRAFT usulan to Validator Wilayah ─────
test.describe("Penghapusan BMN — operator submits to wilayah", () => {
  test.use({ storageState: storageStatePath("operator_a") });

  test("operator_a submits H1 (4000 → 4001) via the detail page", async ({ page, request }) => {
    expect(await phStatusKode(request, H1_OPERATOR)).toBe(4000);

    await openDetail(page, H1_OPERATOR);
    await expect(page.getByText("E2E Laptop Hapus A").first()).toBeVisible({ timeout: 20000 });

    await clickAction(
      page,
      page.getByRole("button", { name: "Ajukan ke Validator Wilayah" }),
      "submit-wilayah",
    );

    // UI reflects the transition: the operator submit action is gone after reload
    await page.reload({ waitUntil: "domcontentloaded" });
    await expect(shell(page)).toBeVisible({ timeout: 20000 });
    await expect(page.getByRole("button", { name: "Ajukan ke Validator Wilayah" })).toHaveCount(0);

    // backend source-of-truth: H1 moved 4000 → 4001
    await expect.poll(() => phStatusKode(request, H1_OPERATOR), { timeout: 15000 }).toBe(4001);
  });
});

// ── Penghapusan: validator wilayah forwards to Validator Pusat ──────────────
test.describe("Penghapusan BMN — validator wilayah forwards to pusat", () => {
  test.use({ storageState: storageStatePath("validator_wilayah") });

  test("validator_wilayah forwards H2 (4001 → 4003)", async ({ page, request }) => {
    expect(await phStatusKode(request, H2_WILAYAH)).toBe(4001);

    await openDetail(page, H2_WILAYAH);
    await clickAction(
      page,
      page.getByRole("button", { name: "Teruskan ke Validator Pusat" }),
      "validator-wilayah",
    );

    await page.reload({ waitUntil: "domcontentloaded" });
    await expect(shell(page)).toBeVisible({ timeout: 20000 });
    await expect(page.getByRole("button", { name: "Teruskan ke Validator Pusat" })).toHaveCount(0);

    // backend: H2 forwarded to pusat (4003)
    await expect.poll(() => phStatusKode(request, H2_WILAYAH), { timeout: 15000 }).toBe(4003);
  });
});

// ── Penghapusan: pusat verifies (API — FE gap) then generates konsep SK (UI) ─
test.describe("Penghapusan BMN — pusat verifies + generates konsep SK", () => {
  test.use({ storageState: storageStatePath("validator_pusat") });

  test("validator_pusat verifies H3 (API) and generates the konsep SK (UI)", async ({ page, request }) => {
    expect(await phStatusKode(request, H3_PUSAT)).toBe(4003);

    // Step 1 — verifikasi (4003 → 4004) via the API. The FE has no caller for
    // this validator_pusat-gated endpoint (see the header comment): through
    // the UI the workflow strands at 4003, so the SK surface below would be
    // unreachable. Driving the missing step via the real endpoint keeps the
    // rest of the chain honest while the FE gap is tracked.
    const pusat = await apiLogin(request, credsFor(userFor("validator_pusat")));
    const verif = await request.post(`${PH_API}/${H3_PUSAT}/verifikasi-pusat`, {
      headers: { Authorization: `Bearer ${pusat.accessToken}` },
      data: { catatan: "e2e verifikasi pusat" },
    });
    expect(verif.ok(), `verifikasi-pusat (${verif.status()})`).toBeTruthy();
    await expect.poll(() => phStatusKode(request, H3_PUSAT), { timeout: 15000 }).toBe(4004);

    // Step 2 — Generate Konsep SK via the UI (the button renders only when the
    // backend says can_generate_sk, i.e. at 4004).
    await openDetail(page, H3_PUSAT);
    const generate = page.getByRole("button", { name: "Generate Konsep SK" });
    await expect(generate).toBeVisible({ timeout: 20000 });
    await generate.click();

    // backend: 4004 → 4005 and the rendered konsep-SK URL is persisted
    // (konsep_sk_url = the DOCX download URL; the PDF path is server-side only)
    await expect.poll(() => phStatusKode(request, H3_PUSAT), { timeout: 30000 }).toBe(4005);
    const row = await phRow(request, H3_PUSAT);
    expect(row.konsep_sk_url, "konsep SK URL persisted").toBeTruthy();

    // the konsep SK PDF actually downloads (real bytes, not just a URL column)
    const pdf = await request.get(`${PH_API}/${H3_PUSAT}/konsep-sk.pdf`, {
      headers: { Authorization: `Bearer ${pusat.accessToken}` },
    });
    expect(pdf.ok(), `download konsep-sk.pdf (${pdf.status()})`).toBeTruthy();
    expect((await pdf.body()).length, "PDF is non-empty").toBeGreaterThan(0);
  });
});


// ── Penghapusan: SIMAN cross-check identifies an asset by all THREE parts ───
//
// `verifikasi-siman` is what a validator reads before an SK is issued, so its
// verdict has to be about the right asset. It used to look the asset up by NUP
// alone; NUP is a sequence number within one barang code at one satker, so on
// real data that returned some other satker's asset (asking about NUP 677 at
// 006010199005016000KP — a "Kursi Kayu" — returned a "Pistol" from
// 006010199419345000KP) and the barang codes then disagreed for an expected
// 90% of assets. Each of the three fixture records covers one verdict.
test.describe("Penghapusan BMN — verifikasi SIMAN by kode satker + kode barang + NUP", () => {
  async function verifikasi(request: APIRequestContext, id: string) {
    const { accessToken } = await apiLogin(request, credsFor(userFor("validator_pusat")));
    const resp = await request.get(`${PH_API}/${id}/verifikasi-siman`, {
      headers: { Authorization: `Bearer ${accessToken}` },
    });
    expect(resp.ok(), `GET verifikasi-siman/${id} (${resp.status()})`).toBeTruthy();
    return (await resp.json()).data;
  }

  test("H1 matches its own satker's asset, dotted kode barang and all", async ({ request }) => {
    const v = await verifikasi(request, H1_OPERATOR);
    expect(v.ditemukan, "asset found").toBe(true);
    // The usulan carries `3.10.01.02.003`; SIMAN stores `3100102003`. This
    // assertion is what fails if either half of the comparison stops being
    // normalised — the query would match nothing and report "tidak ditemukan".
    expect(v.kode_barang_cocok, "dotted usulan matches undotted SIMAN").toBe(true);
    expect(v.layak_lanjut, "safe to proceed").toBe(true);
    // nama barang = the STANDARD name that belongs to the barang code
    // (`ur_sskel`), never the SIMAN operator's own label for the item.
    expect(v.nama_barang_siman).toBe("Note Book");
  });

  test("H2 finds the NUP at its own satker but under a different barang code", async ({ request }) => {
    const v = await verifikasi(request, H2_WILAYAH);
    expect(v.ditemukan, "an asset with that NUP exists here").toBe(true);
    expect(v.kode_barang_cocok).toBe(false);
    expect(v.layak_lanjut, "validator must check manually").toBe(false);
    // Reachable ONLY because satker is part of the key: keyed on NUP alone the
    // fallback would have matched an arbitrary row anywhere in the country.
    expect(v.kode_barang_siman).toBe("3020101003");
    expect(v.pesan).toContain("0200010");
  });

  test("H3 reports a genuine absence, naming all three parts", async ({ request }) => {
    const v = await verifikasi(request, H3_PUSAT);
    expect(v.ditemukan).toBe(false);
    expect(v.layak_lanjut).toBe(false);
    expect(v.kode_barang_siman).toBeNull();
    // The message has to name the satker and barang code too — "NUP E2E-H-3"
    // on its own does not identify anything a validator could go and check.
    expect(v.pesan).toContain("0200020");
    expect(v.pesan).toContain("3.05.02.01.002");
  });
});

// ── Penghapusan: list is satker_code-scoped per role (#70) ───────────────────
test.describe("Penghapusan BMN — list scoping by satker_code", () => {
  test("operators see only their own satker; pusat sees all", async ({ request }) => {
    // operator_a (0200010) sees H1/H2, never operator_b's H3 (0200020)
    const a = await phListIds(request, "operator_a");
    expect(a, "operator_a sees own H1").toContain(H1_OPERATOR);
    expect(a, "operator_a must NOT see 0200020 row").not.toContain(H3_PUSAT);

    // operator_b (0200020) sees H3, never 0200010 rows
    const b = await phListIds(request, "operator_b");
    expect(b, "operator_b sees own H3").toContain(H3_PUSAT);
    expect(b, "operator_b must NOT see 0200010 rows").not.toContain(H1_OPERATOR);
    expect(b, "operator_b must NOT see 0200010 rows").not.toContain(H2_WILAYAH);

    // validator_pusat is cross-satker: sees every seeded row
    const p = await phListIds(request, "validator_pusat");
    for (const id of [H1_OPERATOR, H2_WILAYAH, H3_PUSAT]) {
      expect(p, `pusat sees ${id}`).toContain(id);
    }
  });
});

// ── Penghapusan: transitions are role-gated (require_role per endpoint) ──────
test.describe("Penghapusan BMN — role-gated transitions", () => {
  test("wrong-role callers cannot drive transitions", async ({ request }) => {
    // operator may NOT perform the wilayah-only forward (require_role check
    // runs before any state check, so the rejection is role-driven).
    const op = await apiLogin(request, credsFor(userFor("operator_a")));
    const r1 = await request.post(`${PH_API}/${H2_WILAYAH}/forward-pusat`, {
      headers: { Authorization: `Bearer ${op.accessToken}` },
      data: { aksi: "forward", catatan: "e2e role-gate probe" },
    });
    expect(r1.status(), "operator blocked from forward-pusat").toBeGreaterThanOrEqual(400);

    // validator_wilayah may NOT perform the pusat-only verifikasi
    const wil = await apiLogin(request, credsFor(userFor("validator_wilayah")));
    const r2 = await request.post(`${PH_API}/${H3_PUSAT}/verifikasi-pusat`, {
      headers: { Authorization: `Bearer ${wil.accessToken}` },
      data: { catatan: "e2e role-gate probe" },
    });
    expect(r2.status(), "wilayah blocked from verifikasi-pusat").toBeGreaterThanOrEqual(400);
  });
});

// ── Pemakaian: list + detail render the seeded ACTIVE izin ──────────────────
test.describe("Izin Pemakaian BMN — list & detail render seeded data", () => {
  test.use({ storageState: storageStatePath("operator_a") });

  test("the list shows the seeded izin and the detail offers lifecycle actions", async ({ page }) => {
    await page.goto(`${BASE}/pengelolaan/pemakaian`, { waitUntil: "domcontentloaded" });
    await expect(shell(page)).toBeVisible({ timeout: 20000 });
    await expect(page.getByText("E2E Laptop Dinas Pinjam").first()).toBeVisible({
      timeout: 20000,
    });

    await page.goto(`${BASE}/pengelolaan/pemakaian/detail/${I1_IZIN}`, {
      waitUntil: "domcontentloaded",
    });
    await expect(shell(page)).toBeVisible({ timeout: 20000 });
    await expect(page.getByText("E2E Laptop Dinas Pinjam").first()).toBeVisible({
      timeout: 20000,
    });
    // ACTIVE permits expose the lifecycle actions (Perpanjang / Cabut Izin)
    await expect(page.getByRole("button", { name: "Cabut Izin" })).toBeVisible({
      timeout: 20000,
    });
  });
});

// ── Pemakaian: list scoping + revoke policy ──────────────────────────────────
test.describe("Izin Pemakaian BMN — scoping & revoke policy", () => {
  test("the izin list is satker_code-scoped (#70)", async ({ request }) => {
    const listIds = async (userKey: string) => {
      const { accessToken } = await apiLogin(request, credsFor(userFor(userKey)));
      const resp = await request.get(`${PM_API}?page=1&per_page=100`, {
        headers: { Authorization: `Bearer ${accessToken}` },
      });
      expect(resp.ok(), `GET pemakaian-bmn list as ${userKey} (${resp.status()})`).toBeTruthy();
      const body = await resp.json();
      // Assert the envelope shape before mapping. `?? []` only covers an ABSENT
      // `data`; when the BE nested the page struct here (data.data) this line
      // threw an opaque "(...).map is not a function" that named neither the
      // endpoint nor the mismatch. Contract: flat PaginatedResponse.
      expect(
        Array.isArray(body.data),
        `pemakaian-bmn list must return a flat PaginatedResponse (data: []) — got data of type ` +
          `${Array.isArray(body.data) ? "array" : typeof body.data}: ${JSON.stringify(body.data)?.slice(0, 200)}`,
      ).toBeTruthy();
      return body.data.map((r: { id: string }) => r.id);
    };

    expect(await listIds("operator_a"), "operator_a sees own izin").toContain(I1_IZIN);
    expect(await listIds("operator_b"), "operator_b must NOT see 0200010 izin").not.toContain(I1_IZIN);
  });

  test("revoke is rejected for non-approver roles AND for admin", async ({ request }) => {
    // PemakaianBmnPolicy: Revoke on ACTIVE = approver_satker ONLY. Note the
    // approver_satker/validator_satker roles are absent from the authenc role
    // seed (tracked finding), so today NO principal passes this gate — both
    // probes assert the fail-closed behavior that IS enforced.
    const op = await apiLogin(request, credsFor(userFor("operator_a")));
    const r1 = await request.post(`${PM_API}/${I1_IZIN}/revoke`, {
      headers: { Authorization: `Bearer ${op.accessToken}` },
      data: { alasan: "e2e revoke probe" },
    });
    expect(r1.status(), "operator blocked from revoke").toBeGreaterThanOrEqual(400);

    // Admin is EXPLICITLY blocked from revoke (stakeholder mandate,
    // enforce_no_admin_revoke) — the one action where admin has no bypass.
    const admin = await apiLogin(request); // default = the all-role seed user (admin)
    const r2 = await request.post(`${PM_API}/${I1_IZIN}/revoke`, {
      headers: { Authorization: `Bearer ${admin.accessToken}` },
      data: { alasan: "e2e admin revoke probe" },
    });
    expect(r2.status(), "admin blocked from revoke (no-admin-revoke mandate)").toBeGreaterThanOrEqual(400);

    // the izin is untouched — still ACTIVE
    const pusat = await apiLogin(request, credsFor(userFor("validator_pusat")));
    const detail = await request.get(`${PM_API}/${I1_IZIN}`, {
      headers: { Authorization: `Bearer ${pusat.accessToken}` },
    });
    expect(detail.ok(), `GET izin ${I1_IZIN} (${detail.status()})`).toBeTruthy();
    const body = await detail.json();
    expect(body.data.status, "izin still ACTIVE after rejected revokes").toBe("ACTIVE");
  });
});

// ── Create-form pages ────────────────────────────────────────────────────────
// (Full literal paths so the route-coverage gate detects these as visited.)
test.describe("Pengelolaan — create-form pages", () => {
  test.use({ storageState: storageStatePath("operator_a") });

  test("penghapusan create form is reachable", async ({ page }) => {
    await reachable(page, `${BASE}/pengelolaan/penghapusan/buat`);
  });

  // The pemakaian form used to be checked with `reachable()` alone — the
  // authenticated shell mounts, therefore green. That proves nothing about the
  // step the form exists for. It now drives the asset picker end to end.
  //
  // The picker replaced a NUP text field, and that changed what these tests can
  // assert. A NUP does not identify an asset: it is a sequence number WITHIN one
  // satker for one kode barang, so 44 017 assets across 553 satkers carry NUP
  // `1`. The picker returns identified ROWS instead, and the search is scoped to
  // the caller's satker on the server — which is why the cross-satker case below
  // now asserts an empty result rather than an error message.

  /**
   * Search one picker and pick the single row it returns.
   *
   * `slug` addresses a specific picker: a form may mount several (one per extra
   * BMN row), and every id/testid carries the slug. It also forces the label
   * match to be exact — `getByLabel` is substring + case-insensitive, so a bare
   * "Cari BMN" would also match every "Cari BMN tambahan" box on the page.
   */
  async function pickAset(page: Page, needle: string, slug = "aset") {
    // Pemakaian cards (`aset-<n>`) and the single penghapusan picker (`aset`)
    // all carry "Cari BMN"; only penghapusan's extra rows say "tambahan".
    const label = slug.startsWith("aset-tambahan") ? "Cari BMN tambahan" : "Cari BMN";
    const kotak = page.locator(`#${slug}-cari`);
    // The label must point at THIS picker's box. Duplicated ids would make the
    // second row's label focus the first row's input, so assert the pairing
    // before typing into it.
    await expect(page.getByLabel(label, { exact: true }).and(kotak)).toHaveCount(1);
    await kotak.fill(needle);
    const hasil = page.getByTestId(`${slug}-hasil`);
    await expect(hasil, `picker returned nothing for ${needle}`).toBeVisible({
      timeout: 20000,
    });
    const rows = hasil.locator("li");
    await expect(rows, "the seed puts exactly one asset behind each marker").toHaveCount(1);
    await rows.first().getByRole("button").click();
  }

  test("pemakaian form resolves the asset, then reports it free", async ({ page }) => {
    await page.goto(`${BASE}/pengelolaan/pemakaian/buat`, { waitUntil: "domcontentloaded" });
    await pickAset(page, "E2E-A-1", "aset-0");

    // The identity the backend was actually asked about, rendered so the
    // operator can see which asset the system means. Structural locator, not
    // getByText: substring + case-insensitive matching has passed against the
    // wrong element here before.
    const identitas = page.getByTestId("aset-0-terpilih");
    await expect(identitas, "picked asset never rendered").toBeVisible({ timeout: 20000 });
    await expect(identitas, "kode barang must come from SIMAN kd_brg").toContainText("3020101003");
    await expect(identitas).toContainText("E2E-A-1");

    // Availability is checked on pick — no separate button to press, because
    // the question needs the kode barang the picked row carries.
    // E2E-A-1 carries permit I2 at SUBMITTED, not ACTIVE — so it is free.
    await expect(page.getByTestId("bmn-ketersediaan-0")).toContainText("tersedia", {
      timeout: 20000,
    });
  });

  test("pemakaian form reports an asset its own satker already holds", async ({ page }) => {
    await page.goto(`${BASE}/pengelolaan/pemakaian/buat`, { waitUntil: "domcontentloaded" });
    await pickAset(page, "E2E-A-2", "aset-0");

    const identitas = page.getByTestId("aset-0-terpilih");
    await expect(identitas).toBeVisible({ timeout: 20000 });
    await expect(identitas).toContainText("3100102003");

    // Permit I1 is ACTIVE on this asset, held by the caller's own colleague —
    // inside the satker the holder MUST be named, or the operator has no way
    // to know who to ask.
    const status = page.getByTestId("bmn-ketersediaan-0");
    await expect(status).toContainText("sedang digunakan", { timeout: 20000 });
    await expect(status, "in-satker holder must be named").toContainText("E2E Operator Jakpus");
  });

  // Permit I1 holds E2E-A-2 from 2026-01-01 to 2026-12-31. The form asked only
  // "is it held?", so it said "sedang digunakan" for ANY period, and the server
  // refused a permit that starts years after I1 ends — although `cek-bmn` has
  // promised "pemakaian berurutan" (an existing permit that ends before the new
  // one starts is fine) since it was written. The screen said no to a request
  // the rules allow.
  //
  // Both windows are FIXED: the seed's permit is dated, so a clock-relative
  // window would stop overlapping it when the calendar leaves 2026.
  test("pemakaian form tells a period that follows the held one from one that overlaps it", async ({
    page,
  }) => {
    await page.goto(`${BASE}/pengelolaan/pemakaian/buat`, { waitUntil: "domcontentloaded" });
    await page.locator("#pegawai-nip").fill("200000000000000001");
    await expect(page.getByTestId("pegawai-terpilih")).toBeVisible({ timeout: 20000 });

    // Period first, then the asset: the answer is about THIS period.
    await page.locator("#periode-mulai").fill("2031-03-01");
    await page.locator("#periode-selesai").fill("2031-03-10");
    await pickAset(page, "E2E-A-2", "aset-0");

    const status = page.getByTestId("bmn-ketersediaan-0");
    await expect(status, "a period after the held one is bookable").toHaveAttribute(
      "data-status",
      "PEMAKAIAN_BERURUTAN",
      { timeout: 20000 },
    );
    await expect(status).toContainText("tersedia");
    // The holder is context, not a refusal: it says what comes before this permit.
    await expect(status).toContainText("E2E Operator Jakpus");
    await expect(status).toContainText("2026-12-31");

    // Move the period onto I1's: same asset, different answer, no re-pick.
    await page.locator("#periode-mulai").fill("2026-06-01");
    await page.locator("#periode-selesai").fill("2026-06-10");
    await expect(status, "an overlapping period is still refused").toContainText(
      "sedang digunakan",
      { timeout: 20000 },
    );
    await expect(status).not.toHaveAttribute("data-status", "PEMAKAIAN_BERURUTAN");

    // And back: the permit that follows is FILED, not merely flagged.
    await page.locator("#periode-mulai").fill("2031-03-01");
    await page.locator("#periode-selesai").fill("2031-03-10");
    await expect(status).toHaveAttribute("data-status", "PEMAKAIAN_BERURUTAN", {
      timeout: 20000,
    });
    await page.locator("#keperluan").fill(`Uji e2e pemakaian berurutan ${Date.now()}`);
    await page.locator("#serial-number-0").fill("SN-E2E-0002"); // a laptop: serial is required

    const dibalas = page.waitForResponse(
      (r) => r.request().method() === "POST" && r.url().endsWith("/pemakaian-bmn"),
    );
    await page.getByRole("button", { name: "Ajukan Permohonan" }).click();
    const balasan = await dibalas;
    expect(
      balasan.status(),
      `a permit that follows the held one must be accepted: ${await balasan.text()}`,
    ).toBe(201);
  });

  test("pemakaian picker never offers an asset belonging to another satker", async ({ page }) => {
    await page.goto(`${BASE}/pengelolaan/pemakaian/buat`, { waitUntil: "domcontentloaded" });
    // E2E-B-1 sits in satker 0200020. operator_a is in 0200010.
    await page.getByLabel("Cari BMN", { exact: true }).fill("E2E-B-1");

    // Scoping happens server-side, so the row is not merely un-clickable — it
    // is never sent. Assert the empty state rather than the absence of a list:
    // "no list yet" is also what a still-running search looks like.
    await expect(page.getByTestId("aset-0-kosong"), "another satker's asset was offered").toBeVisible(
      { timeout: 20000 },
    );
    await expect(page.getByTestId("aset-0-hasil")).toHaveCount(0);
    await expect(page.getByTestId("aset-0-terpilih")).toHaveCount(0);
    // Nothing about the other satker's holder may appear.
    await expect(page.locator("body")).not.toContainText("E2E Operator Jaksel");
  });

  // The extra-BMN rows on both create forms were three (pemakaian) and four
  // (penghapusan) free-text boxes asking for kode barang / nama barang / NUP —
  // all three of which sit in integrasi.siman_aset. They now carry their own
  // picker.
  //
  // Keying those rows by position was also measurably wrong. Against the old
  // build: type AAA into row 0 and BBB into row 1 of the pemakaian form, delete
  // row 0, and the surviving row STILL SHOWS AAA while the payload carries BBB
  // (Leptos reuses the DOM node on rebuild, and those inputs had no
  // `prop:value`, so nothing overwrote the deleted row's text). The second test
  // below is the regression for that: identity must follow the row, not its
  // index.

  test("pemakaian extra BMN row resolves its own asset, independent of the main one", async ({
    page,
  }) => {
    await page.goto(`${BASE}/pengelolaan/pemakaian/buat`, { waitUntil: "domcontentloaded" });
    await pickAset(page, "E2E-A-1", "aset-0");

    await page.getByTestId("tambah-bmn").click();
    await pickAset(page, "E2E-A-2", "aset-1");

    // Each picker holds its own row. A single shared Vec would have let the
    // second pick overwrite the first.
    await expect(page.getByTestId("aset-0-terpilih")).toContainText("E2E-A-1");
    const tambahan = page.getByTestId("aset-1-terpilih");
    await expect(tambahan).toContainText("E2E-A-2");
    await expect(tambahan, "kode barang must come from SIMAN kd_brg").toContainText("3100102003");
  });

  test("removing an extra BMN row keeps the remaining row's own asset", async ({ page }) => {
    await page.goto(`${BASE}/pengelolaan/pemakaian/buat`, { waitUntil: "domcontentloaded" });

    await page.getByTestId("tambah-bmn").click();
    await page.getByTestId("tambah-bmn").click();
    // Fill only the SECOND row, then delete the first. Keying rows by position
    // would reassign identity here and hand row 1's picker the row 0 slug —
    // the pick would appear to jump to the deleted row.
    await pickAset(page, "E2E-A-2", "aset-2");

    await page.getByRole("button", { name: "Hapus BMN" }).first().click();

    await expect(page.getByTestId("aset-1-terpilih")).toHaveCount(0);
    await expect(page.getByTestId("aset-2-terpilih")).toContainText("E2E-A-2");
  });

  test("penghapusan extra item row fills its identity from the picker", async ({ page }) => {
    await page.goto(`${BASE}/pengelolaan/penghapusan/buat`, { waitUntil: "domcontentloaded" });

    await expect(page.getByText("Belum ada item tambahan.")).toBeVisible();
    await page.getByRole("button", { name: "+ Tambah Item" }).click();
    await pickAset(page, "E2E-A-2", "aset-tambahan-0");

    // Identity is read from the picked row, not transcribed — and so is the
    // value. It used to be a typeable box on the grounds that the server only
    // overrode the MAIN item's figure; that left every extra item's value in the
    // operator's hands, and the value is what places the usulan within an
    // authority. It is now shown, never entered.
    const tambahan = page.getByTestId("aset-tambahan-0-terpilih");
    await expect(tambahan).toContainText("3100102003");
    await expect(tambahan).toContainText("E2E-A-2");
    await expect(page.getByTestId("nilai-tambahan-0")).toContainText("15.000.000");
    await expect(
      page.locator("#nilai-tambahan-0"),
      "the extra item's value is no longer something the operator types",
    ).toHaveCount(0);
  });

  test("penghapusan form no longer asks anyone to type what the picked asset already says", async ({
    page,
  }) => {
    await page.goto(`${BASE}/pengelolaan/penghapusan/buat`, { waitUntil: "domcontentloaded" });
    await pickAset(page, "E2E-A-1");

    const terpilih = page.getByTestId("aset-terpilih");
    await expect(terpilih).toContainText("Station Wagon");
    await expect(terpilih).toContainText("3020101003");
    // The three boxes that used to sit under the picker, filled by it and
    // editable anyway, are gone. What is left of the value is read-only.
    for (const id of ["#kode_barang", "#nama_barang", "#nup"]) {
      await expect(page.locator(id), `${id} is a box to type into again`).toHaveCount(0);
    }
    await expect(page.locator("#nilai_perolehan")).toHaveJSProperty("readOnly", true);
    await expect(page.locator("#nilai_perolehan")).toHaveValue("250000000");
  });

  test("penghapusan form refuses to send until an asset is picked", async ({ page }) => {
    await page.goto(`${BASE}/pengelolaan/penghapusan/buat`, { waitUntil: "domcontentloaded" });
    await page.locator("#tanggal").fill("2031-03-01");
    await page.locator("#metode").selectOption("Musnah");
    await page.locator("#alasan").fill("Rusak berat dan tidak ekonomis untuk diperbaiki");
    await page.locator("#lampiran").fill("https://storage.example.com/lampiran-uji.pdf");

    let sent = false;
    page.on("request", (r) => {
      if (r.method() === "POST" && r.url().endsWith("/penghapusan-bmn")) sent = true;
    });
    await page.getByRole("button", { name: "Simpan Draft" }).click();
    await expect(page.getByText("Pilih BMN yang diusulkan terlebih dahulu").first()).toBeVisible();
    expect(sent, "a request went out with no asset named").toBe(false);
  });

  // ── The submit itself ─────────────────────────────────────────────────────
  // No spec used to press "Ajukan Permohonan". The form read the satker from a
  // JWT claim (`satker_id`) that authenc never issues, so it refused to send
  // for EVERY user ("satker_id kosong") and nothing in CI noticed: the specs
  // above stop at the asset picker, and the integration tests post hand-written
  // JSON rather than what the form builds.
  //
  // This one presses it, and asserts the two things the fix is about:
  //   * the request carries the NIP and NOTHING ELSE about the person — every
  //     field MySIMKARI already knows is the server's to fill, not the form's;
  //   * what the server stored is MySIMKARI's answer (seed-multisatker.sql row
  //     200000000000000001), read back through the API as the same operator.
  test("pemakaian form files a permit from the NIP alone, and the server fills the person in", async ({
    page,
  }) => {
    const NIP = "200000000000000001";
    const keperluan = `Uji e2e pengajuan hanya dengan NIP ${Date.now()}`;

    await page.goto(`${BASE}/pengelolaan/pemakaian/buat`, { waitUntil: "domcontentloaded" });

    // Only the NIP is typed. The card is the lookup's answer, shown so the
    // operator can see who they picked.
    await page.locator("#pegawai-nip").fill(NIP);
    const kartu = page.getByTestId("pegawai-terpilih");
    await expect(kartu, "NIP lookup never resolved the employee").toBeVisible({ timeout: 20000 });
    await expect(kartu).toContainText("E2E Operator Jakpus");

    // A far-future window: nothing else in the seed is booked then.
    await page.locator("#periode-mulai").fill("2031-03-01");
    await page.locator("#periode-selesai").fill("2031-03-10");
    await page.locator("#keperluan").fill(keperluan);

    await pickAset(page, "E2E-A-1", "aset-0");
    await expect(page.getByTestId("bmn-ketersediaan-0")).toContainText("tersedia", {
      timeout: 20000,
    });
    // E2E-A-1 is a vehicle, so the CHECK on kendaraan rows applies.
    await page.locator("#no-polisi-0").fill("B 1234 UJI");
    await page.locator("#no-bpkb-0").fill("BPKB-E2E-0001");

    const kirim = page.waitForRequest(
      (r) => r.method() === "POST" && r.url().endsWith("/pemakaian-bmn"),
    );
    const dibalas = page.waitForResponse(
      (r) => r.request().method() === "POST" && r.url().endsWith("/pemakaian-bmn"),
    );
    await page.getByRole("button", { name: "Ajukan Permohonan" }).click();

    const body = (await kirim).postDataJSON() as Record<string, unknown>;
    for (const f of [
      "pegawai_nama",
      "pegawai_satker_id",
      "pegawai_satker_nama",
      "pegawai_jabatan",
      "pegawai_golongan",
      "pegawai_pangkat",
      "pegawai_unit_kerja",
      "foto_pegawai",
    ]) {
      expect(Object.keys(body), `the form sent ${f}, which MySIMKARI already knows`).not.toContain(f);
    }
    expect(body.pegawai_nip).toBe(NIP);

    const balasan = await dibalas;
    expect(balasan.status(), `submit refused: ${await balasan.text()}`).toBe(201);

    // The permit as stored, read back by id through the API.
    const id = (await balasan.json()).data.id as string;
    const { accessToken } = await apiLogin(page.request, credsFor(userFor("operator_a")));
    const resp = await page.request.get(`${PM_API}/${id}`, {
      headers: { Authorization: `Bearer ${accessToken}` },
    });
    expect(resp.ok(), `GET ${id} (${resp.status()})`).toBeTruthy();
    const izin = (await resp.json()).data;
    expect(izin.keperluan).toBe(keperluan);
    expect(izin.pegawai_nama).toBe("E2E Operator Jakpus");
    expect(izin.pegawai_jabatan).toBe("Operator Satker");
    expect(izin.pegawai_pangkat).toBe("Penata Muda");
    expect(izin.pegawai_satker_nama).toBe("KEJAKSAAN NEGERI JAKARTA PUSAT");
    expect(izin.satker_code).toBe("0200010");
    expect(izin.status).toBe("DRAFT");
  });

  // The penghapusan form's submit had no spec either. It verified ONE asset on
  // the server — the "item utama" — and wrote the rest of the list as sent, so
  // the names and values on every extra item were whatever the form said. This
  // presses the button with two assets and asserts both halves of the fix:
  //   * the request names each asset by kode barang + NUP and carries no name
  //     and no value, anywhere;
  //   * what was stored is SIMAN's answer for BOTH assets (seed-multisatker.sql:
  //     E2E-A-1 Station Wagon Rp 250 000 000, E2E-A-2 Note Book Rp 15 000 000).
  test("penghapusan form files an usulan from the picked assets alone, and SIMAN fills in name and value", async ({
    page,
  }) => {
    const alasan = `Uji e2e usulan penghapusan dari pilihan aset ${Date.now()}`;

    await page.goto(`${BASE}/pengelolaan/penghapusan/buat`, { waitUntil: "domcontentloaded" });
    await pickAset(page, "E2E-A-1");
    await page.getByRole("button", { name: "+ Tambah Item" }).click();
    await pickAset(page, "E2E-A-2", "aset-tambahan-0");
    await page.locator("#tanggal").fill("2031-03-01");
    await page.locator("#metode").selectOption("Musnah");
    await page.locator("#alasan").fill(alasan);
    await page.locator("#lampiran").fill("https://storage.example.com/lampiran-uji.pdf");

    const kirim = page.waitForRequest(
      (r) => r.method() === "POST" && r.url().endsWith("/penghapusan-bmn"),
    );
    const dibalas = page.waitForResponse(
      (r) => r.request().method() === "POST" && r.url().endsWith("/penghapusan-bmn"),
    );
    await page.getByRole("button", { name: "Simpan Draft" }).click();

    const body = (await kirim).postDataJSON() as {
      items: Array<Record<string, unknown>>;
    } & Record<string, unknown>;
    for (const f of ["nama_barang", "nilai_perolehan"]) {
      expect(Object.keys(body), `the form sent ${f}, which SIMAN already knows`).not.toContain(f);
      for (const [i, it] of body.items.entries()) {
        expect(Object.keys(it), `item ${i + 1} carries ${f}`).not.toContain(f);
      }
    }
    expect(body.items.map((i) => i.nup)).toEqual(["E2E-A-1", "E2E-A-2"]);
    expect(body.nup).toBe("E2E-A-1");

    const balasan = await dibalas;
    expect(balasan.status(), `submit refused: ${await balasan.text()}`).toBe(201);

    const id = (await balasan.json()).data.id as string;
    const { accessToken } = await apiLogin(page.request, credsFor(userFor("operator_a")));
    // `/detail`, not the bare GET: only the detail response lists the items.
    const resp = await page.request.get(`${PH_API}/${id}/detail`, {
      headers: { Authorization: `Bearer ${accessToken}` },
    });
    expect(resp.ok(), `GET ${id}/detail (${resp.status()})`).toBeTruthy();
    const usulan = (await resp.json()).data;
    expect(usulan.alasan).toBe(alasan);
    expect(usulan.nama_barang).toBe("Station Wagon");
    expect(Number(usulan.nilai_perolehan)).toBe(250000000);
    const items = usulan.items as Array<{ nup: string; nama_barang: string; nilai_perolehan: number }>;
    expect(items.map((i) => [i.nup, i.nama_barang, Number(i.nilai_perolehan)])).toEqual([
      ["E2E-A-1", "Station Wagon", 250000000],
      ["E2E-A-2", "Note Book", 15000000],
    ]);

    // The same assets on the screen a validator approves from. The detail page
    // used to print the first one only, so the rest of the list was on no screen.
    await page.goto(`${BASE}/pengelolaan/penghapusan/detail/${id}`, {
      waitUntil: "domcontentloaded",
    });
    const tabel = page.getByTestId("tabel-bmn-usulan");
    await expect(tabel).toBeVisible({ timeout: 20000 });
    await expect(tabel.locator("tbody tr")).toHaveCount(2);
    await expect(tabel).toContainText("Station Wagon");
    await expect(tabel).toContainText("Note Book");
    await expect(page.getByTestId("total-nilai-usulan")).toContainText("265.000.000");
  });
});

// ── Pemakaian: satker-internal approval chain, driven through the UI (#96) ──
// The chain was unusable twice over before this: the FE had no buttons, and
// the two roles the policy requires did not exist in the IAM seed. Both halves
// are exercised here — if either regressed, the buttons would not render.

/** Backend source-of-truth for a permit's status, read as validator_pusat. */
async function pmStatus(request: APIRequestContext, id: string): Promise<string> {
  const { accessToken } = await apiLogin(request, credsFor(userFor("validator_pusat")));
  const resp = await request.get(`${PM_API}/${id}`, {
    headers: { Authorization: `Bearer ${accessToken}` },
  });
  expect(resp.ok(), `GET pemakaian-bmn/${id} (${resp.status()})`).toBeTruthy();
  const body = await resp.json();
  return body.data.status;
}

async function openPermit(page: import("@playwright/test").Page, id: string) {
  await page.goto(`${BASE}/pengelolaan/pemakaian/detail/${id}`, {
    waitUntil: "domcontentloaded",
  });
  await expect(shell(page), "app shell mounts").toBeVisible({ timeout: 20000 });
}

test.describe("Pemakaian BMN — validator satker forwards to approver", () => {
  test.use({ storageState: storageStatePath("validator_satker") });

  test("validator_satker forwards I2 (SUBMITTED → SUBMITTED_APPROVER_SATKER)", async ({ page, request }) => {
    expect(await pmStatus(request, I2_IZIN)).toBe("SUBMITTED");

    await openPermit(page, I2_IZIN);
    await expect(page.getByText("E2E Laptop Ajuan Satker").first()).toBeVisible({
      timeout: 20000,
    });

    await clickAction(
      page,
      page.getByRole("button", { name: "Teruskan ke Approver Satker" }),
      "validator-satker-action",
    );

    // The action is gone once the permit has moved on.
    await page.reload({ waitUntil: "domcontentloaded" });
    await expect(shell(page)).toBeVisible({ timeout: 20000 });
    await expect(page.getByRole("button", { name: "Teruskan ke Approver Satker" })).toHaveCount(0);

    await expect.poll(() => pmStatus(request, I2_IZIN), { timeout: 15000 }).toBe("SUBMITTED_APPROVER_SATKER");
  });
});

test.describe("Pemakaian BMN — operator is offered no approval action", () => {
  test.use({ storageState: storageStatePath("operator_a") });

  // The negative half of the same mechanism: allowed_transitions is filtered by
  // role server-side, so the operator who RAISED the permit sees no approval
  // button on it. Rendering the unfiltered state machine would show one here.
  test("operator_a sees neither forward nor approve on I2", async ({ page }) => {
    await openPermit(page, I2_IZIN);
    await expect(page.getByText("E2E Laptop Ajuan Satker").first()).toBeVisible({
      timeout: 20000,
    });
    await expect(page.getByRole("button", { name: "Teruskan ke Approver Satker" })).toHaveCount(0);
    await expect(page.getByRole("button", { name: "Setujui Izin" })).toHaveCount(0);
  });
});

test.describe("Pemakaian BMN — approver satker approves", () => {
  test.use({ storageState: storageStatePath("approver_satker") });

  // Runs after the forward test above (same file, serial by default within a
  // worker); it asserts the precondition rather than assuming it.
  test("approver_satker approves I2 (→ ACTIVE, auto-activated)", async ({ page, request }) => {
    await expect.poll(() => pmStatus(request, I2_IZIN), { timeout: 30000 }).toBe("SUBMITTED_APPROVER_SATKER");

    await openPermit(page, I2_IZIN);
    await clickAction(
      page,
      page.getByRole("button", { name: "Setujui Izin" }),
      "approver-satker-action",
    );

    await page.reload({ waitUntil: "domcontentloaded" });
    await expect(shell(page)).toBeVisible({ timeout: 20000 });
    await expect(page.getByRole("button", { name: "Setujui Izin" })).toHaveCount(0);

    // APPROVED is a TRANSIENT state here, not the outcome. `approver_satker_approve`
    // commits APPROVED and then immediately calls `activate_permit`, which assigns
    // nomor_izin and commits ACTIVE — so the settled status of a successful approval
    // is ACTIVE (matching the I1 assertion above), and APPROVED means activation
    // never completed.
    //
    // What used to make this flip run to run was NOT the poll sampling a moment
    // between the two commits — it was the test cancelling its own request. The
    // click was followed straight by `page.reload()`, which tore down the in-flight
    // POST; nginx logged it `499` and the server dropped the handler mid-chain, so
    // whether the activation got to run came down to CI host load. `clickAction`
    // waits for the response, and the service is now cancel-safe besides.
    await expect.poll(() => pmStatus(request, I2_IZIN), { timeout: 15000 }).toBe("ACTIVE");
  });
});
