/**
 * Per-role RoleGate regression (#33 / F5-C; extends the #482 guard-dedup runtime
 * check beyond the single-route AuthGate smoke in guards-smoke.spec.ts).
 *
 * Perlengkapan's `/admin/*` routes are wrapped by `AdminLayout`, which renders
 * the shared ForbiddenPage when the authenticated session is not admin
 * (`UserSession::is_admin()` — a CLIENT-SIDE JWT-claim check, so this works on
 * the compose CI stack without the FE→BE data path). This asserts the gate per
 * role:
 *   - the four single-role seed users (operator_satker / validator_wilayah /
 *     validator_pusat — none of which is an admin role) are DENIED, see the
 *     forbidden page, and are NOT redirected to login (authZ, not authN);
 *   - the all-role seed user (admin-capable) reaches the admin content.
 *
 * Reuses the per-role storageState written by auth.setup.ts (real authenc JWTs).
 */
import { test, expect } from '@playwright/test';
import { TEST_USERS, storageStatePath } from './helpers/real-auth';

const ADMIN_ROUTE = '/perlengkapan/simpel/v2/admin/users';
// ForbiddenPage (route guard) and the page-level guard both render this.
const FORBIDDEN = /Akses Ditolak/i;
// AdminUsersPage renders this heading only once the admin gate passes.
const ADMIN_HEADING = /Manajemen Pengguna/i;
const LOGIN_HINT = /login/i;

// Non-admin: admin route → forbidden, no admin content, no login redirect.
for (const user of TEST_USERS) {
  test.describe(`admin route denied for non-admin — ${user.key} (${user.role})`, () => {
    test.use({ storageState: storageStatePath(user.key) });

    test(`${user.key} is forbidden at /admin/users`, async ({ page }) => {
      await page.goto(ADMIN_ROUTE, { waitUntil: 'domcontentloaded' });
      await expect(
        page.getByText(FORBIDDEN).first(),
        `${user.key} should see the forbidden page on the admin route`,
      ).toBeVisible({ timeout: 15000 });
      expect(
        LOGIN_HINT.test(page.url()),
        `authZ denial must not redirect to login (got ${page.url()})`,
      ).toBeFalsy();
      await expect(
        page.getByText(ADMIN_HEADING),
        `${user.key} must NOT see admin content`,
      ).toHaveCount(0);
    });
  });
}

// Admin (all-role seed user): the gate passes and admin content renders.
test.describe('admin route allowed for the admin user', () => {
  test.use({ storageState: storageStatePath('admin') });

  test('admin reaches /admin/users content', async ({ page }) => {
    await page.goto(ADMIN_ROUTE, { waitUntil: 'domcontentloaded' });
    await expect(
      page.getByText(ADMIN_HEADING).first(),
      'admin should reach the user-management page (gate passes)',
    ).toBeVisible({ timeout: 15000 });
    await expect(
      page.getByText(FORBIDDEN),
      'admin must NOT see the forbidden page',
    ).toHaveCount(0);
  });
});
