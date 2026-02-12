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
 * E2E UI Test: Penghapusan BMN - Full Frontend Workflow
 *
 * Business Process:
 * 1. View daftar penghapusan BMN
 * 2. Create new penghapusan (operator satker)
 * 3. View penghapusan detail
 * 4. Validator review
 */
test.describe('UI Workflow: Penghapusan BMN', () => {
  test.describe.configure({ mode: 'serial' });

  test('Step 1: View daftar penghapusan BMN', async ({ page }) => {
    await loginAs(page, 'operator_satker');
    await page.goto('/perlengkapan/dashboard/pengelolaan/penghapusan');
    await expect(page.getByText('SK Penghapusan BMN')).toBeVisible({ timeout: 30000 });
    await page.screenshot({ path: 'test-results/ui-penghapusan-01-list.png', fullPage: true });
  });

  test('Step 2: Open create penghapusan form', async ({ page }) => {
    await loginAs(page, 'operator_satker');
    await page.goto('/perlengkapan/dashboard/pengelolaan/penghapusan/baru');
    await expect(page.getByText('Usul Penghapusan BMN')).toBeVisible({ timeout: 30000 });
    await page.screenshot({ path: 'test-results/ui-penghapusan-02-create-form.png', fullPage: true });
  });

  test('Step 3: Fill penghapusan form', async ({ page }) => {
    await loginAs(page, 'operator_satker');
    await page.goto('/perlengkapan/dashboard/pengelolaan/penghapusan/baru');
    await expect(page.getByText('Usul Penghapusan BMN')).toBeVisible({ timeout: 30000 });

    // Fill available form fields
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
        await input.fill('1000000');
      } else if (tag === 'textarea') {
        await input.fill('Barang rusak berat tidak dapat diperbaiki - E2E Test');
      } else if (type === 'text') {
        await input.fill('550e8400-e29b-41d4-a716-446655440000');
      }
    }

    await page.screenshot({ path: 'test-results/ui-penghapusan-03-filled-form.png', fullPage: true });

    const submitBtn = page.getByRole('button', { name: /simpan|submit|kirim|hapus/i }).first();
    if (await submitBtn.isVisible()) {
      await submitBtn.click();
      await page.waitForTimeout(1000);
    }
    await page.screenshot({ path: 'test-results/ui-penghapusan-04-after-submit.png', fullPage: true });
  });

  test('Step 4: View penghapusan detail (mock ID)', async ({ page }) => {
    await loginAs(page, 'operator_satker');
    await page.goto('/perlengkapan/dashboard/pengelolaan/penghapusan/00000000-0000-0000-0000-000000000001');
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(2000);
    await page.screenshot({ path: 'test-results/ui-penghapusan-05-detail.png', fullPage: true });
  });

  test('Step 5: Validator wilayah reviews penghapusan', async ({ page }) => {
    await loginAs(page, 'validator_wilayah');
    await page.goto('/perlengkapan/dashboard/pengelolaan/penghapusan');
    await expect(page.getByText('SK Penghapusan BMN')).toBeVisible({ timeout: 30000 });
    await page.screenshot({ path: 'test-results/ui-penghapusan-06-validator-review.png', fullPage: true });
  });
});
