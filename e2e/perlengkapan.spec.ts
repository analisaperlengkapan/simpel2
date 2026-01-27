import { test, expect } from '@playwright/test';

test.describe('Perlengkapan Module', () => {
  test('should navigate to dashboard and load statistics', async ({ page }) => {
    // Navigate to the app (assuming localhost:8093)
    await page.goto('http://localhost:8093');

    // Check for dashboard title
    await expect(page).toHaveTitle(/SIMPEL Perlengkapan/);

    // Check for dashboard stats
    await expect(page.getByText('Dashboard Perlengkapan')).toBeVisible();
    await expect(page.getByText('Total Aset')).toBeVisible();
  });

  test('should navigate to Bank Aset', async ({ page }) => {
    await page.goto('http://localhost:8093/dashboard/bank-aset/daftar');
    await expect(page.getByText('Daftar Aset')).toBeVisible();
    // Check table headers
    await expect(page.getByText('No Aset')).toBeVisible();
    await expect(page.getByText('Nama Aset')).toBeVisible();
  });

  test('should navigate to Pengadaan', async ({ page }) => {
    await page.goto('http://localhost:8093/dashboard/pengadaan/daftar');
    await expect(page.getByText('Daftar Pengadaan')).toBeVisible();
    await expect(page.getByRole('button', { name: 'Buat Pengadaan' })).toBeVisible();
  });

  test('should navigate to Analisis Kebutuhan', async ({ page }) => {
    await page.goto('http://localhost:8093/dashboard/analisis/daftar');
    await expect(page.getByText('Analisis Kebutuhan')).toBeVisible();
    await expect(page.getByRole('button', { name: 'Buat Analisis Baru' })).toBeVisible();
  });
});
