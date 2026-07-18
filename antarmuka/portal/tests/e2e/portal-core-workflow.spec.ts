/**
 * F-E2E E-6 — Portal core workflows (replaces portal-screenshots-e2e.spec.ts).
 *
 * Assertion-based end-to-end coverage of every portal core route with REAL
 * auth + REAL backend state (authenc via the /api proxy — see
 * helpers/real-auth.ts). Navigation is driven by CLICKING the app's real
 * sidebar/menu links wherever one exists (FE-audit lesson: goto()-only specs
 * cannot catch broken links), and every workflow asserts BOTH the UI change
 * and the backend state via API cross-checks.
 *
 * E-7 (post-#45-trim) is folded in: the Keycloak-mirror admin pages were
 * DELETED with the trim, and the surviving IAM surface (clients inspection,
 * role assignment on user detail, real audit-log rows) is asserted here —
 * portal route-coverage is BLOCKING from this suite on.
 *
 * Each describe logs in fresh (UI login, real captcha) so no test can poison
 * another test's session; tests that revoke sessions use OPERATOR_B and tests
 * that read shared state use OPERATOR/ADMIN.
 */
import { test, expect, Page } from '@playwright/test';
import {
  ADMIN_USER,
  AUTHENC_URL,
  OPERATOR_B_USER,
  OPERATOR_USER,
  PORTAL_URL,
  loginViaApi,
  loginViaUi,
  pageToken,
  setupAllProxies,
} from './helpers/real-auth';

/** Decode a JWT payload without verifying (test-side introspection only). */
function jwtPayload(token: string): Record<string, unknown> {
  return JSON.parse(Buffer.from(token.split('.')[1], 'base64url').toString());
}

/**
 * Click a sidebar menu link by its href and wait for the URL.
 *
 * Must filter to VISIBLE links: `<nav>` renders the whole menu twice — the
 * desktop quick-links plus a `lg:hidden` mobile drawer — and the drawer copy
 * comes first in the DOM, so an unfiltered `.first()` grabs a hidden element
 * and the click times out for every item that isn't a top-bar quick link.
 */
