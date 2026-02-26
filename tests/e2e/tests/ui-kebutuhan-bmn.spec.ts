import { test, expect, Page } from '@playwright/test';

// Helper to set authenticated session
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
    localStorage.setItem('available_roles', JSON.stringify([
      'operator_satker', 'validator_wilayah', 'validator_pusat', 'admin',
    ]));
  }, { session: mockSession, role });
}

/**
 * E2E UI Test: Kebutuhan BMN (BMN Needs Analysis) - Full Frontend Workflow
 *
 * Business Process:
 * 1. Validator Pusat creates pengajuan kebutuhan BMN
 * 2. Operator Satker fills kebutuhan barang
 * 3. Operator Satker submits to Validator Wilayah
 * 4. Validator Wilayah reviews and forwards
 * 5. Validator Pusat approves/rejects
 */
test.describe('UI Workflow: Kebutuhan BMN', () => {
  test.describe.configure({ mode: 'serial' });

  test('Step 1: Access kebutuhan BMN dashboard', async ({ page }) => {
    await loginAs(page, 'validator_pusat');
    await page.goto('/perlengkapan/dashboard/kebutuhan-bmn/dashboard');
    await expect(page.getByText('Dashboard Analisis Kebutuhan BMN')).toBeVisible({ timeout: 30000 });
    await page.screenshot({ path: 'test-results/ui-kebutuhan-bmn-01-dashboard.png', fullPage: true });
  });

  test('Step 2: View kebutuhan BMN list', async ({ page }) => {
    await loginAs(page, 'validator_pusat');
    await page.goto('/perlengkapan/dashboard/kebutuhan-bmn/daftar');
    await expect(page.getByText('Analisis Kebutuhan BMN')).toBeVisible({ timeout: 30000 });
    await page.screenshot({ path: 'test-results/ui-kebutuhan-bmn-02-list.png', fullPage: true });
  });

  test('Step 3: Open create form', async ({ page }) => {
    await loginAs(page, 'validator_pusat');
    await page.goto('/perlengkapan/dashboard/kebutuhan-bmn/baru');
    // Form should be present
    await expect(page.getByText('Buat Pengajuan Kebutuhan BMN')).toBeVisible({ timeout: 30000 });
    await page.screenshot({ path: 'test-results/ui-kebutuhan-bmn-03-create-form.png', fullPage: true });
  });

  test('Step 4: Fill and submit kebutuhan BMN form', async ({ page }) => {
    await loginAs(page, 'validator_pusat');
    await page.goto('/perlengkapan/dashboard/kebutuhan-bmn/baru');

    // Try to fill form fields (different form implementations may have different labels)
    const namaInput = page.locator('input[name*="nama"], input[placeholder*="nama"], [aria-label*="nama"]').first();
    if (await namaInput.isVisible()) {
      await namaInput.fill('Kebutuhan BMN TA 2026 - E2E Test');
    }

    const deskripsiInput = page.locator('textarea, input[name*="deskripsi"], [aria-label*="deskripsi"]').first();
    if (await deskripsiInput.isVisible()) {
      await deskripsiInput.fill('Pengajuan kebutuhan BMN untuk testing');
    }

    // Screenshot the filled form
    await page.screenshot({ path: 'test-results/ui-kebutuhan-bmn-04-fill-form.png', fullPage: true });

    // Try to submit
    const submitButton = page.getByRole('button', { name: /simpan|submit|kirim|buat/i }).first();
    if (await submitButton.isVisible() && await submitButton.isEnabled()) {
      await submitButton.click();
      await page.waitForTimeout(1000);
    }
    await page.screenshot({ path: 'test-results/ui-kebutuhan-bmn-05-after-submit.png', fullPage: true });
  });

  test('Step 5: Operator Satker views kebutuhan BMN detail', async ({ page }) => {
    await loginAs(page, 'operator_satker');
    await page.goto('/perlengkapan/dashboard/kebutuhan-bmn/daftar');
    await expect(page.getByText('Analisis Kebutuhan BMN')).toBeVisible({ timeout: 30000 });
    await page.screenshot({ path: 'test-results/ui-kebutuhan-bmn-06-operator-list.png', fullPage: true });
  });

  test('Step 6: Navigate to satker detail', async ({ page }) => {
    await loginAs(page, 'operator_satker');
    await page.goto('/perlengkapan/dashboard/kebutuhan-bmn/satker/SKR001');
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(2000);
    await page.screenshot({ path: 'test-results/ui-kebutuhan-bmn-07-satker-detail.png', fullPage: true });
  });
});
