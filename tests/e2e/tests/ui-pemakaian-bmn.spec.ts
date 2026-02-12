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
 * E2E UI Test: Pemakaian BMN - Full Frontend Workflow
 *
 * Business Process:
 * 1. View daftar pemakaian BMN
 * 2. Create new pemakaian BMN (konsep surat)
 * 3. View pemakaian BMN detail
 * 4. Validator review workflow
 */
test.describe('UI Workflow: Pemakaian BMN', () => {
  test.describe.configure({ mode: 'serial' });

  test('Step 1: View daftar pemakaian BMN', async ({ page }) => {
    await loginAs(page, 'operator_satker');
    await page.goto('/perlengkapan/dashboard/pemakaian-bmn/daftar');
    await expect(page.getByText('Izin Pemakaian BMN')).toBeVisible({ timeout: 30000 });
    await page.screenshot({ path: 'test-results/ui-pemakaian-bmn-01-list.png', fullPage: true });
  });

  test('Step 2: Open create pemakaian BMN form', async ({ page }) => {
    await loginAs(page, 'operator_satker');
    await page.goto('/perlengkapan/dashboard/pemakaian-bmn/baru');
    // Should show form
    await expect(page.getByText('Permohonan Izin Pemakaian BMN')).toBeVisible({ timeout: 30000 });
    await page.screenshot({ path: 'test-results/ui-pemakaian-bmn-02-create-form.png', fullPage: true });
  });

  test('Step 3: Fill pemakaian BMN form', async ({ page }) => {
    await loginAs(page, 'operator_satker');
    await page.goto('/perlengkapan/dashboard/pemakaian-bmn/baru');
    await expect(page.getByText('Permohonan Izin Pemakaian BMN')).toBeVisible({ timeout: 30000 });

    // Fill form fields
    const inputs = page.locator('input:visible, textarea:visible, select:visible');
    const count = await inputs.count();

    for (let i = 0; i < Math.min(count, 5); i++) {
      const input = inputs.nth(i);
      const type = await input.getAttribute('type');
      const tag = await input.evaluate(el => el.tagName.toLowerCase());

      if (tag === 'select') {
        const options = input.locator('option');
        if (await options.count() > 1) {
          await input.selectOption({ index: 1 });
        }
      } else if (type === 'date') {
        await input.fill('2025-06-15');
      } else if (type === 'number') {
        await input.fill('1');
      } else if (tag === 'textarea') {
        await input.fill('E2E Test - Pemakaian BMN untuk keperluan operasional kantor');
      } else {
        await input.fill('E2E Test Data');
      }
    }

    await page.screenshot({ path: 'test-results/ui-pemakaian-bmn-03-filled-form.png', fullPage: true });

    // Try to submit
    const submitBtn = page.getByRole('button', { name: /simpan|submit|kirim/i }).first();
    if (await submitBtn.isVisible()) {
      await submitBtn.click();
      await page.waitForTimeout(1000);
    }
    await page.screenshot({ path: 'test-results/ui-pemakaian-bmn-04-after-submit.png', fullPage: true });
  });

  test('Step 4: Legacy pemakaian list', async ({ page }) => {
    await loginAs(page, 'operator_satker');
    await page.goto('/perlengkapan/dashboard/pengelolaan/pemakaian/daftar');
    await expect(page.getByText('Daftar Pemakaian BMN')).toBeVisible({ timeout: 30000 });
    await page.screenshot({ path: 'test-results/ui-pemakaian-bmn-05-legacy-list.png', fullPage: true });
  });

  test('Step 5: Create legacy pemakaian form', async ({ page }) => {
    await loginAs(page, 'operator_satker');
    await page.goto('/perlengkapan/dashboard/pengelolaan/pemakaian/baru');
    await expect(page.getByText('Catat Pemakaian BMN')).toBeVisible({ timeout: 30000 });

    // Fill form
    const namaInput = page.locator('input[name*="nama"], [aria-label*="Peminjam"], input').first();
    if (await namaInput.isVisible()) {
      await namaInput.fill('E2E User Peminjam');
    }

    await page.screenshot({ path: 'test-results/ui-pemakaian-bmn-06-legacy-form.png', fullPage: true });
  });
});
