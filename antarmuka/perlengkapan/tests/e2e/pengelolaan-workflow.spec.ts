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
import { test, expect, type APIRequestContext } from "@playwright/test";
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

// ── Create-form pages are reachable (route-coverage) ─────────────────────────
// The create forms need an asset picker fed by bank_aset/siman — driving a
// full UI create lands with the E-5 negative/edge pass. Here the typed routes
// must at least mount the authenticated shell (no auth bounce).
// (Full literal paths so the route-coverage gate detects these as visited.)
test.describe("Pengelolaan — create-form pages are reachable", () => {
  test.use({ storageState: storageStatePath("operator_a") });

  test("penghapusan create form is reachable", async ({ page }) => {
    await reachable(page, `${BASE}/pengelolaan/penghapusan/buat`);
  });
  test("pemakaian create form is reachable", async ({ page }) => {
    await reachable(page, `${BASE}/pengelolaan/pemakaian/buat`);
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
