import { test, expect } from '@playwright/test';

test.describe('Perlengkapan Module', () => {
  test.beforeEach(async ({ page }) => {
    // Assuming we have a way to mock login or we use a seed
    // For now, we assume the app starts at login or we can bypass if using a specific test env
    // Navigate to dashboard root
    await page.goto('/');

    // If login is needed, handle it here (Simplified for this example)
    // await page.getByPlaceholder('Username').fill('admin');
    // await page.getByPlaceholder('Password').fill('password');
    // await page.getByRole('button', { name: 'Login' }).click();
    // await expect(page).toHaveURL('/dashboard');
  });

  test('should display asset list', async ({ page }) => {
    await page.goto('/dashboard/bank-aset/daftar');

    // Check header
    await expect(page.getByRole('heading', { name: 'Daftar Aset' })).toBeVisible();

    // Check if table exists
    await expect(page.locator('table')).toBeVisible();
  });

  test('should create new pengadaan', async ({ page }) => {
    await page.goto('/dashboard/pengadaan/daftar');

    // Check empty state or table
    await expect(page.getByRole('heading', { name: 'Daftar Pengadaan' })).toBeVisible();

    // Click "Buat Pengadaan"
    await page.getByRole('link', { name: 'Buat Pengadaan' }).click();
    await expect(page).toHaveURL('/dashboard/pengadaan/baru');

    // Fill Form
    await page.getByLabel('Judul Pengadaan').fill('E2E Test Procurement');
    await page.getByLabel('Jenis Pengadaan').selectOption('TIK');
    await page.getByLabel('Anggaran (Rp)').fill('15000000');

    // Submit
    await page.getByRole('button', { name: 'Simpan' }).click();

    // Expect redirect to list
    await expect(page).toHaveURL('/dashboard/pengadaan/daftar');

    // Expect new item to be in the list
    await expect(page.getByText('E2E Test Procurement')).toBeVisible();
    await expect(page.getByText('TIK')).toBeVisible();
    await expect(page.getByText('Rp 15000000.00').or(page.getByText('15000000'))).toBeVisible();
  });

  test('should display analysis list', async ({ page }) => {
      await page.goto('/dashboard/analisis/daftar');
      await expect(page.getByRole('heading', { name: 'Analisis Kebutuhan' })).toBeVisible();
  });
});
