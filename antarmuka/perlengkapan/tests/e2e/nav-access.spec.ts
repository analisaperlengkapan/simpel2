/**
 * Route-coverage / navigation reachability (#33 / F5-C). Replaces the deleted
 * legacy MOCK ui-*.spec.ts (dead `helpers/session.ts` token + stale pre-v2
 * routes) with a real-auth pass: every main feature-module route must mount the
 * authenticated app shell for an authorized user — catching unregistered routes,
 * WASM crashes, and auth/guard misconfig.
 *
 * Signal: `app_chrome.rs` renders a sticky <header> containing "SIMPEL" on every
 * authenticated route (the AuthenticatedLayout wrapper), so it's a stable,
 * data-independent reachability marker — no FE→BE call required. Deep per-feature
 * WORKFLOWS (create→submit→approve state machines, SK generation, exports) are the
 * @staging comprehensive pass (F5-C Layer-3 / F5-E), authored against live data.
 *
 * Reuses the per-role storageState from auth.setup.ts.
 */
import { test, expect } from '@playwright/test';
import { storageStatePath } from './helpers/real-auth';

const BASE = '/perlengkapan/simpel/v2';

// Feature modules every authenticated user can open (role-gating, where it
// applies, renders inside the shell — it does not redirect away).
const CORE_ROUTES = [
  `${BASE}/dashboard`,
  `${BASE}/bank-aset/dashboard`,
  `${BASE}/bank-aset/daftar`,
  `${BASE}/kebutuhan-bmn/daftar`,
  `${BASE}/pakaian-dinas/pengajuan`,
  `${BASE}/pengelolaan/pemakaian`,
  `${BASE}/pengelolaan/penghapusan`,
];

// Cross-satker / admin-leaning modules (assert reachability with the admin user).
const ADMIN_ROUTES = [
  `${BASE}/analitik/roadmap`,
  `${BASE}/notifikasi`,
  `${BASE}/admin/roles`,
  `${BASE}/admin/audit`,
  `${BASE}/admin/master`,
  `${BASE}/admin/templates`,
];

const LOGIN = /login/i;
const FORBIDDEN = /Akses Ditolak/i;
const shell = (page: import('@playwright/test').Page) =>
  page.locator('header').getByText('SIMPEL').first();

test.describe('Route coverage — admin user reaches every module', () => {
  test.use({ storageState: storageStatePath('admin') });

  for (const route of [...CORE_ROUTES, ...ADMIN_ROUTES]) {
    test(`shell mounts at ${route}`, async ({ page }) => {
      await page.goto(route, { waitUntil: 'domcontentloaded' });
      await expect(shell(page), `app shell should mount on ${route}`).toBeVisible({
        timeout: 20000,
      });
      expect(LOGIN.test(page.url()), `must not redirect to login (${page.url()})`).toBeFalsy();
      await expect(
        page.getByText(FORBIDDEN),
        `admin must not be forbidden on ${route}`,
      ).toHaveCount(0);
    });
  }
});

test.describe('Route coverage — operator reaches the core modules', () => {
  test.use({ storageState: storageStatePath('operator_a') });

  for (const route of CORE_ROUTES) {
    test(`shell mounts at ${route}`, async ({ page }) => {
      await page.goto(route, { waitUntil: 'domcontentloaded' });
      await expect(shell(page), `app shell should mount on ${route}`).toBeVisible({
        timeout: 20000,
      });
      // authZ (role-gating) renders inside the shell; it must NOT bounce an
      // authenticated operator back to login.
      expect(LOGIN.test(page.url()), `must not redirect to login (${page.url()})`).toBeFalsy();
    });
  }
});
