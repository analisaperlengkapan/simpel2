import { test, expect } from '@playwright/test';

test.describe('Perlengkapan Module', () => {
  test('should navigate to dashboard and load statistics', async ({ page }) => {
    // Navigate to the app (using relative URL, relies on baseURL)
    await page.goto('/');

    // Check for dashboard title
    await expect(page).toHaveTitle(/SIMPEL Perlengkapan/);

    // Check for dashboard stats
    await expect(page.getByText('Dashboard Perlengkapan')).toBeVisible();
    await expect(page.getByText('Total Aset')).toBeVisible();
  });

  test('should navigate to Bank Aset', async ({ page }) => {
    await page.goto('/dashboard/bank-aset/daftar');
    await expect(page.getByText('Daftar Aset')).toBeVisible();
    // Check table headers
    await expect(page.getByText('No Aset')).toBeVisible();
    await expect(page.getByText('Nama Aset')).toBeVisible();
  });

  test('should navigate to Pengadaan and open create form', async ({ page }) => {
    await page.goto('/dashboard/pengadaan/daftar');
    await expect(page.getByText('Daftar Pengadaan')).toBeVisible();

    // Click create button
    await page.getByRole('link', { name: 'Buat Pengadaan' }).click();

    // Verify form opened
    await expect(page.getByText('Buat Pengadaan Baru')).toBeVisible();
    await expect(page.getByLabel('Judul Pengadaan')).toBeVisible();
  });

  test('should navigate to Analisis Kebutuhan and open create form', async ({ page }) => {
    await page.goto('/dashboard/analisis/daftar');
    await expect(page.getByText('Analisis Kebutuhan')).toBeVisible();

    // Click create button
    await page.getByRole('link', { name: 'Buat Analisis Baru' }).click();

    // Verify form opened
    await expect(page.getByText('Buat Analisis Kebutuhan')).toBeVisible();
    await expect(page.getByLabel('Judul Analisis')).toBeVisible();
  });
});
