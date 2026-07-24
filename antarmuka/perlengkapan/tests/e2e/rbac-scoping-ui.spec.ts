/**
 * UI-layer RBAC data-scoping (#33 / F5-C, Layer-3). Complements the API-layer
 * proof (rbac-scoping.spec.ts) by verifying the FE actually *renders* only the
 * in-scope rows: the WASM bank-aset list calls the backend origin-relative
 * (`api/client.rs` API_BASE=/api/v1/perlengkapan), which the perlengkapan nginx
 * proxies to layanan-perlengkapan — so this needs the compose FE→BE upstream fix
 * (the entrypoint upstream-port override) to resolve in CI.
 *
 * Asserts presence/absence of the deterministic seed NUP markers per role
 * (tests/fixtures/e2e/seed-multisatker.sql), reusing the per-role storageState
 * from auth.setup.ts. NUP is a unique, stable per-asset marker rendered in the
 * list table (list_page.rs renders {nup}), so this is robust to layout/pagination
 * (5 seed rows < the default page size).
 */
import { test, expect } from '@playwright/test';
import { TEST_USERS, storageStatePath } from './helpers/real-auth';

const DAFTAR = '/perlengkapan/simpel/v2/bank-aset/daftar';

// Seed NUP markers per satker.
const NUP_A = ['E2E-A-1', 'E2E-A-2']; // 0200010 Jakpus (DKI)
const NUP_B = ['E2E-B-1', 'E2E-B-2']; // 0200020 Jaksel (DKI)
const NUP_C = ['E2E-C-1']; //            0300010 Bandung (JABAR)

const EXPECT: Record<string, { visible: string[]; hidden: string[] }> = {
  operator_a: { visible: NUP_A, hidden: [...NUP_B, ...NUP_C] },
  operator_b: { visible: NUP_B, hidden: [...NUP_A, ...NUP_C] },
  validator_wilayah: { visible: [...NUP_A, ...NUP_B], hidden: NUP_C },
  validator_pusat: { visible: [...NUP_A, ...NUP_B, ...NUP_C], hidden: [] },
  // Satker-internal approval chain (#96): both sit in 0200010 (Jakpus) alongside
  // operator_a. Neither is a cross-satker role (Claims::is_cross_satker_role), so
  // AsetScope::from_claims puts them in AsetScope::Satker — same 2 in-scope rows
  // as operator_a, nothing from other satkers.
  validator_satker: { visible: NUP_A, hidden: [...NUP_B, ...NUP_C] },
  approver_satker: { visible: NUP_A, hidden: [...NUP_B, ...NUP_C] },
};

for (const user of TEST_USERS) {
  test.describe(`bank-aset daftar renders scoped rows — ${user.key} (${user.role})`, () => {
    test.use({ storageState: storageStatePath(user.key) });

    test(`${user.key} sees only its in-scope assets`, async ({ page }) => {
      const exp = EXPECT[user.key];
      await page.goto(DAFTAR, { waitUntil: 'domcontentloaded' });

      // The list auto-fetches on mount; wait for the first in-scope NUP to render
      // (proves the WASM booted AND the FE→BE proxied call returned scoped data).
      await expect(
        page.getByText(exp.visible[0], { exact: false }).first(),
        `${user.key} should see ${exp.visible[0]} on the daftar`,
      ).toBeVisible({ timeout: 20000 });

      for (const nup of exp.visible) {
        await expect(
          page.getByText(nup, { exact: false }).first(),
          `${user.key} should see ${nup}`,
        ).toBeVisible();
      }
      // Absence is meaningful here: the data has already rendered above.
      for (const nup of exp.hidden) {
        await expect(
          page.getByText(nup, { exact: false }),
          `${user.key} (${user.role}) must NOT see ${nup} (cross-satker leakage)`,
        ).toHaveCount(0);
      }
    });
  });
}
