/**
 * Guards regression smoke (F5-A) — the runtime check for the #482 guard dedup
 * (BLOCKER for F6). Perlengkapan's AuthGate/RoleGate/SatkerGate were migrated to
 * the shared `lib-ui` primitives but only `cargo check`/wasm-verified (trunk
 * OOMs locally), never exercised in a browser. This is that missing runtime gate.
 *
 * Real-auth stack required (docker-compose.e2e.yml). The authenticated case
 * reuses the storageState from the `setup` project (real JWT in `auth_token`);
 * the unauthenticated case overrides storageState to empty.
 *
 * Assertions are URL/redirect-based (robust) rather than coupled to specific UI
 * copy. A protected route is used as the guard subject.
 */
import { test, expect } from '@playwright/test';

// A route behind AuthGate. NOTE the canonical microfrontend mount point is
// `/perlengkapan/simpel/` (Trunk public_url + <base href> + nginx try_files
// fallback). Routes are relative to that base, e.g. `/kebutuhan-bmn/daftar`
// (src/lib.rs). A bare `/perlengkapan/...` path 404s at nginx (no SPA fallback)
// so the WASM never boots and the guard can't run — use the full v2 path.
const PROTECTED_ROUTE = '/perlengkapan/simpel/kebutuhan-bmn/daftar';
const LOGIN_HINT = /login/i; // redirect target contains "login" (routes::path::LOGIN = /perlengkapan/simpel/login)

test.describe('Perlengkapan guards — unauthenticated (AuthGate redirect)', () => {
  // No session: fresh empty storage, ignore the project's authenticated state.
  test.use({ storageState: { cookies: [], origins: [] } });

  test('protected route redirects an unauthenticated visitor to login', async ({ page }) => {
    await page.goto(PROTECTED_ROUTE, { waitUntil: 'domcontentloaded' });
    // The guard sets window.location to the login path once the WASM app has mounted and run its
    // redirect effect. How long that takes depends on the host (the bundle download, the first
    // paint): a fixed 3 s sleep followed by one look passed on rc53's first staging run and failed
    // on its second, with the same code. Wait for the redirect instead, up to a generous bound.
    const redirectedToLogin = async () =>
      LOGIN_HINT.test(page.url()) ||
      // transient redirect copy from RedirectToPerlengkapanLogin
      (await page.getByText(/Mengalihkan ke halaman login|login|masuk/i).first().isVisible().catch(() => false));
    await expect
      .poll(redirectedToLogin, {
        timeout: 20_000,
        message: `unauthenticated access to ${PROTECTED_ROUTE} should redirect to login`,
      })
      .toBeTruthy();
  });
});

test.describe('Perlengkapan guards — authenticated (AuthGate passes)', () => {
  // Uses the project storageState (real JWT injected by auth.setup.ts).
  test('protected route renders for an authenticated user (no login redirect)', async ({ page }) => {
    await page.goto(PROTECTED_ROUTE, { waitUntil: 'domcontentloaded' });
    await page.waitForTimeout(3000);
    expect(
      LOGIN_HINT.test(page.url()),
      `authenticated access to ${PROTECTED_ROUTE} must NOT redirect to login (got ${page.url()})`,
    ).toBeFalsy();
  });
});