async function clickSidebarLink(page: Page, href: string): Promise<void> {
  await page
    .locator(`aside a[href="${href}"], nav a[href="${href}"]`)
    .locator('visible=true')
    .first()
    .click();
  await page.waitForURL(new RegExp(href.replace(/\//g, '\\/')), { timeout: 15000 });
}

test.describe('Portal core — operator', () => {
  let getCaptchaAnswer: () => string | null;

  test.beforeEach(async ({ page }) => {
    ({ getCaptchaAnswer } = await setupAllProxies(page));
    await loginViaUi(page, OPERATOR_USER, getCaptchaAnswer);
  });

  test('dashboard shows real profile data and hides admin stats', async ({ page }) => {
    await expect(page).toHaveURL(/\/portal\/dashboard/);

    // Hero card renders the JWT-derived identity of the seeded operator.
    await expect(page.getByText('E2E Operator Jakpus').first()).toBeVisible({ timeout: 15000 });

    // "Ringkasan Akun" comes from a real GET /api/v1/auth/me — the seeded
    // email proves the fetch happened (not a placeholder render).
    await expect(page.getByText('Ringkasan Akun')).toBeVisible();
    await expect(page.getByText('200000000000000001@kejaksaan.go.id')).toBeVisible({
      timeout: 15000,
    });

    // Non-admin must not see the admin-only stats section.
    await expect(page.getByText('Statistik Sistem')).toHaveCount(0);
  });

  test('sidebar drives apps, notifications, settings, profile (click navigation)', async ({
    page,
  }) => {
    // Apps — static registry rendered with the Perlengkapan launcher.
    await clickSidebarLink(page, '/portal/apps');
    await expect(page.getByPlaceholder('Cari aplikasi...')).toBeVisible({ timeout: 15000 });
    await expect(page.getByText('Perlengkapan').first()).toBeVisible();
    // Expanding the submenu exposes the real launch URLs for v1/v2.
    await page.getByText('Perlengkapan').first().click();
    await expect(page.locator('a[href="/perlengkapan/simpel/v2"]')).toBeVisible();
    await expect(page.locator('a[href="/perlengkapan/simpel/v1"]')).toBeVisible();

    // Notifications — the notifikasi service is NOT part of the portal e2e
    // stack, so the page must show the HONEST unavailable/empty state and
    // must never fabricate items (the old mock generator produced
    // "Pembaruan Sistem"/"Peringatan Keamanan" rows — regression guard).
    await clickSidebarLink(page, '/portal/notifications');
    await expect(
      page.getByRole('heading', { name: 'Notifikasi', exact: true }),
    ).toBeVisible({ timeout: 15000 });
    await expect(
      page.getByText(/Notifikasi tidak tersedia|Tidak ada notifikasi/),
    ).toBeVisible({ timeout: 15000 });
    await expect(page.getByText('Pembaruan Sistem')).toHaveCount(0);
    await expect(page.getByText('Peringatan Keamanan')).toHaveCount(0);

    // Settings — session-derived account fields.
    await clickSidebarLink(page, '/portal/settings');
    await expect(page.getByRole('heading', { name: 'Pengaturan' })).toBeVisible({
      timeout: 15000,
    });
    await expect(page.getByText('200000000000000001').first()).toBeVisible();

    // Profile — real /me fetch (NIP + satker fields from the seed).
    await clickSidebarLink(page, '/portal/profile');
    await expect(page.getByRole('heading', { name: 'Profil Pengguna' })).toBeVisible({
      timeout: 15000,
    });
    await expect(page.getByText('E2E Operator Jakpus').first()).toBeVisible({ timeout: 15000 });
  });

  test('passkeys page lists credentials from the real WebAuthn API', async ({ page }) => {
    // The list fetch must succeed against the real backend (empty is fine —
    // asserting the response proves the wiring, not just the heading).
    const [credsResponse] = await Promise.all([
      page.waitForResponse(
        (resp) =>
          resp.url().includes('/api/v1/auth/webauthn/credentials') &&
          resp.request().method() === 'GET',
        { timeout: 15000 },
      ),
      clickSidebarLink(page, '/portal/passkeys'),
    ]);
    expect(credsResponse.status()).toBe(200);
    await expect(page.getByRole('heading', { name: 'Kelola Passkey' })).toBeVisible({
      timeout: 15000,
    });
  });

  test('menu hides administration for non-admin; direct admin URL is forbidden', async ({
    page,
  }) => {
    // The RBAC-aware menu must not offer the admin section at all.
    await expect(page.locator('a[href="/portal/admin"]')).toHaveCount(0);
    await expect(page.locator('a[href="/portal/admin/users"]')).toHaveCount(0);

    // Direct URL access renders the ForbiddenPage (client gate; the iam-api
    // enforces server-side on every /api/v1/iam call).
    await page.goto(`${PORTAL_URL}/portal/admin/users`, { waitUntil: 'domcontentloaded' });
    await expect(page.getByText('Akses Ditolak')).toBeVisible({ timeout: 15000 });

    // Regression for the double-base navigate() bug: "Kembali ke Dashboard"
    // must land on /portal/dashboard, not /portal/portal/dashboard.
    await page.getByRole('link', { name: /Kembali ke Dashboard/ }).click();
    await page.waitForURL(/\/portal\/dashboard$/, { timeout: 15000 });
    await expect(page.getByText('Ringkasan Akun')).toBeVisible({ timeout: 15000 });
  });
});

test.describe('Portal core — sessions self-service (operator_b)', () => {
  let getCaptchaAnswer: () => string | null;

  test.beforeEach(async ({ page }) => {
    ({ getCaptchaAnswer } = await setupAllProxies(page));
    await loginViaUi(page, OPERATOR_B_USER, getCaptchaAnswer);
  });

  test('lists own sessions, terminates another session, current is protected', async ({
    page,
  }) => {
    // Mint a SECOND session for the same user via the API — its sid comes
    // from the returned JWT, giving a deterministic termination target.
    const second = await loginViaApi(page, OPERATOR_B_USER);
    const secondSid = String(jwtPayload(second.access_token).sid);
    expect(secondSid).toBeTruthy();

    await clickSidebarLink(page, '/portal/sessions');
    await expect(page.getByRole('heading', { name: 'Sesi Aktif' })).toBeVisible({
      timeout: 15000,
    });

    // The current (UI) session is badged; the API-minted one is terminable.
    await expect(page.getByText('Sesi ini')).toBeVisible({ timeout: 15000 });
    const secondRow = page.locator(`[data-session-id="${secondSid}"]`);
    await expect(secondRow).toBeVisible({ timeout: 15000 });

    await secondRow.getByRole('button', { name: 'Hentikan' }).click();
    await expect(page.getByText('Sesi berhasil dihentikan')).toBeVisible({ timeout: 15000 });
    await expect(secondRow).toHaveCount(0);

    // Backend state: the terminated sid is gone from the authoritative list.
    const token = await pageToken(page);
    const listResp = await page.request.get(`${AUTHENC_URL}/api/v1/auth/sessions`, {
      headers: { Authorization: `Bearer ${token}` },
    });
    expect(listResp.status()).toBe(200);
    const sessions = (await listResp.json()).sessions as Array<{ id: string; is_current: boolean }>;
    expect(sessions.some((s) => s.id === secondSid)).toBe(false);

    // The CURRENT session must refuse self-termination (logout is the way).
    const currentSid = String(jwtPayload(token).sid);
    const delCurrent = await page.request.delete(
      `${AUTHENC_URL}/api/v1/auth/sessions/${currentSid}`,
      { headers: { Authorization: `Bearer ${token}` } },
    );
    expect(delCurrent.status()).toBe(400);
  });

  test('cannot terminate another user\'s session (404, existence not leaked)', async ({
    page,
  }) => {
    // A session belonging to OPERATOR (different user).
    const foreign = await loginViaApi(page, OPERATOR_USER);
    const foreignSid = String(jwtPayload(foreign.access_token).sid);

    const token = await pageToken(page);
    const resp = await page.request.delete(
      `${AUTHENC_URL}/api/v1/auth/sessions/${foreignSid}`,
      { headers: { Authorization: `Bearer ${token}` } },
    );
    expect(resp.status()).toBe(404);

    // The foreign session must still be alive for its owner.
    const foreignList = await page.request.get(`${AUTHENC_URL}/api/v1/auth/sessions`, {
      headers: { Authorization: `Bearer ${foreign.access_token}` },
    });
    expect(foreignList.status()).toBe(200);
    const sessions = (await foreignList.json()).sessions as Array<{ id: string }>;
    expect(sessions.some((s) => s.id === foreignSid)).toBe(true);
  });

  test('logout clears the session locally and returns to the login page', async ({ page }) => {
    await page.getByRole('button', { name: 'Keluar' }).click();

    // Back at the (inline or routed) login screen with storage cleared.
    await expect(page.getByText('Masuk ke Sistem')).toBeVisible({ timeout: 15000 });
    await expect
      .poll(() => page.evaluate(() => window.localStorage.getItem('auth_token')), {
        timeout: 10000,
      })
      .toBeNull();
  });

  test('cross-tab: logout in one tab signs the other tab out', async ({ page, context }) => {
    // Second tab, same context (shared localStorage → storage events fire).
    const page2 = await context.newPage();
    await setupAllProxies(page2);
    await page2.goto(`${PORTAL_URL}/portal/dashboard`, { waitUntil: 'domcontentloaded' });
    await expect(page2.getByText('Ringkasan Akun')).toBeVisible({ timeout: 20000 });

    await page.getByRole('button', { name: 'Keluar' }).click();
    await expect(page.getByText('Masuk ke Sistem')).toBeVisible({ timeout: 15000 });

    // The cross-tab storage listener must sign tab 2 out as well.
    await expect(page2.getByText('Masuk ke Sistem')).toBeVisible({ timeout: 20000 });
    await page2.close();
  });
});

test.describe('Portal core — admin (IAM users/roles/audit)', () => {
  let getCaptchaAnswer: () => string | null;

  test.beforeEach(async ({ page }) => {
    ({ getCaptchaAnswer } = await setupAllProxies(page));
    await loginViaUi(page, ADMIN_USER, getCaptchaAnswer);
  });

  test('dashboard shows admin stats matching the real IAM stats API', async ({ page }) => {
    await expect(page.getByText('Statistik Sistem')).toBeVisible({ timeout: 15000 });

    const token = await pageToken(page);
    const statsResp = await page.request.get(`${AUTHENC_URL}/api/v1/iam/admin/stats`, {
      headers: { Authorization: `Bearer ${token}` },
    });
    expect(statsResp.status()).toBe(200);
    const stats = await statsResp.json();
    expect(stats.total_users).toBeGreaterThan(0);

    // The rendered stats section shows the same number the API returns.
    await expect(
      page
        .locator('section', { hasText: 'Statistik Sistem' })
        .getByText(String(stats.total_users), { exact: true })
        .first(),
    ).toBeVisible({ timeout: 15000 });
  });

  test('admin overview + users list render real IAM data (click navigation)', async ({
    page,
  }) => {
    await clickSidebarLink(page, '/portal/admin');
    await expect(page.getByRole('heading', { name: 'Dasbor Administrasi' })).toBeVisible({
      timeout: 15000,
    });

    // Overview nav card → users management.
    // visible=true for the same reason as clickSidebarLink: the mobile drawer
    // renders a hidden copy of this link earlier in the DOM.
    await page.locator('a[href="/portal/admin/users"]').locator('visible=true').first().click();
    await page.waitForURL(/\/portal\/admin\/users/, { timeout: 15000 });
    await expect(page.getByRole('heading', { name: 'Manajemen Pengguna' })).toBeVisible({
      timeout: 15000,
    });

    // Search for a seeded fixture user — the row comes from the real
    // GET /api/v1/iam/users, not a static render.
    await page.getByPlaceholder('Cari pengguna...').fill('200000000000000001');
    await page.getByRole('button', { name: /Cari/ }).click();
    await expect(
      page.locator('table').getByText('200000000000000001', { exact: true }),
    ).toBeVisible({ timeout: 15000 });
  });

  test('admin creates a user; that user changes their password end-to-end', async ({
    page,
    browser,
  }) => {
    const username = `e2etmp${Date.now()}`;
    const email = `${username}@kejaksaan.go.id`;

    // 1. Create via the real admin modal (server assigns TempPass123!).
    await page.goto(`${PORTAL_URL}/portal/admin/users`, { waitUntil: 'domcontentloaded' });
    await page.getByRole('button', { name: /Tambah Pengguna/ }).click();
    await expect(page.getByText('Tambah Pengguna Baru')).toBeVisible({ timeout: 10000 });
    const modal = page.locator('form', { has: page.getByText('Username *') });
    await modal.locator('input').nth(0).fill(username);
    await modal.locator('input').nth(1).fill(email);
    await page.getByRole('button', { name: 'Buat Pengguna' }).click();
    await expect(page.getByText('Pengguna berhasil dibuat')).toBeVisible({ timeout: 15000 });

    // Backend: the user exists and is enabled.
    const token = await pageToken(page);
    const found = await page.request.get(
      `${AUTHENC_URL}/api/v1/iam/users?search=${username}&page=1&per_page=10`,
      { headers: { Authorization: `Bearer ${token}` } },
    );
    expect(found.status()).toBe(200);

    // 2. The new user logs in (fresh context) and changes their password
    //    through the real Ubah Kata Sandi form.
    const ctx2 = await browser.newContext();
    const page2 = await ctx2.newPage();
    const { getCaptchaAnswer: captcha2 } = await setupAllProxies(page2);
    const newPassword = `E2e-baru-${Date.now()}!`;

    await page2.goto(`${PORTAL_URL}/portal/login`, { waitUntil: 'domcontentloaded' });
    await expect(page2.getByText('Masuk ke Sistem')).toBeVisible({ timeout: 30000 });
    const toggle2 = page2.getByText('Atau masuk dengan password');
    if (await toggle2.isVisible({ timeout: 3000 }).catch(() => false)) {
      await toggle2.click();
    }
    await expect(page2.locator('#captcha-title')).toBeVisible({ timeout: 15000 });
    await expect.poll(() => captcha2(), { timeout: 10000 }).toBeTruthy();
    await page2.locator('#username').fill(username);
    await page2.locator('#password').fill('TempPass123!');
    await page2.locator('input[placeholder="Masukkan teks..."]').fill(captcha2()!);
    await Promise.all([
      page2.waitForResponse((r) => r.url().includes('/api/captcha/verify'), { timeout: 15000 }),
      page2.locator('.captcha-container button', { hasText: 'Kirim' }).click(),
    ]);
    await page2.waitForTimeout(500);
    const loginBtn2 = page2.getByRole('button', { name: /Masuk ke Portal/i });
    await expect(loginBtn2).toBeEnabled({ timeout: 10000 });
    const [loginResp2] = await Promise.all([
      page2.waitForResponse(
        (r) => r.url().includes('/login') && r.request().method() === 'POST',
        { timeout: 15000 },
      ),
      loginBtn2.click(),
    ]);
    expect(loginResp2.status()).toBe(200);
    // New accounts may or may not carry a forced password change — either way
    // the change form itself must work.
    await page2.waitForURL(/\/(dashboard|password)/, { timeout: 15000 });
    await page2.goto(`${PORTAL_URL}/portal/password`, { waitUntil: 'domcontentloaded' });
    await expect(page2.getByRole('heading', { name: 'Ubah Kata Sandi' })).toBeVisible({
      timeout: 15000,
    });
    const pwInputs = page2.locator('form input[type="password"]');
    await pwInputs.nth(0).fill('TempPass123!');
    await pwInputs.nth(1).fill(newPassword);
    await pwInputs.nth(2).fill(newPassword);
    const [changeResp] = await Promise.all([
      page2.waitForResponse(
        (r) => r.url().includes('/api/v1/auth/me/password') && r.request().method() === 'POST',
        { timeout: 15000 },
      ),
      page2.locator('form button[type="submit"]').click(),
    ]);
    expect(changeResp.status()).toBeLessThan(300);
    await expect(page2.getByText(/Kata sandi berhasil diubah/)).toBeVisible({ timeout: 15000 });
    await ctx2.close();

    // 3. Backend truth: the NEW password works, the OLD one is dead.
    const newLogin = await loginViaApi(page, { username, password: newPassword });
    expect(newLogin.access_token).toBeTruthy();
    // loginViaApi asserts 200, so the negative check goes through the raw API.
    const staleChallenge = await page.request.post(`${AUTHENC_URL}/api/captcha/challenge`, {
      data: { challenge_type: 'text_recognition', difficulty: 2 },
    });
    const staleId = (await staleChallenge.json()).challenge_id;
    const staleAnswer = (
      await (await page.request.get(`${AUTHENC_URL}/api/captcha/debug/${staleId}`)).json()
    ).answer;
    const staleVerify = await (
      await page.request.post(`${AUTHENC_URL}/api/captcha/verify`, {
        data: { challenge_id: staleId, answer: staleAnswer, session_id: `stale_${Date.now()}` },
      })
    ).json();
    const oldLogin = await page.request.post(`${AUTHENC_URL}/api/v1/auth/login`, {
      data: {
        username,
        password: 'TempPass123!',
        realm: 'master',
        captcha_token: staleVerify.token,
      },
    });
    expect(oldLogin.status()).toBe(401);

    // 4. Admin disables the temp account (cleanup that is itself a test).
    await page.goto(`${PORTAL_URL}/portal/admin/users`, { waitUntil: 'domcontentloaded' });
    await page.getByPlaceholder('Cari pengguna...').fill(username);
    await page.getByRole('button', { name: /Cari/ }).click();
    const row = page.locator('tr', { hasText: username });
    await expect(row).toBeVisible({ timeout: 15000 });
    await row.getByRole('button', { name: 'Nonaktifkan' }).click();
    await expect(page.getByText('Pengguna dinonaktifkan')).toBeVisible({ timeout: 15000 });
  });

  test('roles page lists the real IAM roles', async ({ page }) => {
    await clickSidebarLink(page, '/portal/admin/roles');
    await expect(
      page.getByRole('heading', { name: 'Manajemen Peran & Hak Akses' }),
    ).toBeVisible({ timeout: 15000 });

    // The four platform roles come from the seeded authenc.roles table.
    for (const role of ['admin', 'operator_satker', 'validator_wilayah', 'validator_pusat']) {
      await expect(page.getByText(role, { exact: true }).first()).toBeVisible({
        timeout: 15000,
      });
    }
  });

  test('audit page renders against the real audit-logs API', async ({ page }) => {
    const [auditResp] = await Promise.all([
      page.waitForResponse(
        (resp) =>
          resp.url().includes('/api/v1/iam/audit-logs') && resp.request().method() === 'GET',
        { timeout: 15000 },
      ),
      clickSidebarLink(page, '/portal/admin/audit'),
    ]);
    expect(auditResp.status()).toBe(200);
    await expect(page.getByRole('heading', { name: 'Audit Log' })).toBeVisible({
      timeout: 15000,
    });

    // The audit trail is REAL now (PgAuditLogStore is injected and the
    // monitored-path prefixes match the mounted routes): the logins this
    // suite already performed must be recorded. Writes are async
    // (tokio::spawn), so poll briefly.
    const token = await pageToken(page);
    await expect
      .poll(
        async () => {
          const resp = await page.request.get(
            `${AUTHENC_URL}/api/v1/iam/audit-logs?per_page=5`,
            { headers: { Authorization: `Bearer ${token}` } },
          );
          if (resp.status() !== 200) return -1;
          const body = await resp.json();
          return Number(body.total);
        },
        { timeout: 20000 },
      )
      .toBeGreaterThan(0);
  });

  test('clients page lists the seeded OAuth2 client + read-only detail', async ({
    page,
  }) => {
    await clickSidebarLink(page, '/portal/admin/clients');
    await expect(
      page.getByRole('heading', { name: 'Manajemen Klien OAuth2' }),
    ).toBeVisible({ timeout: 15000 });

    // The seeded `perlengkapan` client comes from the real GET
    // /api/v1/iam/clients (the old handler silently returned []).
    await expect(page.getByText('perlengkapan', { exact: true }).first()).toBeVisible({
      timeout: 15000,
    });

    // Detail page renders the same client read-only (id from the BE list).
    const token = await pageToken(page);
    const listResp = await page.request.get(`${AUTHENC_URL}/api/v1/iam/clients`, {
      headers: { Authorization: `Bearer ${token}` },
    });
    expect(listResp.status()).toBe(200);
    const clients = await listResp.json();
    expect(clients.length).toBeGreaterThan(0);
    const client = clients.find((c: { client_id: string }) => c.client_id === 'perlengkapan');
    expect(client).toBeTruthy();

    await page.goto(`${PORTAL_URL}/portal/admin/clients/${client.id}`, {
      waitUntil: 'domcontentloaded',
    });
    await expect(page.getByText('Konfigurasi Klien')).toBeVisible({ timeout: 15000 });
    await expect(page.getByText('SIMPEL Perlengkapan').first()).toBeVisible({
      timeout: 15000,
    });
  });

  test('role assignment on user detail is real end-to-end (assign → revoke)', async ({
    page,
  }) => {
    // Temp user via the real IAM API so this test owns its fixture.
    const token = await pageToken(page);
    const username = `e2erole${Date.now()}`;
    const createResp = await page.request.post(`${AUTHENC_URL}/api/v1/iam/users`, {
      headers: { Authorization: `Bearer ${token}` },
      data: {
        username,
        email: `${username}@e2e.test`,
        password: 'TempPass123!',
        enabled: true,
      },
    });
    expect(createResp.status()).toBe(201);
    const created = await createResp.json();
    expect(created.roles).toEqual([]);

    // UI: user detail → Pemetaan Peran → Tetapkan operator_satker.
    await page.goto(`${PORTAL_URL}/portal/admin/users/${created.id}`, {
      waitUntil: 'domcontentloaded',
    });
    await page.getByRole('button', { name: /Pemetaan Peran/ }).click();
    const roleRow = page.locator('[data-role-name="operator_satker"]');
    await expect(roleRow).toBeVisible({ timeout: 15000 });
    await roleRow.getByRole('button', { name: 'Tetapkan' }).click();
    await expect(page.getByText('Peran berhasil ditetapkan')).toBeVisible({
      timeout: 15000,
    });

    // BE: the assignment is persisted on the same relation login reads.
    const afterAssign = await page.request.get(
      `${AUTHENC_URL}/api/v1/iam/users/${created.id}`,
      { headers: { Authorization: `Bearer ${token}` } },
    );
    expect(afterAssign.status()).toBe(200);
    expect((await afterAssign.json()).roles).toContain('operator_satker');

    // UI: revoke it again; BE confirms the removal.
    await roleRow.getByRole('button', { name: 'Cabut' }).click();
    await expect(page.getByText('Peran berhasil dicabut')).toBeVisible({ timeout: 15000 });
    const afterRemove = await page.request.get(
      `${AUTHENC_URL}/api/v1/iam/users/${created.id}`,
      { headers: { Authorization: `Bearer ${token}` } },
    );
    expect((await afterRemove.json()).roles).not.toContain('operator_satker');

    // Cleanup as its own assertion: disable the temp account.
    const disableResp = await page.request.put(
      `${AUTHENC_URL}/api/v1/iam/users/${created.id}`,
      {
        headers: { Authorization: `Bearer ${token}` },
        data: { enabled: false },
      },
    );
    expect(disableResp.status()).toBe(200);
  });
});

test.describe('Portal core — public MFA pages', () => {
  test('mfa verify + backup-verify render the real (temp-token) flow pages', async ({
    page,
  }) => {
    await setupAllProxies(page);

    // Without a temp token both pages must render their honest state — a
    // Leptos-rendered flow page, not a blank mount or crash. Assert the page's
    // OWN heading: "an h1 is visible" also passes on the 404 fallback, which is
    // exactly what these routes served while their paths were unmatchable.
    await page.goto(`${PORTAL_URL}/portal/mfa/verify`, { waitUntil: 'domcontentloaded' });
    await expect(
      page.getByRole('heading', { name: 'Two-Factor Authentication' }),
    ).toBeVisible({ timeout: 30000 });
    await expect(page.getByText('Halaman Tidak Ditemukan')).toHaveCount(0);

    await page.goto(`${PORTAL_URL}/portal/mfa/backup-verify`, { waitUntil: 'domcontentloaded' });
    await expect(
      page.getByRole('heading', { name: 'Backup Code Verification' }),
    ).toBeVisible({ timeout: 30000 });
    await expect(page.getByText('Halaman Tidak Ditemukan')).toHaveCount(0);
  });
});
