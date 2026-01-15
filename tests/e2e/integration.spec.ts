import { test, expect } from '@playwright/test';

test.describe('Portal Frontend Integration', () => {
    const PORTAL_URL = 'http://localhost:8090';

    test('Complete login and secret management flow', async ({ page }) => {
        // 1. Navigate to Login Page
        page.on('console', msg => console.log(`BROWSER: ${msg.text()}`));
        page.on('requestfailed', request => console.log(`REQ_FAILED: ${request.url()} - ${request.failure()?.errorText}`));

        await page.goto(`${PORTAL_URL}/login`);
        await expect(page).toHaveTitle(/SIMPelv2/);

        // Wait for loading screen to disappear
        await page.waitForSelector('#loading-screen', { state: 'hidden', timeout: 30000 });

        // 2. Perform Login
        await page.fill('#username', 'admin');
        await page.fill('#password', 'admin123');

        // 3. Solve CAPTCHA (Visual challenge: 2 + 2 = 4)
        // Wait for the captcha challenge to load
        await expect(page.locator('.captcha-container')).toBeVisible({ timeout: 10000 });
        await page.click('button:has-text("4")');
        // Use first() because there might be multiple (status indicator and feedback)
        await expect(page.locator('text=Verification successful!').first()).toBeVisible({ timeout: 10000 });

        // 4. Submit Login
        await page.click('button:has-text("Masuk ke Portal")');

        // 5. Verify Dashboard Loading
        await expect(page).toHaveURL(/.*dashboard/);
        await expect(page.locator('h1')).toContainText('Dashboard');

        // 6. Navigate to Secret Management
        await page.goto(`${PORTAL_URL}/secrets`);
        await expect(page.locator('h1')).toContainText('Manajemen Rahasia');

        // 7. Add a New Secret
        await page.click('text=Tambah Rahasia');

        const path = `tests/e2e/secret-${Date.now()}`;
        await page.fill('input[placeholder="e.g. app/config/database"]', path);
        await page.fill('input[placeholder="Key"]', 'api_key');
        await page.fill('input[placeholder="Value"]', 'secret-value-123');
        await page.fill('textarea', 'Created by Playwright E2E test');

        await page.click('button:has-text("Simpan")');

        // 8. Verify the secret is listed
        await expect(page.locator(`text=${path}`)).toBeVisible();

        // 9. Logout
        await page.click('text=Logout');
        await expect(page).toHaveURL(/.*login/);
    });
});
