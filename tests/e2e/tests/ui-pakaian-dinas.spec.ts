import { test, expect, Page } from '@playwright/test';

async function loginAs(page: Page, role: string) {
  const mockSession = JSON.stringify({
    id: '20000000-0000-0000-0000-000000000001',
    username: '199203142014031001',
    name: 'User E2E',
    email: 'e2e@kejaksaan.go.id',
    role: { Custom: role },
    avatar: null,
    division: 'Biro Perlengkapan',
    captcha_validated: true,
    mfa_enabled: false,
    mfa_setup_required: false,
    created_at: new Date().toISOString(),
    access_token: 'mock-jwt-token',
    refresh_token: 'mock-refresh',
    expires_at: Math.floor(Date.now() / 1000) + 86400,
    permissions: role === 'admin' ? ['*'] : [],
  });
  // Use addInitScript so localStorage is set BEFORE any page scripts run
  await page.addInitScript(({ session, role }) => {
    localStorage.setItem('user_session', session);
    localStorage.setItem('auth_token', 'mock-jwt-token-for-e2e');
    localStorage.setItem('active_role', role);
  }, { session: mockSession, role });
}

/**
 * E2E UI Test: Pakaian Dinas - Full Frontend Workflow
 *
 * Business Process:
 * 1. View jenis pakaian dinas list (master data)
 * 2. View pengajuan pakaian dinas list
 * 3. View ukuran pegawai per satker
 * 4. View laporan pakaian dinas
 */
test.describe('UI Workflow: Pakaian Dinas', () => {
  test.describe.configure({ mode: 'serial' });

  test('Step 1: View jenis pakaian dinas master list', async ({ page }) => {
    await loginAs(page, 'operator_satker');
    await page.goto('/perlengkapan/dashboard/pakaian-dinas/jenis');
    await expect(page.getByRole('heading', { name: 'Jenis Pakaian Dinas' })).toBeVisible({ timeout: 30000 });
    await page.screenshot({ path: 'test-results/ui-pakaian-dinas-01-jenis-list.png', fullPage: true });
  });

  test('Step 2: View pengajuan pakaian dinas list', async ({ page }) => {
    await loginAs(page, 'operator_satker');
    await page.goto('/perlengkapan/dashboard/pakaian-dinas/pengajuan');
    await expect(page.getByRole('heading', { name: 'Pengajuan Pakaian Dinas' })).toBeVisible({ timeout: 30000 });
    await page.screenshot({ path: 'test-results/ui-pakaian-dinas-02-pengajuan-list.png', fullPage: true });
  });

  test('Step 3: View ukuran pegawai per satker', async ({ page }) => {
    await loginAs(page, 'operator_satker');
    await page.goto('/perlengkapan/dashboard/pakaian-dinas/ukuran/SKR001');
    await expect(page.getByText('Ukuran Pakaian Pegawai')).toBeVisible({ timeout: 30000 });
    await page.screenshot({ path: 'test-results/ui-pakaian-dinas-03-ukuran-satker.png', fullPage: true });
  });

  test('Step 4: View laporan pakaian dinas', async ({ page }) => {
    await loginAs(page, 'operator_satker');
    await page.goto('/perlengkapan/dashboard/pakaian-dinas/laporan');
    await expect(page.getByRole('heading', { name: 'Laporan Pakaian Dinas' })).toBeVisible({ timeout: 30000 });
    await page.screenshot({ path: 'test-results/ui-pakaian-dinas-04-laporan.png', fullPage: true });
  });

  test('Step 5: View rekap laporan', async ({ page }) => {
    await loginAs(page, 'operator_satker');
    await page.goto('/perlengkapan/dashboard/pakaian-dinas/laporan/rekap');
    await expect(page.getByRole('heading', { name: 'Laporan Pakaian Dinas' })).toBeVisible({ timeout: 30000 });
    await page.screenshot({ path: 'test-results/ui-pakaian-dinas-05-rekap.png', fullPage: true });
  });
});
