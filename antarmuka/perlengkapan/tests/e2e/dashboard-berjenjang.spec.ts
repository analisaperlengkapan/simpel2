/**
 * The dashboard drill-down follows the reader's scope, level by level:
 *
 *   • operator_satker   — one satker, no region or satker selector, a line naming it;
 *   • validator_wilayah — the whole wilayah by default, a satker selector under it;
 *   • validator_pusat   — the whole country by default; a wilayah narrows the satker list AT ONCE,
 *                         and a satker within a wilayah narrows the figures to that satker alone.
 *
 * Expectations are DERIVED from the environment under test, not enumerated: the satker list a
 * wilayah must offer is read from `/bank-aset/filter-options` (each satker carries its `induk`,
 * the wilayah it sits in) and the figures from `/bank-aset/dashboard` with the same filters. The UI
 * must print what the API counts. Locators are structural (`#f-wilayah`, `#f-satker`, test ids),
 * never substring text matches.
 */
import { test, expect, type APIRequestContext, type Page } from "@playwright/test";
import { PERLENGKAPAN_API_URL, storageStatePath, tokenFor, formatThousands } from "./helpers/real-auth";

const DASBOR = "/perlengkapan/simpel/dashboard";
const API = `${PERLENGKAPAN_API_URL}/api/v1/perlengkapan`;

interface Opt {
  value: string;
  count: number;
  label?: string;
  induk?: string;
}

async function opsi(request: APIRequestContext, token: string): Promise<{ wilayah: Opt[]; satker_kode: Opt[] }> {
  const r = await request.get(`${API}/bank-aset/filter-options?facet=wilayah,satker_kode`, {
    headers: { Authorization: `Bearer ${token}` },
  });
  expect(r.ok(), `filter-options: ${r.status()}`).toBeTruthy();
  const b = await r.json();
  return b.data ?? b;
}

async function totalAset(request: APIRequestContext, token: string, query: string): Promise<number> {
  const r = await request.get(`${API}/bank-aset/dashboard${query}`, { headers: { Authorization: `Bearer ${token}` } });
  expect(r.ok(), `dashboard${query}: ${r.status()}`).toBeTruthy();
  const b = await r.json();
  return (b.data ?? b).total_aset as number;
}

const ringkasan = (page: Page) => page.getByTestId("filter-ringkasan");

/** Values offered by a select, minus the "Seluruh …" reset. Pure reader: safe inside expect.poll. */
async function offered(page: Page, id: string): Promise<string[]> {
  return (await page.locator(`#${id} option`).evaluateAll((os) => os.map((o) => (o as HTMLOptionElement).value)))
    .filter((v) => v !== "")
    .sort();
}

async function selected(page: Page, id: string): Promise<string> {
  return page.locator(`#${id}`).evaluate((e) => (e as HTMLSelectElement).value);
}

test.describe("dashboard drill-down — operator_satker", () => {
  test.use({ storageState: storageStatePath("operator_a") });

  test("sees one satker: no region or satker selector, a line naming it, and still jenis/tahun", async ({
    page,
    request,
  }) => {
    const token = await tokenFor(request, "operator_a");
    const o = await opsi(request, token);
    expect(o.satker_kode, "an operator has exactly one satker in scope").toHaveLength(1);
    const nama = (o.satker_kode[0].label ?? o.satker_kode[0].value).trim();

    await page.goto(DASBOR, { waitUntil: "domcontentloaded" });
    await expect(ringkasan(page)).toContainText(nama, { timeout: 30000 });
    await expect(page.locator("#f-wilayah"), "an operator cannot widen to a region").toHaveCount(0);
    await expect(page.locator("#f-satker"), "an operator has no satker to choose").toHaveCount(0);
    await expect(page.locator("#f-jenis"), "jenis still means something to an operator").toHaveCount(1);
    await expect(page.locator("#f-tahun")).toHaveCount(1);
  });
});

test.describe("dashboard drill-down — validator_wilayah", () => {
  test.use({ storageState: storageStatePath("validator_wilayah") });

  test("whole wilayah by default, one satker under it on request", async ({ page, request }) => {
    const token = await tokenFor(request, "validator_wilayah");
    const o = await opsi(request, token);
    expect(o.satker_kode.length, "a wilayah validator oversees at least one satker").toBeGreaterThan(0);
    const wilayahTotal = await totalAset(request, token, "");

    await page.goto(DASBOR, { waitUntil: "domcontentloaded" });
    await expect(page.locator("#f-wilayah"), "the wilayah is fixed by the claims").toHaveCount(0);
    await expect(ringkasan(page)).toContainText("seluruh satuan kerjanya", { timeout: 30000 });
    await expect
      .poll(() => offered(page, "f-satker"), { timeout: 30000, message: "the satker list is exactly the wilayah's" })
      .toEqual(o.satker_kode.map((s) => s.value).sort());
    await expect(page.getByText(formatThousands(wilayahTotal), { exact: true }).first()).toBeVisible();

    // Narrow to one satker: the figures are that satker's, and the choice stays on screen.
    const satker = o.satker_kode[0];
    const satkerTotal = await totalAset(request, token, `?satker_kode=${encodeURIComponent(satker.value)}`);
    await page.locator("#f-satker").selectOption(satker.value);
    await expect(ringkasan(page)).toContainText("›");
    await expect(page.getByText(formatThousands(satkerTotal), { exact: true }).first()).toBeVisible({ timeout: 20000 });
    expect(await selected(page, "f-satker"), "what is shown is what is filtered").toBe(satker.value);

    // Clearing returns to the whole wilayah, on screen as well as in the figures.
    await page.getByTestId("filter-bersihkan").click();
    await expect(ringkasan(page)).toContainText("seluruh satuan kerjanya");
    expect(await selected(page, "f-satker")).toBe("");
    await expect(page.getByText(formatThousands(wilayahTotal), { exact: true }).first()).toBeVisible({ timeout: 20000 });
  });
});

