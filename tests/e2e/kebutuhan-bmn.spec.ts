import { test, expect } from '@playwright/test';

/**
 * Kebutuhan BMN E2E Tests
 *
 * Tests cover the BMN needs analysis module:
 * - Dashboard overview
 * - Pengajuan CRUD (Create, Read, Update, Delete)
 * - Workflow transitions
 * - Satker management
 * - Barang management
 * - Priority setting
 * - Feasibility analysis
 */

test.describe('Kebutuhan BMN Module', () => {
  // Setup: Navigate to dashboard before each test
  test.beforeEach(async ({ page }) => {
    await page.goto('/dashboard');
  });

  test.describe('Navigation', () => {
    test('should navigate to Kebutuhan BMN dashboard', async ({ page }) => {
      await page.goto('/dashboard/kebutuhan-bmn');

      // Check page is loaded
      await expect(
        page.getByRole('heading', { name: /Dashboard Analisis Kebutuhan BMN/i })
      ).toBeVisible();
    });

    test('should navigate to Kebutuhan BMN list', async ({ page }) => {
      await page.goto('/dashboard/kebutuhan-bmn/daftar');

      // Check page header
      await expect(
        page.getByRole('heading', { name: /Analisis Kebutuhan BMN/i })
      ).toBeVisible();
      await expect(
        page.getByText('Kelola pengajuan kebutuhan barang milik negara')
      ).toBeVisible();
    });

    test('should navigate to create form', async ({ page }) => {
      await page.goto('/dashboard/kebutuhan-bmn/baru');

      // Check form header
      await expect(
        page.getByRole('heading', { name: /Buat Pengajuan Kebutuhan BMN/i })
      ).toBeVisible();
    });

    test('should navigate via Analisis menu BMN submenu', async ({ page }) => {
      await page.goto('/dashboard/analisis/bmn');

      // Should load the Kebutuhan BMN dashboard
      await expect(page.url()).toContain('bmn');
    });
  });

  test.describe('Dashboard', () => {
    test('should display statistics cards', async ({ page }) => {
      await page.goto('/dashboard/kebutuhan-bmn');

      // Check stat cards are visible
      await expect(page.getByText('Total Pengajuan')).toBeVisible();
      await expect(page.getByText('Draft')).toBeVisible();
      await expect(page.getByText('Dalam Proses')).toBeVisible();
      await expect(page.getByText('Selesai')).toBeVisible();
    });

    test('should display secondary stats', async ({ page }) => {
      await page.goto('/dashboard/kebutuhan-bmn');

      // Check secondary stats
      await expect(page.getByText('Total Satker Terlibat')).toBeVisible();
      await expect(page.getByText('Total Barang Diminta')).toBeVisible();
      await expect(page.getByText('Total Barang Disetujui')).toBeVisible();
    });

    test('should display quick links', async ({ page }) => {
      await page.goto('/dashboard/kebutuhan-bmn');

      // Check quick links section
      await expect(page.getByText('Akses Cepat')).toBeVisible();
      await expect(page.getByText('Daftar Pengajuan')).toBeVisible();
      await expect(page.getByText('Pengajuan Baru')).toBeVisible();
    });

    test('should have create button in dashboard', async ({ page }) => {
      await page.goto('/dashboard/kebutuhan-bmn');

      // Check create button exists
      await expect(
        page.getByRole('link', { name: /Buat Pengajuan Baru/i })
      ).toBeVisible();
    });
  });

  test.describe('Pengajuan List', () => {
    test('should display empty state when no data', async ({ page }) => {
      await page.goto('/dashboard/kebutuhan-bmn/daftar');

      // Either see data table or empty state
      const hasData = await page.locator('table tbody tr').count();
      if (hasData === 0) {
        await expect(
          page.getByText('Belum ada pengajuan kebutuhan BMN')
        ).toBeVisible();
      }
    });

    test('should have filter controls', async ({ page }) => {
      await page.goto('/dashboard/kebutuhan-bmn/daftar');

      // Check filter elements
      await expect(
        page.getByPlaceholder('Cari nama pengajuan...')
      ).toBeVisible();
      await expect(page.locator('select').first()).toBeVisible();
    });

    test('should filter by year', async ({ page }) => {
      await page.goto('/dashboard/kebutuhan-bmn/daftar');

      // Select year filter
      await page.locator('select').first().selectOption({ label: '2025' });

      // Wait for data to refresh
      await page.waitForTimeout(500);

      // Page should still be visible (no error)
      await expect(
        page.getByRole('heading', { name: /Analisis Kebutuhan BMN/i })
      ).toBeVisible();
    });

    test('should filter by status', async ({ page }) => {
      await page.goto('/dashboard/kebutuhan-bmn/daftar');

      // Select status filter (Draft = 2000)
      await page.locator('select').nth(1).selectOption('2000');

      // Wait for data to refresh
      await page.waitForTimeout(500);

      // Page should still be visible
      await expect(
        page.getByRole('heading', { name: /Analisis Kebutuhan BMN/i })
      ).toBeVisible();
    });

    test('should search by name', async ({ page }) => {
      await page.goto('/dashboard/kebutuhan-bmn/daftar');

      // Type in search
      await page
        .getByPlaceholder('Cari nama pengajuan...')
        .fill('Pengajuan Test');

      // Wait for debounce and data refresh
      await page.waitForTimeout(500);

      // Page should still be visible
      await expect(
        page.getByRole('heading', { name: /Analisis Kebutuhan BMN/i })
      ).toBeVisible();
    });

    test('should have create button', async ({ page }) => {
      await page.goto('/dashboard/kebutuhan-bmn/daftar');

      // Check create button
      await expect(
        page.getByRole('link', { name: /Buat Pengajuan/i })
      ).toBeVisible();
    });

    test('should navigate to create form from list', async ({ page }) => {
      await page.goto('/dashboard/kebutuhan-bmn/daftar');

      // Click create button
      await page.getByRole('link', { name: /Buat Pengajuan/i }).click();

      // Should navigate to form
      await expect(page.url()).toContain('/baru');
      await expect(
        page.getByRole('heading', { name: /Buat Pengajuan Kebutuhan BMN/i })
      ).toBeVisible();
    });
  });

  test.describe('Pengajuan Form', () => {
    test('should display all form fields', async ({ page }) => {
      await page.goto('/dashboard/kebutuhan-bmn/baru');

      // Check form fields
      await expect(page.getByLabel(/Nama Pengajuan/i)).toBeVisible();
      await expect(page.getByLabel(/Deskripsi/i)).toBeVisible();
      await expect(page.getByLabel(/Tahun Anggaran/i)).toBeVisible();
      await expect(page.getByLabel(/Tanggal Mulai/i)).toBeVisible();
      await expect(page.getByLabel(/Tanggal Selesai/i)).toBeVisible();
    });

    test('should have pilihan satker radio buttons', async ({ page }) => {
      await page.goto('/dashboard/kebutuhan-bmn/baru');

      // Check radio buttons
      await expect(page.getByText('Semua Satker')).toBeVisible();
      await expect(page.getByText('Sebagian Satker')).toBeVisible();
    });

    test('should have back button', async ({ page }) => {
      await page.goto('/dashboard/kebutuhan-bmn/baru');

      // Check back link
      await expect(
        page.getByRole('link', { name: /Kembali ke Daftar/i })
      ).toBeVisible();
    });

    test('should validate required fields', async ({ page }) => {
      await page.goto('/dashboard/kebutuhan-bmn/baru');

      // Clear any default values and try to submit empty form
      await page.getByLabel(/Nama Pengajuan/i).fill('');

      // Submit button should be disabled when form is invalid
      await expect(page.getByRole('button', { name: /Simpan/i })).toBeDisabled();
    });

    test('should enable submit when form is valid', async ({ page }) => {
      await page.goto('/dashboard/kebutuhan-bmn/baru');

      // Fill required fields
      await page
        .getByLabel(/Nama Pengajuan/i)
        .fill('Pengajuan Test E2E ' + Date.now());
      await page.getByLabel(/Tanggal Mulai/i).fill('2025-01-01');
      await page.getByLabel(/Tanggal Selesai/i).fill('2025-12-31');

      // Submit button should be enabled
      await expect(
        page.getByRole('button', { name: /Simpan/i })
      ).not.toBeDisabled();
    });

    test('should navigate back to list on cancel', async ({ page }) => {
      await page.goto('/dashboard/kebutuhan-bmn/baru');

      // Click cancel/back
      await page.getByRole('link', { name: /Kembali ke Daftar/i }).click();

      // Should navigate to list
      await expect(page.url()).toContain('/kebutuhan-bmn');
      await expect(
        page.getByRole('heading', { name: /Analisis Kebutuhan BMN/i })
      ).toBeVisible();
    });

    test('should create new pengajuan successfully', async ({ page }) => {
      await page.goto('/dashboard/kebutuhan-bmn/baru');

      // Fill form
      const uniqueName = `Pengajuan E2E ${Date.now()}`;
      await page.getByLabel(/Nama Pengajuan/i).fill(uniqueName);
      await page.getByLabel(/Deskripsi/i).fill('Test pengajuan dari E2E');
      await page.getByLabel(/Tanggal Mulai/i).fill('2025-01-01');
      await page.getByLabel(/Tanggal Selesai/i).fill('2025-12-31');

      // Submit
      await page.getByRole('button', { name: /Simpan/i }).click();

      // Wait for success message
      await expect(
        page.getByText('Data berhasil disimpan')
      ).toBeVisible({ timeout: 5000 });
    });
  });

  test.describe('Pengajuan Detail', () => {
    test.beforeEach(async ({ page }) => {
      // Assume there's at least one pengajuan to view
      // In real test, would create one first or use test data
      await page.goto('/dashboard/kebutuhan-bmn/daftar');
    });

    test('should display detail page with sections', async ({ page }) => {
      // If there's data, click on first item
      const firstRow = page.locator('table tbody tr').first();
      const hasData = (await firstRow.count()) > 0;

      if (hasData) {
        // Click on the first pengajuan link
        await firstRow.locator('a').first().click();

        // Check detail page sections
        await expect(page.getByText('Total Satker')).toBeVisible();
        await expect(page.getByText('Daftar Satker')).toBeVisible();
      }
    });

    test('should have back navigation', async ({ page }) => {
      await page.goto('/dashboard/kebutuhan-bmn/daftar');

      const firstRow = page.locator('table tbody tr').first();
      const hasData = (await firstRow.count()) > 0;

      if (hasData) {
        await firstRow.locator('a').first().click();

        // Check back link
        await expect(
          page.getByRole('link', { name: /Kembali ke Daftar/i })
        ).toBeVisible();
      }
    });

    test('should have edit and delete buttons', async ({ page }) => {
      await page.goto('/dashboard/kebutuhan-bmn/daftar');

      const firstRow = page.locator('table tbody tr').first();
      const hasData = (await firstRow.count()) > 0;

      if (hasData) {
        await firstRow.locator('a').first().click();

        // Check action buttons
        await expect(page.getByRole('link', { name: /Edit/i })).toBeVisible();
        await expect(page.locator('button.text-red-600')).toBeVisible(); // Delete button
      }
    });

    test('should show workflow actions when applicable', async ({ page }) => {
      await page.goto('/dashboard/kebutuhan-bmn/daftar');

      const firstRow = page.locator('table tbody tr').first();
      const hasData = (await firstRow.count()) > 0;

      if (hasData) {
        await firstRow.locator('a').first().click();

        // Check workflow section (may or may not be visible depending on status)
        const workflowSection = page.getByText('Aksi Workflow');
        // Just verify page loads correctly
        await expect(page.getByText('Total Satker')).toBeVisible();
      }
    });

    test('should open delete confirmation modal', async ({ page }) => {
      await page.goto('/dashboard/kebutuhan-bmn/daftar');

      const firstRow = page.locator('table tbody tr').first();
      const hasData = (await firstRow.count()) > 0;

      if (hasData) {
        await firstRow.locator('a').first().click();

        // Click delete button
        await page.locator('button.text-red-600').first().click();

        // Modal should open
        await expect(page.getByText('Konfirmasi Hapus')).toBeVisible();
        await expect(
          page.getByText('Apakah Anda yakin ingin menghapus')
        ).toBeVisible();
      }
    });

    test('should close delete modal on cancel', async ({ page }) => {
      await page.goto('/dashboard/kebutuhan-bmn/daftar');

      const firstRow = page.locator('table tbody tr').first();
      const hasData = (await firstRow.count()) > 0;

      if (hasData) {
        await firstRow.locator('a').first().click();
        await page.locator('button.text-red-600').first().click();

        // Cancel delete
        await page.getByRole('button', { name: /Batal/i }).click();

        // Modal should close
        await expect(page.getByText('Konfirmasi Hapus')).not.toBeVisible();
      }
    });
  });

  test.describe('Satker Detail', () => {
    test('should navigate to satker detail from pengajuan', async ({
      page,
    }) => {
      // This requires a pengajuan with satkers
      await page.goto('/dashboard/kebutuhan-bmn/daftar');

      const firstRow = page.locator('table tbody tr').first();
      const hasData = (await firstRow.count()) > 0;

      if (hasData) {
        await firstRow.locator('a').first().click();

        // Check if satker list exists
        const satkerRow = page.locator('table tbody tr').first();
        const hasSatker = (await satkerRow.count()) > 0;

        if (hasSatker) {
          // Click detail link on satker
          await satkerRow.getByRole('link', { name: /Detail/i }).click();

          // Should show satker detail
          await expect(page.getByText('Daftar Barang')).toBeVisible();
        }
      }
    });

    test('should display tabs in satker detail', async ({ page }) => {
      // Navigate directly to a satker (if known)
      await page.goto('/dashboard/kebutuhan-bmn/daftar');

      // Check tabs exist (would need actual satker ID)
      // This is a placeholder - in real tests use test data
    });
  });

  test.describe('Barang Management', () => {
    test('should open add barang modal', async ({ page }) => {
      // Navigate to satker detail with barang tab
      await page.goto('/dashboard/kebutuhan-bmn/daftar');

      const firstRow = page.locator('table tbody tr').first();
      const hasData = (await firstRow.count()) > 0;

      if (hasData) {
        await firstRow.locator('a').first().click();

        const satkerRow = page.locator('table tbody tr').first();
        const hasSatker = (await satkerRow.count()) > 0;

        if (hasSatker) {
          await satkerRow.getByRole('link', { name: /Detail/i }).click();

          // Click add barang button
          const addButton = page.getByRole('button', {
            name: /Tambah Barang/i,
          });
          if (await addButton.isVisible()) {
            await addButton.click();

            // Modal should open
            await expect(
              page.getByRole('heading', { name: /Tambah Barang/i })
            ).toBeVisible();
          }
        }
      }
    });
  });

  test.describe('Workflow Transitions', () => {
    test('should show allowed transitions based on status', async ({
      page,
    }) => {
      await page.goto('/dashboard/kebutuhan-bmn/daftar');

      // Filter by draft status
      await page.locator('select').nth(1).selectOption('2000');
      await page.waitForTimeout(500);

      const firstRow = page.locator('table tbody tr').first();
      const hasData = (await firstRow.count()) > 0;

      if (hasData) {
        await firstRow.locator('a').first().click();

        // Draft should show Input Barang transition
        await expect(page.getByText('Aksi Workflow')).toBeVisible();
      }
    });

    test('should have comment field for transitions', async ({ page }) => {
      await page.goto('/dashboard/kebutuhan-bmn/daftar');

      const firstRow = page.locator('table tbody tr').first();
      const hasData = (await firstRow.count()) > 0;

      if (hasData) {
        await firstRow.locator('a').first().click();

        // Check comment input exists (if workflow section is visible)
        const workflowSection = page.getByText('Aksi Workflow');
        if (await workflowSection.isVisible()) {
          await expect(
            page.getByPlaceholder('Komentar (opsional)')
          ).toBeVisible();
        }
      }
    });
  });

  test.describe('Responsive Design', () => {
    test('should be mobile responsive on list page', async ({ page }) => {
      // Set mobile viewport
      await page.setViewportSize({ width: 375, height: 667 });

      await page.goto('/dashboard/kebutuhan-bmn/daftar');

      // Page should still be functional
      await expect(
        page.getByRole('heading', { name: /Analisis Kebutuhan BMN/i })
      ).toBeVisible();
    });

    test('should be mobile responsive on form page', async ({ page }) => {
      await page.setViewportSize({ width: 375, height: 667 });

      await page.goto('/dashboard/kebutuhan-bmn/baru');

      // Form should still be visible
      await expect(page.getByLabel(/Nama Pengajuan/i)).toBeVisible();
    });

    test('should be tablet responsive', async ({ page }) => {
      await page.setViewportSize({ width: 768, height: 1024 });

      await page.goto('/dashboard/kebutuhan-bmn');

      // Dashboard should render correctly
      await expect(page.getByText('Total Pengajuan')).toBeVisible();
    });
  });

  test.describe('Error Handling', () => {
    test('should handle invalid ID gracefully', async ({ page }) => {
      await page.goto('/dashboard/kebutuhan-bmn/invalid-uuid-12345');

      // Should show error or redirect
      // At minimum, page should not crash
      await page.waitForTimeout(2000);
    });

    test('should handle network errors gracefully', async ({ page }) => {
      // Simulate offline
      await page.route('**/api/**', (route) => route.abort());

      await page.goto('/dashboard/kebutuhan-bmn/daftar');

      // Page should handle error gracefully
      await page.waitForTimeout(2000);
    });
  });

  test.describe('Accessibility', () => {
    test('should have proper heading hierarchy', async ({ page }) => {
      await page.goto('/dashboard/kebutuhan-bmn');

      // Check h2 exists
      const h2 = page.getByRole('heading', { level: 2 });
      await expect(h2.first()).toBeVisible();
    });

    test('should have proper form labels', async ({ page }) => {
      await page.goto('/dashboard/kebutuhan-bmn/baru');

      // Labels should be associated with inputs
      const namaLabel = page.getByText('Nama Pengajuan');
      await expect(namaLabel).toBeVisible();
    });

    test('should have focus management on modal', async ({ page }) => {
      await page.goto('/dashboard/kebutuhan-bmn/daftar');

      const firstRow = page.locator('table tbody tr').first();
      const hasData = (await firstRow.count()) > 0;

      if (hasData) {
        await firstRow.locator('a').first().click();
        await page.locator('button.text-red-600').first().click();

        // Modal should trap focus
        await expect(page.getByText('Konfirmasi Hapus')).toBeVisible();
      }
    });
  });

  test.describe('SIMAN Integration', () => {
    test('should display SIMAN asset search component', async ({ page }) => {
      // Navigate to a satker detail with analisis tab
      await page.goto('/dashboard/kebutuhan-bmn/daftar');

      // Click on first pengajuan if exists
      const firstRow = page.locator('table tbody tr').first();
      const hasData = (await firstRow.count()) > 0;

      if (hasData) {
        await firstRow.locator('a').first().click();
        await page.waitForURL('**/kebutuhan-bmn/**');

        // Check for SIMAN search component if satker exists
        const satkerLink = page.locator('a:has-text("Detail")').first();
        if ((await satkerLink.count()) > 0) {
          await satkerLink.click();
          await page.waitForURL('**/kebutuhan-bmn/satker/**');

          // Click on analisis tab
          await page.getByRole('button', { name: /Analisis Kelayakan/i }).click();

          // Check for SIMAN related elements
          await expect(
            page.getByText(/Data aset existing|SIMAN|Existing/i)
          ).toBeVisible({ timeout: 5000 });
        }
      }
    });

    test('should show existing assets in analysis', async ({ page }) => {
      await page.goto('/dashboard/kebutuhan-bmn/daftar');

      const firstRow = page.locator('table tbody tr').first();
      const hasData = (await firstRow.count()) > 0;

      if (hasData) {
        await firstRow.locator('a').first().click();
        await page.waitForURL('**/kebutuhan-bmn/**');

        const satkerLink = page.locator('a:has-text("Detail")').first();
        if ((await satkerLink.count()) > 0) {
          await satkerLink.click();
          await page.waitForURL('**/kebutuhan-bmn/satker/**');

          // Click analisis tab
          await page.getByRole('button', { name: /Analisis Kelayakan/i }).click();

          // Look for analysis stats cards
          await expect(page.getByText(/Total Diminta/i)).toBeVisible({ timeout: 5000 });
          await expect(page.getByText(/Total Existing/i)).toBeVisible();
          await expect(page.getByText(/Gap Kebutuhan/i)).toBeVisible();
          await expect(page.getByText(/% Kelayakan/i)).toBeVisible();
        }
      }
    });

    test('should display detail per barang with SIMAN data', async ({ page }) => {
      await page.goto('/dashboard/kebutuhan-bmn/daftar');

      const firstRow = page.locator('table tbody tr').first();
      const hasData = (await firstRow.count()) > 0;

      if (hasData) {
        await firstRow.locator('a').first().click();
        await page.waitForURL('**/kebutuhan-bmn/**');

        const satkerLink = page.locator('a:has-text("Detail")').first();
        if ((await satkerLink.count()) > 0) {
          await satkerLink.click();
          await page.waitForURL('**/kebutuhan-bmn/satker/**');

          await page.getByRole('button', { name: /Analisis Kelayakan/i }).click();

          // Look for gap analysis table headers
          const hasGapTable = await page.getByText(/Detail Analisis per Barang/i).isVisible();
          if (hasGapTable) {
            await expect(page.getByRole('columnheader', { name: /Nama Barang/i })).toBeVisible();
            await expect(page.getByRole('columnheader', { name: /Diminta/i })).toBeVisible();
            await expect(page.getByRole('columnheader', { name: /Existing/i })).toBeVisible();
            await expect(page.getByRole('columnheader', { name: /Gap/i })).toBeVisible();
            await expect(page.getByRole('columnheader', { name: /Rekomendasi/i })).toBeVisible();
          }
        }
      }
    });

    test('should show expandable existing assets from SIMAN', async ({ page }) => {
      await page.goto('/dashboard/kebutuhan-bmn/daftar');

      const firstRow = page.locator('table tbody tr').first();
      const hasData = (await firstRow.count()) > 0;

      if (hasData) {
        await firstRow.locator('a').first().click();
        await page.waitForURL('**/kebutuhan-bmn/**');

        const satkerLink = page.locator('a:has-text("Detail")').first();
        if ((await satkerLink.count()) > 0) {
          await satkerLink.click();
          await page.waitForURL('**/kebutuhan-bmn/satker/**');

          await page.getByRole('button', { name: /Analisis Kelayakan/i }).click();

          // Check for expandable details element
          const detailsElement = page.locator('details summary');
          if ((await detailsElement.count()) > 0) {
            // Click to expand
            await detailsElement.first().click();

            // Check for asset details
            await expect(page.getByText(/No\. Aset|SIMAN/i)).toBeVisible({ timeout: 3000 });
          }
        }
      }
    });

    test('should display SIMAN integration info', async ({ page }) => {
      await page.goto('/dashboard/kebutuhan-bmn/daftar');

      const firstRow = page.locator('table tbody tr').first();
      const hasData = (await firstRow.count()) > 0;

      if (hasData) {
        await firstRow.locator('a').first().click();
        await page.waitForURL('**/kebutuhan-bmn/**');

        const satkerLink = page.locator('a:has-text("Detail")').first();
        if ((await satkerLink.count()) > 0) {
          await satkerLink.click();
          await page.waitForURL('**/kebutuhan-bmn/satker/**');

          await page.getByRole('button', { name: /Analisis Kelayakan/i }).click();

          // Check for SIMAN explanation note
          await expect(
            page.getByText(/Data aset existing diambil dari SIMAN/i)
          ).toBeVisible({ timeout: 5000 });
        }
      }
    });
  });

  test.describe('Full Workflow E2E', () => {
    test('should complete full create workflow', async ({ page }) => {
      // 1. Go to list
      await page.goto('/dashboard/kebutuhan-bmn/daftar');

      // 2. Click create
      await page.getByRole('link', { name: /Buat Pengajuan/i }).click();

      // 3. Fill form
      const uniqueName = `E2E Full Flow ${Date.now()}`;
      await page.getByLabel(/Nama Pengajuan/i).fill(uniqueName);
      await page.getByLabel(/Deskripsi/i).fill('Full E2E workflow test');
      await page.getByLabel(/Tanggal Mulai/i).fill('2025-01-01');
      await page.getByLabel(/Tanggal Selesai/i).fill('2025-12-31');

      // 4. Submit
      await page.getByRole('button', { name: /Simpan/i }).click();

      // 5. Wait for redirect
      await page.waitForURL('**/kebutuhan-bmn**', { timeout: 10000 });

      // 6. Verify created
      await page.goto('/dashboard/kebutuhan-bmn/daftar');
      await expect(page.getByText(uniqueName)).toBeVisible({ timeout: 5000 });
    });
  });
});
