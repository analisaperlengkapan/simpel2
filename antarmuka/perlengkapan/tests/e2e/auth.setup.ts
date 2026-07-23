/**
 * Playwright "setup" project: authenticate the seeded users against the REAL
 * authenc REST API and persist each resulting JWT as storageState, so dependent
 * specs start already-authenticated with a backend-valid session.
 *
 *   - `perlengkapan.json` + `admin.json` — the base seed user
 *     (NIP 199203142014031001, has ALL roles incl. admin). The smoke project
 *     uses `perlengkapan.json`; the guard suite uses `admin.json` for the
 *     admin-passes case.
 *   - `<key>.json` per single-role user in TEST_USERS (operator_a/operator_b/
 *     validator_wilayah/validator_pusat/validator_satker/approver_satker) —
 *     used by the per-role guard + UI scoping + satker approval-chain suites.
 *
 * Requires the e2e stack up (docker-compose.e2e.yml: authenc REST on :18088) and
 * the multi-satker fixture loaded (the per-role users live in that seed).
 */
import { test as setup } from "@playwright/test";
import { seedRealAuth, credsFor, storageStatePath, TEST_USERS } from "./helpers/real-auth";

const DEFAULT_STATE = "results/.auth/perlengkapan.json";

setup("authenticate the all-role seed user", async ({ browser }) => {
  const context = await browser.newContext();
  await seedRealAuth(context); // API login → inject real JWT into localStorage
  // Smoke project reads DEFAULT_STATE; the guard suite reads `admin.json`.
  await context.storageState({ path: DEFAULT_STATE });
  await context.storageState({ path: storageStatePath("admin") });
  await context.close();
});

for (const user of TEST_USERS) {
  setup(`authenticate per-role user — ${user.key}`, async ({ browser }) => {
    const context = await browser.newContext();
    await seedRealAuth(context, credsFor(user));
    await context.storageState({ path: storageStatePath(user.key) });
    await context.close();
  });
}
