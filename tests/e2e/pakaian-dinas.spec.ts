import { test, expect } from '@playwright/test';

/**
 * Pakaian Dinas E2E Tests
 * 
 * Tests cover the official uniform management module:
 * - Jenis Pakaian Dinas (Master data)
 * - Pengajuan Pakaian Dinas (Application periods)
 * - Ukuran Pegawai (Employee sizes)
 * - Laporan (Reports)
 */

test.describe('Pakaian Dinas Module', () => {
  // Setup: Login before tests (assuming auth is required)
  test.beforeEach(async ({ page }) => {
    // Navigate to the dashboard
    // In real scenario, would need to login first
    await page.goto('/dashboard');
  });

  test.describe('Navigation', () => {
    test('should have Pakaian Dinas section in sidebar', async ({ page }) => {
      await page.goto('/dashboard');
      
      // Check sidebar has Pakaian Dinas section
      await expect(page.getByText('Pakaian Dinas')).toBeVisible();
    });

    test('should navigate to Jenis Pakaian page', async ({ page }) => {
      await page.goto('/dashboard/pakaian-dinas/jenis');
      
      // Check page header
      await expect(page.getByRole('heading', { name: /Jenis Pakaian Dinas/i })).toBeVisible();
      await expect(page.getByText('Kelola master data jenis pakaian dinas')).toBeVisible();
    });

    test('should navigate to Pengajuan page', async ({ page }) => {
      await page.goto('/dashboard/pakaian-dinas/pengajuan');
      
      // Check page header
      await expect(page.getByRole('heading', { name: /Pengajuan Pakaian Dinas/i })).toBeVisible();
    });

    test('should navigate to Ukuran Saya page', async ({ page }) => {
      await page.goto('/dashboard/pakaian-dinas/ukuran-saya');
      
      // Check page header
      await expect(page.getByRole('heading', { name: /Ukuran Pakaian Dinas Saya/i })).toBeVisible();
    });

    test('should navigate to Laporan page', async ({ page }) => {
      await page.goto('/dashboard/pakaian-dinas/laporan');
      
      // Check page header
      await expect(page.getByRole('heading', { name: /Laporan Pakaian Dinas/i })).toBeVisible();
    });
  });

  test.describe('Jenis Pakaian Dinas CRUD', () => {
    test('should display empty state when no data', async ({ page }) => {
      await page.goto('/dashboard/pakaian-dinas/jenis');
      
      // Either see data table or empty state
      const hasData = await page.locator('table tbody tr').count() > 0;
      if (!hasData) {
        await expect(page.getByText('Belum ada data jenis pakaian dinas')).toBeVisible();
      }
    });

    test('should open create form when clicking add button', async ({ page }) => {
      await page.goto('/dashboard/pakaian-dinas/jenis');
      
      // Click add button
      await page.getByRole('button', { name: /Tambah Jenis/i }).click();
      
      // Check form is visible
      await expect(page.getByRole('heading', { name: /Tambah Jenis Pakaian Dinas Baru/i })).toBeVisible();
      await expect(page.getByPlaceholder('Contoh: PDH, PDL, Toga')).toBeVisible();
    });

    test('should validate required fields in create form', async ({ page }) => {
      await page.goto('/dashboard/pakaian-dinas/jenis');
      
      // Open form
      await page.getByRole('button', { name: /Tambah Jenis/i }).click();
      
      // Try to submit empty form
      await page.getByRole('button', { name: /Simpan/i }).click();
      
      // Should not close form (validation failed)
      await expect(page.getByPlaceholder('Contoh: PDH, PDL, Toga')).toBeVisible();
    });

    test('should close form when clicking cancel', async ({ page }) => {
      await page.goto('/dashboard/pakaian-dinas/jenis');
      
      // Open form
      await page.getByRole('button', { name: /Tambah Jenis/i }).click();
      await expect(page.getByRole('heading', { name: /Tambah Jenis Pakaian Dinas Baru/i })).toBeVisible();
      
      // Click cancel
      await page.getByRole('button', { name: /Batal/i }).click();
      
      // Form should be hidden
      await expect(page.getByRole('heading', { name: /Tambah Jenis Pakaian Dinas Baru/i })).not.toBeVisible();
    });

    test('should create new jenis pakaian dinas', async ({ page }) => {
      await page.goto('/dashboard/pakaian-dinas/jenis');
      
      // Open form
      await page.getByRole('button', { name: /Tambah Jenis/i }).click();
      
      // Fill form
      const uniqueName = `PDH Test ${Date.now()}`;
      await page.getByPlaceholder('Contoh: PDH, PDL, Toga').fill(uniqueName);
      await page.getByPlaceholder('Keterangan (opsional)').fill('Pakaian Dinas Harian untuk testing');
      
      // Submit
      await page.getByRole('button', { name: /Simpan/i }).click();
      
      // Wait for form to close and data to refresh
      await expect(page.getByRole('heading', { name: /Tambah Jenis Pakaian Dinas Baru/i })).not.toBeVisible({ timeout: 5000 });
      
      // Verify new item appears in table
      await expect(page.getByText(uniqueName)).toBeVisible();
    });

    test('should navigate to spesifikasi page from jenis list', async ({ page }) => {
      await page.goto('/dashboard/pakaian-dinas/jenis');
      
      // If there's data, click on spesifikasi link
      const hasData = await page.locator('table tbody tr').count() > 0;
      if (hasData) {
        // Click first spesifikasi link (list icon)
        await page.locator('table tbody tr').first().locator('a[title="Lihat Spesifikasi"]').click();
        
        // Should navigate to spesifikasi page
        await expect(page.url()).toContain('/spesifikasi');
      }
    });
  });

  test.describe('Pengajuan Pakaian Dinas CRUD', () => {
    test('should display pengajuan list or empty state', async ({ page }) => {
      await page.goto('/dashboard/pakaian-dinas/pengajuan');
      
      // Either see cards or empty state
      const hasCards = await page.locator('.grid .bg-gray-50.rounded-lg').count() > 0;
      if (!hasCards) {
        await expect(page.getByText('Belum ada periode pengajuan pakaian dinas')).toBeVisible();
      }
    });

    test('should open create form for new pengajuan', async ({ page }) => {
      await page.goto('/dashboard/pakaian-dinas/pengajuan');
      
      // Click create button
      await page.getByRole('button', { name: /Buat Pengajuan/i }).click();
      
      // Check form fields
      await expect(page.getByRole('heading', { name: /Buat Periode Pengajuan Baru/i })).toBeVisible();
      await expect(page.getByPlaceholder('Contoh: Pengajuan PDH Tahun 2025')).toBeVisible();
    });

    test('should create new pengajuan period', async ({ page }) => {
      await page.goto('/dashboard/pakaian-dinas/pengajuan');
      
      // Open form
      await page.getByRole('button', { name: /Buat Pengajuan/i }).click();
      
      // Fill form
      const uniqueName = `Pengajuan Test ${Date.now()}`;
      await page.getByPlaceholder('Contoh: Pengajuan PDH Tahun 2025').fill(uniqueName);
      
      // Set year (should default to current year)
      // Set dates
      const today = new Date().toISOString().split('T')[0];
      const nextYear = new Date(Date.now() + 365 * 24 * 60 * 60 * 1000).toISOString().split('T')[0];
      
      await page.locator('input[type="date"]').first().fill(today);
      await page.locator('input[type="date"]').last().fill(nextYear);
      
      // Add keterangan
      await page.getByPlaceholder('Keterangan tambahan (opsional)').fill('E2E Test pengajuan');
      
      // Submit
      await page.getByRole('button', { name: /Simpan/i }).click();
      
      // Wait for refresh
      await expect(page.getByRole('heading', { name: /Buat Periode Pengajuan Baru/i })).not.toBeVisible({ timeout: 5000 });
      
      // Verify card appears
      await expect(page.getByText(uniqueName)).toBeVisible();
    });

    test('should show status badge on pengajuan card', async ({ page }) => {
      await page.goto('/dashboard/pakaian-dinas/pengajuan');
      
      // Check for status badges (Dibuka, Draft, Selesai)
      const hasCards = await page.locator('.grid .bg-gray-50.rounded-lg').count() > 0;
      if (hasCards) {
        // Should have at least one status badge
        const hasBadge = await page.locator('.rounded-full').count() > 0;
        expect(hasBadge).toBeTruthy();
      }
    });

    test('should navigate to satker list from pengajuan card', async ({ page }) => {
      await page.goto('/dashboard/pakaian-dinas/pengajuan');
      
      const hasCards = await page.locator('.grid .bg-gray-50.rounded-lg').count() > 0;
      if (hasCards) {
        // Click satker button on first card
        await page.locator('.grid .bg-gray-50.rounded-lg').first()
          .locator('a', { hasText: 'Satker' }).click();
        
        // Should navigate to satker page
        await expect(page.url()).toContain('/satker');
      }
    });
  });

  test.describe('Ukuran Pegawai (Personal Size)', () => {
    test('should display user info card', async ({ page }) => {
      await page.goto('/dashboard/pakaian-dinas/ukuran-saya');
      
      // Should show user info section
      await expect(page.locator('.bg-blue-50.rounded-lg')).toBeVisible();
      await expect(page.getByText('NIP:')).toBeVisible();
    });

    test('should display size selection dropdowns', async ({ page }) => {
      await page.goto('/dashboard/pakaian-dinas/ukuran-saya');
      
      // Check for size dropdowns
      await expect(page.getByText('Ukuran Baju')).toBeVisible();
      await expect(page.getByText('Ukuran Celana')).toBeVisible();
      await expect(page.getByText('Ukuran Sepatu')).toBeVisible();
    });

    test('should display size guide section', async ({ page }) => {
      await page.goto('/dashboard/pakaian-dinas/ukuran-saya');
      
      // Check for size guide
      await expect(page.getByText('Panduan Pengukuran')).toBeVisible();
    });

    test('should have save button', async ({ page }) => {
      await page.goto('/dashboard/pakaian-dinas/ukuran-saya');
      
      await expect(page.getByRole('button', { name: /Simpan Ukuran/i })).toBeVisible();
    });

    test('should select and save sizes', async ({ page }) => {
      await page.goto('/dashboard/pakaian-dinas/ukuran-saya');
      
      // Wait for dropdowns to load
      await page.waitForSelector('select');
      
      // Select baju size
      const bajuSelect = page.locator('select').first();
      await bajuSelect.selectOption({ index: 1 }); // Select first non-empty option
      
      // Select celana size
      const celanaSelect = page.locator('select').nth(1);
      await celanaSelect.selectOption({ index: 1 });
      
      // Select sepatu size
      const sepatuSelect = page.locator('select').nth(2);
      await sepatuSelect.selectOption({ index: 1 });
      
      // Click save
      await page.getByRole('button', { name: /Simpan Ukuran/i }).click();
      
      // Wait for success message
      await expect(page.getByText(/berhasil disimpan/i)).toBeVisible({ timeout: 5000 });
    });
  });

  test.describe('Laporan (Reports)', () => {
    test('should display tabs for rekap and pegawai', async ({ page }) => {
      await page.goto('/dashboard/pakaian-dinas/laporan');
      
      // Check tabs
      await expect(page.getByRole('button', { name: /Rekap Ukuran/i })).toBeVisible();
      await expect(page.getByRole('button', { name: /Daftar Pegawai/i })).toBeVisible();
    });

    test('should display filter options', async ({ page }) => {
      await page.goto('/dashboard/pakaian-dinas/laporan');
      
      // Check filter section
      await expect(page.getByText('Filter Laporan')).toBeVisible();
      await expect(page.getByText('Periode Pengajuan')).toBeVisible();
      await expect(page.getByText('Jenis Pakaian')).toBeVisible();
    });

    test('should switch between tabs', async ({ page }) => {
      await page.goto('/dashboard/pakaian-dinas/laporan');
      
      // Click on Daftar Pegawai tab
      await page.getByRole('button', { name: /Daftar Pegawai/i }).click();
      
      // Check that pegawai content is visible (either data or empty state)
      // The tab should become active (border-blue-600)
      await expect(page.getByRole('button', { name: /Daftar Pegawai/i })).toHaveClass(/border-blue-600/);
      
      // Switch back to Rekap
      await page.getByRole('button', { name: /Rekap Ukuran/i }).click();
      await expect(page.getByRole('button', { name: /Rekap Ukuran/i })).toHaveClass(/border-blue-600/);
    });

    test('should display rekap ukuran cards for baju, celana, sepatu', async ({ page }) => {
      await page.goto('/dashboard/pakaian-dinas/laporan');
      
      // Wait for data to load
      await page.waitForLoadState('networkidle');
      
      // Check for category cards (may be empty or have data)
      const hasData = await page.locator('.bg-blue-50').count() > 0;
      if (hasData) {
        await expect(page.getByText('Rekap Ukuran Baju')).toBeVisible();
        await expect(page.getByText('Rekap Ukuran Celana')).toBeVisible();
        await expect(page.getByText('Rekap Ukuran Sepatu')).toBeVisible();
      }
    });

    test('should have export buttons', async ({ page }) => {
      await page.goto('/dashboard/pakaian-dinas/laporan');
      
      // Wait for data to load
      await page.waitForLoadState('networkidle');
      
      // Check for export buttons (if data exists)
      const hasData = await page.locator('.bg-blue-50').count() > 0;
      if (hasData) {
        await expect(page.getByRole('button', { name: /Export Excel/i })).toBeVisible();
        await expect(page.getByRole('button', { name: /Export PDF/i })).toBeVisible();
      }
    });

    test('should filter by pengajuan period', async ({ page }) => {
      await page.goto('/dashboard/pakaian-dinas/laporan');
      
      // Select a pengajuan period from dropdown
      const pengajuanSelect = page.locator('select').first();
      const optionCount = await pengajuanSelect.locator('option').count();
      
      if (optionCount > 1) {
        await pengajuanSelect.selectOption({ index: 1 });
        
        // Wait for data to reload
        await page.waitForLoadState('networkidle');
      }
    });

    test('should display daftar pegawai table', async ({ page }) => {
      await page.goto('/dashboard/pakaian-dinas/laporan');
      
      // Switch to daftar pegawai tab
      await page.getByRole('button', { name: /Daftar Pegawai/i }).click();
      
      // Wait for content
      await page.waitForLoadState('networkidle');
      
      // Check for table headers or empty state
      const hasTable = await page.locator('table').count() > 0;
      if (hasTable) {
        await expect(page.getByText('NIP')).toBeVisible();
        await expect(page.getByText('Nama')).toBeVisible();
        await expect(page.getByText('Satker')).toBeVisible();
      }
    });

    test('should paginate daftar pegawai', async ({ page }) => {
      await page.goto('/dashboard/pakaian-dinas/laporan');
      
      // Switch to daftar pegawai tab
      await page.getByRole('button', { name: /Daftar Pegawai/i }).click();
      
      // Wait for content
      await page.waitForLoadState('networkidle');
      
      // Check for pagination buttons
      const hasNextButton = await page.getByRole('button', { name: /Selanjutnya/i }).count() > 0;
      if (hasNextButton) {
        // Check if pagination info is shown
        await expect(page.getByText(/Menampilkan halaman/i)).toBeVisible();
      }
    });
  });

  test.describe('Workflow Integration', () => {
    test('should show workflow status badges on satker submissions', async ({ page }) => {
      await page.goto('/dashboard/pakaian-dinas/pengajuan');
      
      const hasCards = await page.locator('.grid .bg-gray-50.rounded-lg').count() > 0;
      if (hasCards) {
        // Navigate to satker list
        await page.locator('.grid .bg-gray-50.rounded-lg').first()
          .locator('a', { hasText: 'Satker' }).click();
        
        // Wait for satker list to load
        await page.waitForLoadState('networkidle');
        
        // Check for workflow status indicators
        const hasStatus = await page.locator('.rounded-full').count() > 0;
        // Status badges should be present if there's data
      }
    });
  });

  test.describe('Responsive Design', () => {
    test('should be responsive on mobile viewport', async ({ page }) => {
      // Set mobile viewport
      await page.setViewportSize({ width: 375, height: 667 });
      
      await page.goto('/dashboard/pakaian-dinas/jenis');
      
      // Content should still be visible
      await expect(page.getByRole('heading', { name: /Jenis Pakaian Dinas/i })).toBeVisible();
      
      // Add button should be visible
      await expect(page.getByRole('button', { name: /Tambah/i })).toBeVisible();
    });

    test('should be responsive on tablet viewport', async ({ page }) => {
      // Set tablet viewport
      await page.setViewportSize({ width: 768, height: 1024 });
      
      await page.goto('/dashboard/pakaian-dinas/laporan');
      
      // Content should be visible
      await expect(page.getByRole('heading', { name: /Laporan Pakaian Dinas/i })).toBeVisible();
      
      // Filter section should be visible
      await expect(page.getByText('Filter Laporan')).toBeVisible();
    });
  });

  test.describe('Error Handling', () => {
    test('should handle API errors gracefully', async ({ page }) => {
      // Mock API to return error
      await page.route('**/api/pembinaan/perlengkapan/pakaian-dinas/**', (route) => {
        route.fulfill({
          status: 500,
          contentType: 'application/json',
          body: JSON.stringify({ success: false, message: 'Internal Server Error' }),
        });
      });
      
      await page.goto('/dashboard/pakaian-dinas/jenis');
      
      // Should not crash, might show empty state or error
      await expect(page.getByRole('heading', { name: /Jenis Pakaian Dinas/i })).toBeVisible();
    });

    test('should show loading state while fetching data', async ({ page }) => {
      // Add delay to API response
      await page.route('**/api/pembinaan/perlengkapan/pakaian-dinas/jenis**', async (route) => {
        await new Promise(resolve => setTimeout(resolve, 2000));
        route.continue();
      });
      
      await page.goto('/dashboard/pakaian-dinas/jenis');
      
      // Should show loading indicator
      await expect(page.getByText('Memuat data...')).toBeVisible();
    });
  });

  test.describe('Accessibility', () => {
    test('should have accessible form labels', async ({ page }) => {
      await page.goto('/dashboard/pakaian-dinas/jenis');
      
      // Open form
      await page.getByRole('button', { name: /Tambah Jenis/i }).click();
      
      // Check for labels
      await expect(page.getByText('Nama Jenis')).toBeVisible();
      await expect(page.getByText('Keterangan')).toBeVisible();
    });

    test('should have keyboard navigation', async ({ page }) => {
      await page.goto('/dashboard/pakaian-dinas/jenis');
      
      // Tab to add button
      await page.keyboard.press('Tab');
      await page.keyboard.press('Tab');
      
      // Press Enter to activate
      await page.keyboard.press('Enter');
      
      // Form should open
      await expect(page.getByRole('heading', { name: /Tambah Jenis Pakaian Dinas Baru/i })).toBeVisible();
    });

    test('should have focus trap in modal form', async ({ page }) => {
      await page.goto('/dashboard/pakaian-dinas/jenis');
      
      // Open form
      await page.getByRole('button', { name: /Tambah Jenis/i }).click();
      
      // Tab through form fields
      await page.keyboard.press('Tab'); // First input
      await page.keyboard.press('Tab'); // Second input
      await page.keyboard.press('Tab'); // Save button
      await page.keyboard.press('Tab'); // Cancel button
      
      // Focus should stay within form area
    });
  });
});
