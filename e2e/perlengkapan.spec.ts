import { test, expect } from '@playwright/test';

test.describe('Perlengkapan Microfrontend Screenshots', () => {

  test.beforeEach(async ({ page }) => {
    // Inject auth token
    await page.addInitScript(() => {
        const session = {
            id: "user-123",
            username: "admin_perlengkapan",
            role: "Admin",
            name: "Admin Perlengkapan",
            email: "admin@kejaksaan.go.id",
            division: "Biro Perlengkapan",
            captcha_validated: true,
            mfa_enabled: false,
            mfa_setup_required: false,
            permissions: ["*"],
            created_at: new Date().toISOString(),
            access_token: "mock_token_jwt_string",
            expires_at: Math.floor(Date.now() / 1000) + 3600,
            // Fields required by Serde deserialization (Option types must be explicit null if no default)
            avatar: null,
            refresh_token: null
        };
        localStorage.setItem('user_session', JSON.stringify(session));
        localStorage.setItem('auth_token', "mock_token_jwt_string");
    });

    // Mock Dashboard Stats
    await page.route('**/api/pembinaan/perlengkapan/dashboard/stats', async route => {
        const json = {
            success: true,
            data: {
                total_aset: 1250,
                total_nilai_aset: 15000000000,
                total_satker: 34,
                aset_baik: 1100,
                aset_rusak: 150,
                categories: [
                    { category: "Elektronik", count: 500, value: 500000000 },
                    { category: "Kendaraan", count: 50, value: 10000000000 }
                ]
            },
            message: "Success"
        };
        await route.fulfill({ json });
    });

    // Mock Assets List
    await page.route('**/api/pembinaan/perlengkapan/assets*', async route => {
         const json = {
            success: true,
            data: Array.from({ length: 5 }).map((_, i) => ({
                id: `aset-${i}`,
                kategori_aset: "Elektronik",
                no_aset: `AST-2024-${i}`,
                nama_aset: `Laptop Dinas Dell Latitude ${i}`,
                kode_barang: "3050104001",
                merk: "Dell",
                tipe: "Latitude 5420",
                kondisi: i % 3 === 0 ? "Rusak Ringan" : "Baik",
                lokasi: "Gedung Utama Lt. 2",
                satker: "Kejaksaan Agung",
                nilai_perolehan: 15000000,
                tgl_perolehan: "2023-01-15",
                updated_at: new Date().toISOString()
            })),
            total: 50,
            page: 1,
            per_page: 20,
            total_pages: 3,
            message: "Success"
        };
        await route.fulfill({ json });
    });

    // Mock Analisis List
    await page.route('**/api/pembinaan/perlengkapan/analisis*', async route => {
        const json = {
            success: true,
            data: Array.from({ length: 5 }).map((_, i) => ({
                id: `analisis-${i}`,
                judul: `Pengadaan Laptop Baru ${i}`,
                kategori: "TI",
                deskripsi: "Kebutuhan laptop untuk staff baru",
                prioritas: "Tinggi",
                status: "Draft",
                estimasi_biaya: 300000000,
                justifikasi: "Mendukung kinerja pegawai",
                created_at: new Date().toISOString(),
                updated_at: new Date().toISOString()
            })),
            total: 5,
            page: 1,
            per_page: 20,
            total_pages: 1,
            message: "Success"
        };
        await route.fulfill({ json });
    });

    // Mock Kebutuhan BMN Dashboard
    await page.route('**/api/pembinaan/perlengkapan/kebutuhan-bmn/dashboard', async route => {
         const json = {
            success: true,
            data: {
                total_pengajuan: 15,
                pengajuan_draft: 5,
                pengajuan_in_progress: 8,
                pengajuan_completed: 2,
                total_satker_terlibat: 10,
                total_barang_diminta: 100,
                total_barang_disetujui: 80,
                by_tahun: [{ tahun: 2024, total: 15 }],
                by_status: [{ status_kode: 2000, status_nama: "Draft", total: 5 }]
            },
            message: "Success"
        };
        await route.fulfill({ json });
    });

    // Mock Pengadaan List
    await page.route('**/api/pembinaan/perlengkapan/pengadaan*', async route => {
        const json = {
            success: true,
            data: Array.from({ length: 5 }).map((_, i) => ({
                id: `pengadaan-${i}`,
                judul: `Pengadaan Server ${i}`,
                deskripsi: "Server Data Center",
                jenis: "Barang",
                status: "Berjalan",
                anggaran: 500000000,
                target_selesai: "2024-12-31",
                pic_user_id: "user-1",
                created_at: new Date().toISOString(),
                updated_at: new Date().toISOString()
            })),
            total: 5,
            page: 1,
            per_page: 20,
            total_pages: 1,
            message: "Success"
        };
        await route.fulfill({ json });
    });

    // Mock Pemeliharaan List
    await page.route('**/api/pembinaan/perlengkapan/pemeliharaan*', async route => {
        const json = {
            success: true,
            data: Array.from({ length: 5 }).map((_, i) => ({
                id: `pemeliharaan-${i}`,
                asset_id: `aset-${i}`,
                jenis_pemeliharaan: "Rutin",
                biaya: 2500000,
                tanggal_mulai: "2024-02-01",
                tanggal_selesai: "2024-02-03",
                pelaksana: "Vendor IT",
                status: "Selesai",
                keterangan: "Pembersihan dan upgrade RAM",
                created_at: new Date().toISOString(),
                updated_at: new Date().toISOString()
            })),
            total: 5,
            page: 1,
            per_page: 20,
            total_pages: 1,
            message: "Success"
        };
        await route.fulfill({ json });
    });

    // Mock Pakaian Dinas Jenis
    await page.route('**/api/pembinaan/perlengkapan/pakaian-dinas/jenis*', async route => {
        const json = {
            success: true,
            data: [
                { id: "jenis-1", nama: "PDH", keterangan: "Pakaian Dinas Harian", created_at: "", updated_at: "" },
                { id: "jenis-2", nama: "PDL", keterangan: "Pakaian Dinas Lapangan", created_at: "", updated_at: "" },
                { id: "jenis-3", nama: "PDU", keterangan: "Pakaian Dinas Upacara", created_at: "", updated_at: "" }
            ],
            total: 3,
            page: 1,
            per_page: 20,
            total_pages: 1,
            message: "Success"
        };
        await route.fulfill({ json });
    });

    // Mock Pakaian Dinas Pengajuan
    await page.route('**/api/pembinaan/perlengkapan/pakaian-dinas/pengajuan*', async route => {
         const json = {
            success: true,
            data: Array.from({ length: 3 }).map((_, i) => ({
                id: `pengajuan-pd-${i}`,
                nama: `Pengajuan PDH Tahun ${2024+i}`,
                tahun: 2024+i,
                is_open: true,
                tgl_open: "2024-01-01",
                tgl_close: "2024-03-01",
                status: "Draft",
                keterangan: "Pengajuan rutin tahunan",
                created_at: new Date().toISOString(),
                updated_at: new Date().toISOString()
            })),
            total: 3,
            page: 1,
            per_page: 20,
            total_pages: 1,
            message: "Success"
        };
        await route.fulfill({ json });
    });
  });

  // Helper for client-side navigation
  async function navigateTo(page, url) {
      // Use full page load navigation to ensure router initializes correctly with the URL
      await page.goto(url);
      // Wait for network idle to ensure assets are loaded
      await page.waitForLoadState('networkidle');
      // Wait for fonts to load
      await page.evaluate(() => document.fonts.ready);
      // Additional wait for WASM hydration and routing
      await page.waitForTimeout(2000);
  }

  test('Dashboard Home', async ({ page }) => {
    await navigateTo(page, '/dashboard');
    await expect(page).toHaveURL(/.*dashboard.*/);
    await expect(page.getByRole('heading', { name: 'Dashboard Perlengkapan' })).toBeVisible({ timeout: 10000 });
    await expect(page.getByText('Total Aset')).toBeVisible();
    await page.screenshot({ path: 'screenshots/dashboard_home.png', fullPage: true });
  });

  test('Bank Aset List', async ({ page }) => {
    await navigateTo(page, '/dashboard/bank-aset/daftar');
    await expect(page.getByRole('heading', { name: 'Daftar Aset' })).toBeVisible({ timeout: 10000 });
    await expect(page.getByRole('cell', { name: 'No Aset' })).toBeVisible();
    await page.screenshot({ path: 'screenshots/bank_aset_list.png', fullPage: true });
  });

  test('Analisis List', async ({ page }) => {
    await navigateTo(page, '/dashboard/analisis/daftar');
    await expect(page.getByRole('heading', { name: 'Analisis Kebutuhan' })).toBeVisible({ timeout: 10000 });
    await expect(page.getByText('Buat Analisis Baru')).toBeVisible();
    await page.screenshot({ path: 'screenshots/analisis_list.png', fullPage: true });
  });

  test('Kebutuhan BMN Dashboard', async ({ page }) => {
    await navigateTo(page, '/dashboard/kebutuhan-bmn/dashboard');
    await expect(page.getByRole('heading', { name: 'Dashboard Analisis Kebutuhan BMN' })).toBeVisible({ timeout: 10000 });
    await expect(page.getByText('Total Pengajuan')).toBeVisible();
    await page.screenshot({ path: 'screenshots/kebutuhan_bmn_dashboard.png', fullPage: true });
  });

  test('Pengadaan List', async ({ page }) => {
    await navigateTo(page, '/dashboard/pengadaan/daftar');
    await expect(page.getByRole('heading', { name: 'Daftar Pengadaan' })).toBeVisible({ timeout: 10000 });
    await expect(page.getByText('Buat Pengadaan')).toBeVisible();
    await page.screenshot({ path: 'screenshots/pengadaan_list.png', fullPage: true });
  });

  test('Pemeliharaan List', async ({ page }) => {
    await navigateTo(page, '/dashboard/pemeliharaan/daftar');
    await expect(page.getByRole('heading', { name: 'Daftar Pemeliharaan Aset' })).toBeVisible({ timeout: 10000 });
    await expect(page.getByText('Catat Pemeliharaan')).toBeVisible();
    await page.screenshot({ path: 'screenshots/pemeliharaan_list.png', fullPage: true });
  });

  test('Pakaian Dinas Jenis', async ({ page }) => {
    await navigateTo(page, '/dashboard/pakaian-dinas/jenis');
    await expect(page.getByRole('heading', { name: 'Jenis Pakaian Dinas' })).toBeVisible({ timeout: 10000 });
    await expect(page.getByText('Tambah Jenis')).toBeVisible();
    await page.screenshot({ path: 'screenshots/pakaian_dinas_jenis.png', fullPage: true });
  });

  test('Pakaian Dinas Pengajuan', async ({ page }) => {
    await navigateTo(page, '/dashboard/pakaian-dinas/pengajuan');
    await expect(page.getByRole('heading', { name: 'Pengajuan Pakaian Dinas' })).toBeVisible({ timeout: 10000 });
    await expect(page.getByText('Buat Pengajuan')).toBeVisible();
    await page.screenshot({ path: 'screenshots/pakaian_dinas_pengajuan.png', fullPage: true });
  });

});
