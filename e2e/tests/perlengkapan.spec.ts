import { test, expect } from '@playwright/test';

test.describe('Perlengkapan Module', () => {
  test.beforeEach(async ({ page }) => {
    // Navigate to dashboard root
    await page.goto('/');

    // Check if login is required and perform login if necessary
    const loginButton = page.getByRole('button', { name: 'Masuk' });
    if (await loginButton.isVisible()) {
        await page.getByLabel('NIP / Email').fill('admin@kejaksaan.go.id');
        await page.getByLabel('Kata Sandi').fill('password');
        await loginButton.click();
        await expect(page).toHaveURL(/\/dashboard/);
    }
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

  test('should create new pemakaian', async ({ page }) => {
    await page.goto('/dashboard/pengelolaan/pemakaian/daftar');

    // Check heading
    await expect(page.getByRole('heading', { name: 'Daftar Pemakaian BMN' })).toBeVisible();

    // Click "Catat Pemakaian"
    await page.getByRole('link', { name: 'Catat Pemakaian' }).click();
    await expect(page).toHaveURL('/dashboard/pengelolaan/pemakaian/baru');

    // Fill Form
    await page.getByLabel('Nama Peminjam').fill('E2E User');
    await page.getByLabel('Asset ID (UUID)').fill('550e8400-e29b-41d4-a716-446655440000');
    await page.getByLabel('Tanggal Mulai').fill('2024-01-01');
    await page.getByLabel('Keperluan').fill('E2E Testing Usage');

    // Submit
    await page.getByRole('button', { name: 'Simpan' }).click();

    // Expect redirect to list
    await expect(page).toHaveURL('/dashboard/pengelolaan/pemakaian/daftar');

    // Expect new item to be in the list
    await expect(page.getByText('E2E User')).toBeVisible();
    await expect(page.getByText('E2E Testing Usage')).toBeVisible();
  });

  test('should create new hibah', async ({ page }) => {
    await page.goto('/dashboard/pengelolaan/hibah/daftar');

    // Check heading
    await expect(page.getByRole('heading', { name: 'Daftar Hibah BMN' })).toBeVisible();

    // Click "Catat Hibah"
    await page.getByRole('link', { name: 'Catat Hibah' }).click();
    await expect(page).toHaveURL('/dashboard/pengelolaan/hibah/baru');

    // Fill Form
    await page.getByLabel('Asset ID (UUID)').fill('550e8400-e29b-41d4-a716-446655440000');
    await page.getByLabel('Pemberi Hibah').fill('E2E Donor');
    await page.getByLabel('Penerima Hibah').fill('E2E Receiver');
    await page.getByLabel('Tanggal Hibah').fill('2024-02-01');
    await page.getByLabel('Keterangan').fill('E2E Testing Hibah');

    // Submit
    await page.getByRole('button', { name: 'Simpan' }).click();

    // Expect redirect to list
    await expect(page).toHaveURL('/dashboard/pengelolaan/hibah/daftar');

    // Expect new item to be in the list
    await expect(page.getByText('E2E Donor')).toBeVisible();
    await expect(page.getByText('E2E Receiver')).toBeVisible();
  });

  test('should create new mutasi', async ({ page }) => {
    await page.goto('/dashboard/pengelolaan/mutasi/daftar');

    // Check heading
    await expect(page.getByRole('heading', { name: 'Daftar Mutasi BMN' })).toBeVisible();

    // Click "Catat Mutasi"
    await page.getByRole('link', { name: 'Catat Mutasi' }).click();
    await expect(page).toHaveURL('/dashboard/pengelolaan/mutasi/baru');

    // Fill Form
    await page.getByLabel('Asset ID (UUID)').fill('550e8400-e29b-41d4-a716-446655440000');
    await page.getByLabel('Satker Asal').fill('Satker A');
    await page.getByLabel('Satker Tujuan').fill('Satker B');
    await page.getByLabel('Penanggung Jawab').fill('E2E Officer');
    await page.getByLabel('Tanggal Mutasi').fill('2024-03-01');
    await page.getByLabel('Keterangan').fill('E2E Testing Mutasi');

    // Submit
    await page.getByRole('button', { name: 'Simpan' }).click();

    // Expect redirect to list
    await expect(page).toHaveURL('/dashboard/pengelolaan/mutasi/daftar');

    // Expect new item to be in the list
    await expect(page.getByText('Satker A')).toBeVisible();
    await expect(page.getByText('Satker B')).toBeVisible();
  });

  test('should create new penghapusan', async ({ page }) => {
    await page.goto('/dashboard/pengelolaan/penghapusan/daftar');

    // Check heading
    await expect(page.getByRole('heading', { name: 'Daftar Penghapusan BMN' })).toBeVisible();

    // Click "Usul Penghapusan"
    await page.getByRole('link', { name: 'Usul Penghapusan' }).click();
    await expect(page).toHaveURL('/dashboard/pengelolaan/penghapusan/baru');

    // Fill Form
    await page.getByLabel('Asset ID (UUID)').fill('550e8400-e29b-41d4-a716-446655440000');
    await page.getByLabel('Tanggal Penghapusan').fill('2024-04-01');
    await page.getByLabel('Metode Penghapusan').selectOption('Musnah');
    await page.getByLabel('Alasan').fill('E2E Testing Penghapusan');
    await page.getByLabel('Nilai Residu (Rp)').fill('0');

    // Submit
    await page.getByRole('button', { name: 'Simpan' }).click();

    // Expect redirect to list
    await expect(page).toHaveURL('/dashboard/pengelolaan/penghapusan/daftar');

    // Expect new item to be in the list
    await expect(page.getByText('Musnah')).toBeVisible();
    await expect(page.getByText('E2E Testing Penghapusan')).toBeVisible();
  });

  test('should create new pengalihan', async ({ page }) => {
    // Mock assets for the select dropdown
    await page.route('**/api/pembinaan/perlengkapan/assets*', async route => {
      const json = {
          success: true,
          data: [{
              id: '550e8400-e29b-41d4-a716-446655440000',
              kategori_aset: 'Tanah',
              no_aset: '1',
              nama_aset: 'Mock Asset',
              updated_at: new Date().toISOString()
          }],
          total: 1,
          page: 1,
          per_page: 100,
          total_pages: 1,
          message: 'Mock assets'
      };
      await route.fulfill({ json });
    });

    await page.goto('/dashboard/pengelolaan/pengalihan/daftar');

    // Check heading
    await expect(page.getByRole('heading', { name: 'Daftar Pengalihan Aset' })).toBeVisible();

    // Click "Tambah Pengalihan"
    await page.getByRole('link', { name: 'Tambah Pengalihan' }).click();
    await expect(page).toHaveURL('/dashboard/pengelolaan/pengalihan/baru');

    // Fill Form
    // Wait for options to populate (implicit by selectOption usually)
    await page.getByLabel('Pilih Aset').selectOption({ label: '1 - Mock Asset' });

    await page.getByLabel('Pihak Lama').fill('E2E Old Party');
    await page.getByLabel('Pihak Baru').fill('E2E New Party');
    await page.getByLabel('Tanggal Pengalihan').fill('2024-05-01');

    // Submit
    await page.getByRole('button', { name: 'Simpan' }).click();

    // Expect redirect to list
    await expect(page).toHaveURL('/dashboard/pengelolaan/pengalihan/daftar');

    // Expect new item to be in the list
    await expect(page.getByText('E2E Old Party')).toBeVisible();
    await expect(page.getByText('E2E New Party')).toBeVisible();
  });

  test('should create new pemeliharaan', async ({ page }) => {
    // Mock assets for the select dropdown
    await page.route('**/api/pembinaan/perlengkapan/assets*', async route => {
      const json = {
          success: true,
          data: [{
              id: '550e8400-e29b-41d4-a716-446655440000',
              kategori_aset: 'Tanah',
              no_aset: '1',
              nama_aset: 'Mock Asset',
              updated_at: new Date().toISOString()
          }],
          total: 1,
          page: 1,
          per_page: 100,
          total_pages: 1,
          message: 'Mock assets'
      };
      await route.fulfill({ json });
    });

    await page.goto('/dashboard/pengelolaan/pemeliharaan/daftar');

    // Check heading
    await expect(page.getByRole('heading', { name: 'Daftar Pemeliharaan Aset' })).toBeVisible();

    // Click "Catat Pemeliharaan"
    await page.getByRole('link', { name: 'Catat Pemeliharaan' }).click();
    await expect(page).toHaveURL('/dashboard/pengelolaan/pemeliharaan/baru');

    // Fill Form
    await page.getByLabel('Pilih Aset').selectOption({ label: '1 - Mock Asset' });

    await page.getByLabel('Jenis Pemeliharaan').selectOption('Rutin');
    await page.getByLabel('Pelaksana').fill('E2E Vendor');
    await page.getByLabel('Tanggal Mulai').fill('2024-06-01');
    await page.getByLabel('Biaya (Rp)').fill('500000');

    // Submit
    await page.getByRole('button', { name: 'Simpan' }).click();

    // Expect redirect to list
    await expect(page).toHaveURL('/dashboard/pengelolaan/pemeliharaan/daftar');

    // Expect new item to be in the list
    await expect(page.getByText('E2E Vendor')).toBeVisible();
    await expect(page.getByText('Rp 500000.00').or(page.getByText('500000'))).toBeVisible();
  });
});