test.describe("dashboard drill-down — validator_pusat", () => {
  test.use({ storageState: storageStatePath("validator_pusat") });

  test("a wilayah narrows the satker list at once; a satker within it narrows the figures to that satker", async ({
    page,
    request,
  }) => {
    const token = await tokenFor(request, "validator_pusat");
    const o = await opsi(request, token);
    // The wilayah holding the most satker: the narrowing is then visible, not vacuous.
    const perWilayah = new Map<string, string[]>();
    for (const s of o.satker_kode) {
      if (s.induk) perWilayah.set(s.induk, [...(perWilayah.get(s.induk) ?? []), s.value]);
    }
    expect(perWilayah.size, "satker options must carry the wilayah they sit in (induk)").toBeGreaterThan(0);
    const [wilayah, anggota] = [...perWilayah.entries()].sort((a, b) => b[1].length - a[1].length)[0];
    const lain = [...perWilayah.keys()].find((w) => w !== wilayah);

    await page.goto(DASBOR, { waitUntil: "domcontentloaded" });
    await expect
      .poll(() => offered(page, "f-satker"), { timeout: 30000, message: "by default every satker is offered" })
      .toEqual(o.satker_kode.map((s) => s.value).sort());
    const semuaTotal = await totalAset(request, token, "");
    await expect(page.getByText(formatThousands(semuaTotal), { exact: true }).first()).toBeVisible();

    // Choose a wilayah: the satker list is narrowed to exactly its satker — without waiting for
    // the server (no options request is made for it).
    let optionRequests = 0;
    page.on("request", (r) => {
      if (r.url().includes("/bank-aset/filter-options")) optionRequests += 1;
    });
    await page.locator("#f-wilayah").selectOption(wilayah);
    await expect
      .poll(() => offered(page, "f-satker"), { timeout: 2000, message: "the satker list follows the wilayah at once" })
      .toEqual([...anggota].sort());
    expect(await selected(page, "f-wilayah"), "the chosen wilayah stays on screen").toBe(wilayah);
    const wilayahTotal = await totalAset(request, token, `?wilayah=${encodeURIComponent(wilayah)}`);
    await expect(page.getByText(formatThousands(wilayahTotal), { exact: true }).first()).toBeVisible({ timeout: 20000 });

    // A satker within the wilayah: both filters are sent and the figures are that satker's.
    const satker = anggota[0];
    const satkerTotal = await totalAset(
      request,
      token,
      `?wilayah=${encodeURIComponent(wilayah)}&satker_kode=${encodeURIComponent(satker)}`,
    );
    await page.locator("#f-satker").selectOption(satker);
    await expect(ringkasan(page)).toContainText("›");
    await expect(page.getByText(formatThousands(satkerTotal), { exact: true }).first()).toBeVisible({ timeout: 20000 });
    expect(await selected(page, "f-satker")).toBe(satker);

    // Switching wilayah drops the satker (it is not in the new one) and offers the new wilayah's.
    if (lain) {
      await page.locator("#f-wilayah").selectOption(lain);
      await expect
        .poll(() => offered(page, "f-satker"), { timeout: 2000 })
        .toEqual([...(perWilayah.get(lain) ?? [])].sort());
      expect(await selected(page, "f-satker"), "a satker of the old wilayah is not carried over").toBe("");
      expect(await selected(page, "f-wilayah")).toBe(lain);
    }

    // Choosing wilayah and satker never asked the server for the options again.
    expect(optionRequests, "the lists are loaded once, not per choice").toBe(0);

    // Clearing returns to the whole country, on screen as well as in the figures.
    await page.getByTestId("filter-bersihkan").click();
    expect(await selected(page, "f-wilayah")).toBe("");
    expect(await selected(page, "f-satker")).toBe("");
    await expect(ringkasan(page)).toContainText("Seluruh Indonesia");
    await expect(page.getByText(formatThousands(semuaTotal), { exact: true }).first()).toBeVisible({ timeout: 20000 });
  });
});
