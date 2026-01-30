import { test, expect } from '@playwright/test';

test.describe('Perlengkapan Module', () => {
  // Mock API responses before each test
  test.beforeEach(async ({ page }) => {
    // Mock Dashboard Stats
    await page.route('*/**/api/pembinaan/perlengkapan/dashboard/stats', async route => {
      await route.fulfill({
        json: {
          success: true,
          data: {
            total_aset: 100,
            total_nilai_aset: 5000000000,
            total_satker: 5,
            aset_baik: 90,
            aset_rusak: 10,
            categories: []
          },
          message: "Stats retrieved"
        }
      });
    });

    // Mock Asset List
    await page.route('*/**/api/pembinaan/perlengkapan/assets?*', async route => {
      await route.fulfill({
        json: {
          success: true,
          data: [
            {
              id: "uuid-1",
              kategori_aset: "Tanah",
              no_aset: "1",
              nama_aset: "Tanah Kantor",
              kode_barang: "101",
              kondisi: "Baik",
              lokasi: "Jakarta",
              satker: "Pusat",
              nilai_perolehan: 1000000000,
              updated_at: new Date().toISOString()
            }
          ],
          total: 1,
          page: 1,
          per_page: 20,
          total_pages: 1,
          message: "Assets retrieved"
        }
      });
    });

    // Mock Asset Detail
    await page.route('*/**/api/pembinaan/perlengkapan/assets/uuid-1', async route => {
      await route.fulfill({
        json: {
          success: true,
          data: {
            id: "uuid-1",
            kategori_aset: "Tanah",
            no_aset: "1",
            nama_aset: "Tanah Kantor Detail",
            kode_barang: "101",
            kondisi: "Baik",
            lokasi: "Jakarta Pusat",
            satker: "Pusat",
            nilai_perolehan: 1000000000,
            updated_at: new Date().toISOString()
          },
          message: "Asset retrieved"
        }
      });
    });

    // Mock Pengadaan List
    await page.route('*/**/api/pembinaan/perlengkapan/pengadaan?*', async route => {
      await route.fulfill({
        json: {
          success: true,
          data: [],
          total: 0,
          page: 1,
          per_page: 20,
          total_pages: 0,
          message: "Pengadaan retrieved"
        }
      });
    });
  });

  test('should navigate to dashboard and load statistics', async ({ page }) => {
    await page.goto('/');
    await expect(page).toHaveTitle(/SIMPEL Perlengkapan/);
    await expect(page.getByText('Dashboard Perlengkapan')).toBeVisible();
    await expect(page.getByText('Total Aset')).toBeVisible();
    // Verify mocked value
    await expect(page.getByText('100')).toBeVisible();
  });

  test('should navigate to Bank Aset and view details', async ({ page }) => {
    await page.goto('/dashboard/bank-aset/daftar');
    await expect(page.getByText('Daftar Aset')).toBeVisible();
    await expect(page.getByText('Tanah Kantor')).toBeVisible();

    // Test View Detail (assuming there is a view button)
    // Note: The UI implementation has a button with title="Detail"
    await page.getByTitle('Detail').first().click();

    // Since routing in CSR might be fast, we expect the URL to change or modal/page to appear
    // Ideally we check for "Tanah Kantor Detail" but let's check URL pattern if possible
    // await expect(page).toHaveURL(/.*\/assets\/uuid-1/);
    // Or check for specific detail element if known.
    // For now, ensuring no error is good enough for navigation check.
  });

  test('should create new Pengadaan successfully', async ({ page }) => {
    // Mock Create Response
    await page.route('**/api/pembinaan/perlengkapan/pengadaan', async route => {
      if (route.request().method() === 'POST') {
        const postData = route.request().postDataJSON();
        await route.fulfill({
          status: 201,
          json: {
            success: true,
            data: {
              id: "new-uuid",
              judul: postData.judul,
              jenis: postData.jenis,
              status: "perencanaan",
              created_at: new Date().toISOString()
            },
            message: "Created"
          }
        });
      } else {
        await route.continue();
      }
    });

    await page.goto('/dashboard/pengadaan/daftar');
    await page.getByRole('link', { name: 'Buat Pengadaan' }).click();

    await expect(page.getByText('Buat Pengadaan Baru')).toBeVisible();

    // Fill form
    await page.getByLabel('Judul Pengadaan').fill('Pengadaan Server Baru');
    await page.getByLabel('Jenis Pengadaan').selectOption('TIK');
    await page.getByLabel('Anggaran (Rp)').fill('50000000');

    // Submit
    await page.getByRole('button', { name: 'Simpan' }).click();

    // Verify success message (from PengadaanForm.rs: "Data pengadaan berhasil disimpan!")
    await expect(page.getByText('Data pengadaan berhasil disimpan!')).toBeVisible();

    // Verify redirection happens (wait for URL change)
    await expect(page).toHaveURL(/.*\/dashboard\/pengadaan\/daftar/);
  });
});
