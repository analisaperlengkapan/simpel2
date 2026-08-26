/**
 * UI-layer RBAC data-scoping (#33 / F5-C, Layer-3). Complements the API-layer
 * proof (rbac-scoping.spec.ts) by verifying the FE actually *renders* only
 * in-scope data: the WASM bank-aset list calls the backend origin-relative
 * (`api/client.rs` API_BASE=/api/v1/perlengkapan), which the perlengkapan nginx
 * proxies to layanan-perlengkapan.
 *
 * ── Why this spec no longer names any asset ──────────────────────────────────
 * It used to assert presence/absence of six hard-coded seed NUP markers
 * (`E2E-A-1` …), justified by the comment "5 seed rows < the default page size".
 * That is a property of a FRESH, EMPTY CI database. Against the staging SIMAN
 * snapshot the page shows PER_PAGE=25 rows out of 1 681 (operator_a) or 624 533
 * (validator_pusat), ordered by `updated_at DESC`, and nothing guarantees a
 * seeded row lands on page 1 at all — so the "visible" assertions failed while
 * the "hidden" ones passed VACUOUSLY (absent because off-page, not because
 * scoped out). A leak test that passes by rendering nothing is the failure mode
 * catalogued in `project_gate_scope_must_be_derived`.
 *
 * It also used `getByText(nup, { exact: false })`, which is substring +
 * case-insensitive: `E2E-A-1` matches a cell reading `E2E-A-12`
 * (`project_playwright_gettext_false_pass`). Locators here are structural
 * (column position in the table) instead.
 *
 * So expectations are DERIVED from the environment under test:
 *   • the scoped filter-options endpoint gives the EXACT set of satkers a user
 *     may see (page-size independent — it is a GROUP BY over the whole scope,
 *     not one page), and
 *   • `deriveScope` gives the exact row total the backend counts.
 * Both are then asserted against what the browser renders.
 */
import { test, expect, type Page } from '@playwright/test';
import {
  TEST_USERS,
  storageStatePath,
  tokenFor,
  deriveScope,
  allowedSatkerNames,
  formatThousands,
} from './helpers/real-auth';

const DAFTAR = '/perlengkapan/simpel/v2/bank-aset/daftar';

/** Table body rows of the asset list (structural, not text-matched). */
function rowsOf(page: Page) {
  return page.locator('table tbody tr');
}

/**
 * The "Satker" column is the 5th `<td>` (Aset, Kode, NUP, Kondisi, Satker,
 * Nilai, actions — see `pages/bank_aset/list_page.rs`). Addressing it by
 * position keeps the assertion about the SATKER cell specifically, so a value
 * appearing in some other column cannot satisfy it.
 */
function satkerCells(page: Page) {
  return rowsOf(page).locator('td:nth-child(5)');
}

/**
 * Satker names offered by the filter dropdown, minus the "Semua Satker" reset.
 *
 * The page has FIVE selects (jenis / kategori / kondisi / sort / satker), so a
 * bare `select option` locator would pool every dropdown's options together and
 * the "offers exactly the in-scope satkers" assertion could never hold. The
 * satker select is identified by the reset option only it contains.
 */
async function dropdownSatkerNames(page: Page): Promise<string[]> {
  const select = page.locator('select:has(option:text-is("Semua Satker"))');
  await expect(select, 'satker filter dropdown should exist').toHaveCount(1);
  const labels = await select.locator('option').allTextContents();
  return labels
    .map((l) => l.trim())
    .filter((l) => l && l !== 'Semua Satker')
    // Options render as `{nama_satker} ({count})` — strip the trailing count.
    .map((l) => l.replace(/\s*\(\d+\)\s*$/, ''))
    .sort();
}

