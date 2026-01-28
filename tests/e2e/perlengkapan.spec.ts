import { test, expect } from '@playwright/test';

test.describe('Perlengkapan Microfrontend', () => {
  test('should load dashboard and display stats', async ({ page }) => {
    // Navigate directly to the Perlengkapan microfrontend
    // Assuming it runs on port 8093 (or proxied via 8080)
    // In E2E env, we usually target the main portal which proxies.
    // Let's assume localhost:8080/dashboard/perlengkapan or similar,
    // BUT the registry said url: "http://localhost:8093".
    // If we run the standalone app, we can hit 8093 directly.

    await page.goto('http://localhost:8093/dashboard');

    // Check for title
    await expect(page).toHaveTitle(/SIMPEL Perlengkapan/);

    // Check for Dashboard header
    await expect(page.locator('h1')).toContainText('Dashboard Perlengkapan');

    // Check for Stats cards (loaded async)
    // We expect "Total Aset" to appear
    await expect(page.getByText('Total Aset')).toBeVisible({ timeout: 10000 });
  });

  test('should navigate to asset list and show table', async ({ page }) => {
    await page.goto('http://localhost:8093/dashboard/bank-aset/daftar');

    // Check header
    await expect(page.locator('h2')).toContainText('Daftar Aset');

    // Check table headers
    await expect(page.getByText('No Aset')).toBeVisible();
    await expect(page.getByText('Nama Aset')).toBeVisible();
    await expect(page.getByText('Kondisi')).toBeVisible();

    // Check filter dropdown
    await expect(page.locator('select')).toBeVisible();
  });
});