for (const user of TEST_USERS) {
  test.describe(`bank-aset daftar renders scoped data — ${user.key} (${user.role})`, () => {
    test.use({ storageState: storageStatePath(user.key) });

    test(`${user.key} renders only in-scope satkers and totals`, async ({ page, request }) => {
      const token = await tokenFor(request, user.key);
      const allowed = await allowedSatkerNames(request, token);
      const scope = await deriveScope(request, token);

      expect(allowed.length, `${user.key} should have at least one satker in scope`).toBeGreaterThan(0);
      expect(scope.total, `${user.key} should see at least one asset`).toBeGreaterThan(0);

      await page.goto(DAFTAR, { waitUntil: 'domcontentloaded' });

      // Wait for the first row: proves the WASM booted AND the proxied call
      // returned data. Everything below is meaningless until data has rendered,
      // which is precisely what made the old absence-assertions vacuous.
      await expect(rowsOf(page).first(), `${user.key} should see rendered asset rows`).toBeVisible({
        timeout: 20000,
      });

      // (1) The total the UI prints must equal the total the backend counts for
      // this token. Exact and environment-independent — and it is the assertion
      // that would have caught the dashboard reading a column nothing writes.
      await expect(
        page.getByText(`Total: ${formatThousands(scope.total)} aset`),
        `${user.key} UI total should equal the API total (${scope.total})`,
      ).toBeVisible();

      // (2) Every satker actually rendered must be one this user may see.
      const rendered = [...new Set((await satkerCells(page).allTextContents()).map((s) => s.trim()))]
        .filter((s) => s && s !== '-')
        .sort();
      expect(rendered.length, `${user.key} rendered rows should carry a satker`).toBeGreaterThan(0);
      for (const name of rendered) {
        expect(allowed, `${user.key} (${user.role}) rendered out-of-scope satker "${name}"`).toContain(name);
      }

      // (3) The filter dropdown must offer EXACTLY the in-scope satkers —
      // page-size independent, so this covers the whole scope and not just
      // page 1. An extra option here is a data leak even if no row shows it.
      expect(
        await dropdownSatkerNames(page),
        `${user.key} satker dropdown must offer exactly its in-scope satkers`,
      ).toEqual(allowed);

      // (4) A satker-tier user sees exactly ONE satker — the strongest form of
      // the isolation claim, and true in any environment.
      if (user.tier === 'satker') {
        expect(allowed, `${user.key} is satker-tier so exactly one satker must be in scope`).toHaveLength(1);
        expect(rendered, `${user.key} must render rows from a single satker`).toEqual(allowed);
      }
    });
  });
}

/**
 * Cross-role isolation at the UI layer, derived rather than enumerated: take a
 * satker that genuinely belongs to operator_b and prove it never reaches
 * operator_a's screen — neither as a row nor as a selectable filter option.
 *
 * The foreign satker is MEASURED (operator_b's own scope, minus anything
 * operator_a may legitimately see), so this stays correct as the underlying
 * SIMAN data changes. The previous version hard-coded three synthetic codes and
 * consequently flagged JakPus's own real assets as a leak.
 */
test.describe('bank-aset daftar does not leak across satkers', () => {
  test.use({ storageState: storageStatePath('operator_a') });

  test('operator_a never sees a satker that belongs to operator_b', async ({ page, request }) => {
    const tokenA = await tokenFor(request, 'operator_a');
    const tokenB = await tokenFor(request, 'operator_b');
    const allowedA = await allowedSatkerNames(request, tokenA);
    const allowedB = await allowedSatkerNames(request, tokenB);

    const foreign = allowedB.filter((s) => !allowedA.includes(s));
    expect(
      foreign.length,
      'operator_a and operator_b must not have identical scopes, or this test proves nothing',
    ).toBeGreaterThan(0);

    await page.goto(DAFTAR, { waitUntil: 'domcontentloaded' });
    await expect(rowsOf(page).first()).toBeVisible({ timeout: 20000 });

    const renderedSatkers = (await satkerCells(page).allTextContents()).map((s) => s.trim());
    const offered = await dropdownSatkerNames(page);
    for (const name of foreign) {
      expect(renderedSatkers, `operator_a rendered operator_b's satker "${name}"`).not.toContain(name);
      expect(offered, `operator_a was offered operator_b's satker "${name}" as a filter`).not.toContain(name);
    }
  });
});
