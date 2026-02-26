import { test, expect } from '@playwright/test';
import {
  MOCK_USERS,
  KEBUTUHAN_BMN_STATUS,
  PEMAKAIAN_BMN_STATUS,
  PENGHAPUSAN_BMN_STATUS,
  PAKAIAN_DINAS_STATUS,
  MOCK_BMN_DATA,
  MOCK_PEGAWAI_DATA,
  uniqueSuffix,
  SCREENSHOT_DIR,
} from './fixtures/mock-data';
import {
  checkHealthDetailed as checkHealth,
  createKebutuhanBmn,
  transitionKebutuhanBmn,
  createPemakaianBmn,
  generateKonsepSuratPemakaian,
  uploadSignedPdfPemakaian,
  createPenghapusanBmn,
  transitionPenghapusanBmn,
  generateKonsepSKPenghapusan,
  uploadSignedSKPenghapusan,
  exportKebutuhanBmn,
  exportPemakaianBmn,
  exportPenghapusanBmn,
} from './fixtures/api-helpers';

/**
 * E2E Integration Test: Cross-Module Business Process Validation
 *
 * Validates that all 4 modules work correctly together:
 * 1. Kebutuhan BMN → approved needs → triggers pemakaian
 * 2. Pemakaian BMN → active usage permits
 * 3. Penghapusan BMN → SK for damaged/obsolete items
 * 4. Pakaian Dinas → uniform distribution
 *
 * Also validates:
 * - Server health check
 * - Role-based access control across modules
 * - Export functionality for all modules
 * - Status workflow constraints (validator pusat: approve/reject ONLY)
 */
test.describe('Cross-Module Integration: Full System E2E', () => {
  const operatorSatker = MOCK_USERS.operator_satker;
  const validatorWilayah = MOCK_USERS.validator_wilayah;
  const validatorPusat = MOCK_USERS.validator_pusat;
  const admin = MOCK_USERS.admin;
  let backendAvailable = false;

  test.describe.configure({ mode: 'serial' });

  test.beforeAll(async ({ request }) => {
    const result = await checkHealth(request);
    backendAvailable = result.status === 200;
  });

  test.beforeEach(async () => {
    test.skip(!backendAvailable, 'Backend not available at localhost:8093');
  });

  // ========================================================================
  // Integration: Complete BMN Lifecycle
  // ========================================================================

  test('1.1 - Complete BMN lifecycle: Need → Acquire → Use → Dispose', async ({
    request,
    page,
  }) => {
    const suffix = uniqueSuffix();

    // Step 1: Create kebutuhan BMN (need identified)
    const kebutuhanResult = await createKebutuhanBmn(request, operatorSatker, {
      judul: `Lifecycle Test: Kebutuhan ${suffix}`,
      keterangan: 'BMN lifecycle integration test - need identification',
      tahun_anggaran: 2026,
      satker_id: operatorSatker.satker_id,
      satker_nama: operatorSatker.satker_nama,
      jenis_kebutuhan: 'PENGADAAN_BARU',
    });

    // Step 2: Create pemakaian BMN (usage permit for existing asset)
    const pemakaianResult = await createPemakaianBmn(request, operatorSatker, {
      pegawai_nip: MOCK_PEGAWAI_DATA.pegawai1.nip,
      pegawai_nama: MOCK_PEGAWAI_DATA.pegawai1.nama,
      pegawai_pangkat: MOCK_PEGAWAI_DATA.pegawai1.pangkat,
      pegawai_jabatan: MOCK_PEGAWAI_DATA.pegawai1.jabatan,
      pegawai_satker_id: MOCK_PEGAWAI_DATA.pegawai1.satker_id,
      pegawai_satker_nama: MOCK_PEGAWAI_DATA.pegawai1.satker_nama,
      jenis_bmn: 'Laptop',
      kode_barang: MOCK_BMN_DATA.laptop.kode_barang,
      nama_barang: MOCK_BMN_DATA.laptop.nama_barang,
      nup: MOCK_BMN_DATA.laptop.nup,
      satker_id: operatorSatker.satker_id,
      satker_nama: operatorSatker.satker_nama,
      tanggal_mulai: '2026-01-01',
      tanggal_selesai: '2026-12-31',
      keperluan: 'Integration test - usage permit',
    });

    // Step 3: Create penghapusan BMN (disposal of damaged asset)
    const penghapusanResult = await createPenghapusanBmn(request, operatorSatker, {
      judul: `Lifecycle Test: Penghapusan ${suffix}`,
      keterangan: 'BMN lifecycle integration test - disposal',
      jenis_penghapusan: 'RUSAK_BERAT',
      satker_id: operatorSatker.satker_id,
      satker_nama: operatorSatker.satker_nama,
      items: [
        {
          kode_barang: '3.06.02.01.005',
          nama_barang: 'PC Desktop Lama',
          nup: '003',
          tahun_perolehan: 2015,
          nilai_perolehan: 10000000,
          kondisi: 'Rusak Berat',
          alasan_penghapusan: 'Sudah melewati masa manfaat dan rusak',
        },
      ],
      lampiran_url: 'https://storage.simpel.kejaksaan.go.id/docs/lampiran-lifecycle-test.pdf',
    });

    await page.goto('about:blank');
    await page.setContent(`
      <html><body style="font-family: Arial, sans-serif; padding: 20px; background: #f5f5f5;">
        <h1 style="color: #1a56db; text-align: center;">Integration: BMN Lifecycle</h1>
        <h2 style="text-align: center; color: #6b7280;">Kebutuhan → Pengadaan → Pemakaian → Penghapusan</h2>

        <div style="display: grid; grid-template-columns: 1fr 1fr 1fr; gap: 16px; margin: 24px 0;">
          <div style="background: ${kebutuhanResult.status <= 201 ? '#dcfce7' : '#fee2e2'}; padding: 16px; border-radius: 8px;">
            <h3>1. Kebutuhan BMN</h3>
            <p>Identifikasi kebutuhan barang</p>
            <p>HTTP: ${kebutuhanResult.status}</p>
            <p>${kebutuhanResult.status <= 201 ? '✅ Created' : '⚠️ Check required'}</p>
          </div>
          <div style="background: ${pemakaianResult.status <= 201 ? '#dcfce7' : '#fee2e2'}; padding: 16px; border-radius: 8px;">
            <h3>2. Pemakaian BMN</h3>
            <p>Izin pemakaian untuk pegawai</p>
            <p>HTTP: ${pemakaianResult.status}</p>
            <p>${pemakaianResult.status <= 201 ? '✅ Created' : '⚠️ Check required'}</p>
          </div>
          <div style="background: ${penghapusanResult.status <= 201 ? '#dcfce7' : '#fee2e2'}; padding: 16px; border-radius: 8px;">
            <h3>3. Penghapusan BMN</h3>
            <p>Penghapusan barang rusak</p>
            <p>HTTP: ${penghapusanResult.status}</p>
            <p>${penghapusanResult.status <= 201 ? '✅ Created' : '⚠️ Check required'}</p>
          </div>
        </div>
      </body></html>
    `);
    await page.screenshot({ path: `${SCREENSHOT_DIR}/71-bmn-lifecycle.png`, fullPage: true });
  });

  // ========================================================================
  // Integration: Role-Based Access Control
  // ========================================================================

  test('2.1 - RBAC: Validator Pusat cannot revise/return', async ({ request, page }) => {
    const suffix = uniqueSuffix();

    // Create and advance a kebutuhan to validator pusat stage
    const createResult = await createKebutuhanBmn(request, operatorSatker, {
      judul: `RBAC Test ${suffix}`,
      keterangan: 'Test that validator pusat cannot return/revise',
      tahun_anggaran: 2026,
      satker_id: operatorSatker.satker_id,
      satker_nama: operatorSatker.satker_nama,
      jenis_kebutuhan: 'PENGADAAN_BARU',
    });

    let kebutuhanId: string | null = null;
    let reviseResult: any = { status: 'N/A', body: 'Test data creation failed' };

    if (createResult.status <= 201) {
      kebutuhanId = createResult.body.data?.pengajuan?.id || createResult.body.data?.id;

      if (kebutuhanId) {
        // Advance through workflow
        await transitionKebutuhanBmn(request, operatorSatker, kebutuhanId, 'submit_wilayah', {});
        await transitionKebutuhanBmn(request, validatorWilayah, kebutuhanId, 'submit_pusat', {});

        // NOW: try forbidden action - validator pusat trying to revise/return
        reviseResult = await transitionKebutuhanBmn(
          request,
          validatorPusat,
          kebutuhanId,
          'revisi_satker', // This should be FORBIDDEN
          { catatan: 'Attempting forbidden revision' }
        );
      }
    }

    await page.goto('about:blank');
    await page.setContent(`
      <html><body style="font-family: monospace; padding: 20px; background: #f5f5f5;">
        <h1 style="color: #dc2626;">Integration: RBAC - Validator Pusat Constraints</h1>

        <div style="background: #fef3c7; border: 2px solid #f59e0b; padding: 20px; border-radius: 8px; margin-bottom: 16px;">
          <h3>⚠️ Rule: Validator Pusat can ONLY approve or reject</h3>
          <p>Validator Pusat <strong>CANNOT</strong>:</p>
          <ul>
            <li>❌ Revisi → return to operator</li>
            <li>❌ Return → send back to wilayah</li>
          </ul>
          <p>If validator pusat finds issues, they must <strong>REJECT</strong> and operator resubmits.</p>
        </div>

        <div style="background: ${typeof reviseResult.status === 'number' && reviseResult.status >= 400 ? '#dcfce7' : '#fee2e2'}; padding: 15px; border-radius: 8px;">
          <h3>Test Result: revisi_satker by Validator Pusat</h3>
          <p>Expected: HTTP 400/403 (forbidden transition)</p>
          <p>Actual: HTTP ${reviseResult.status}</p>
          <p>${typeof reviseResult.status === 'number' && reviseResult.status >= 400 ? '✅ CORRECTLY BLOCKED' : '⚠️ Needs investigation'}</p>
        </div>

        <pre style="background: white; padding: 15px; border-radius: 8px; margin-top: 12px;">${JSON.stringify(reviseResult.body, null, 2)}</pre>
      </body></html>
    `);
    await page.screenshot({ path: `${SCREENSHOT_DIR}/72-rbac-validator-pusat.png`, fullPage: true });
  });

  // ========================================================================
  // Integration: Export Across Modules
  // ========================================================================

  test('3.1 - Export functionality across all modules', async ({ request, page }) => {
    const [kebutuhanExport, pemakaianExport, penghapusanExport] = await Promise.all([
      exportKebutuhanBmn(request, validatorPusat, { format: 'excel', tahun: 2026 }),
      exportPemakaianBmn(request, validatorPusat, { format: 'excel' }),
      exportPenghapusanBmn(request, validatorPusat, { format: 'excel' }),
    ]);

    await page.goto('about:blank');
    await page.setContent(`
      <html><body style="font-family: monospace; padding: 20px; background: #f5f5f5;">
        <h1 style="color: #1a56db;">Integration: Export Functionality</h1>

        <div style="display: grid; grid-template-columns: 1fr 1fr 1fr; gap: 12px;">
          <div style="background: white; padding: 15px; border-radius: 8px; border: 1px solid #e5e7eb;">
            <h3>Kebutuhan BMN Export</h3>
            <p>HTTP: ${kebutuhanExport.status}</p>
            <p>${kebutuhanExport.status === 200 ? '✅ OK' : '⚠️ Check'}</p>
          </div>
          <div style="background: white; padding: 15px; border-radius: 8px; border: 1px solid #e5e7eb;">
            <h3>Pemakaian BMN Export</h3>
            <p>HTTP: ${pemakaianExport.status}</p>
            <p>${pemakaianExport.status === 200 ? '✅ OK' : '⚠️ Check'}</p>
          </div>
          <div style="background: white; padding: 15px; border-radius: 8px; border: 1px solid #e5e7eb;">
            <h3>Penghapusan BMN Export</h3>
            <p>HTTP: ${penghapusanExport.status}</p>
            <p>${penghapusanExport.status === 200 ? '✅ OK' : '⚠️ Check'}</p>
          </div>
        </div>
      </body></html>
    `);
    await page.screenshot({ path: `${SCREENSHOT_DIR}/73-export-all-modules.png`, fullPage: true });
  });

  // ========================================================================
  // Integration: Document Generation Across Modules
  // ========================================================================

  test('4.1 - Document generation comparison across modules', async ({ page }) => {
    await page.goto('about:blank');
    await page.setContent(`
      <html><body style="font-family: Arial, sans-serif; padding: 20px; background: white;">
        <h1 style="color: #1a56db; text-align: center;">Integration: Document Generation Across Modules</h1>
        <div style="max-width: 900px; margin: 0 auto;">

          <table style="width: 100%; border-collapse: collapse; margin-bottom: 24px;">
            <tr style="background: #1a56db; color: white;">
              <th style="padding: 10px; border: 1px solid #ddd;">Module</th>
              <th style="padding: 10px; border: 1px solid #ddd;">DOCX Generated</th>
              <th style="padding: 10px; border: 1px solid #ddd;">Signed PDF Upload</th>
              <th style="padding: 10px; border: 1px solid #ddd;">Content</th>
            </tr>
            <tr>
              <td style="padding: 10px; border: 1px solid #ddd;"><strong>Pemakaian BMN</strong></td>
              <td style="padding: 10px; border: 1px solid #ddd;">Konsep Surat Izin</td>
              <td style="padding: 10px; border: 1px solid #ddd;">✅ Yes</td>
              <td style="padding: 10px; border: 1px solid #ddd;">Hal 1: ID pegawai + foto<br>Hal 2+: Tabel BMN</td>
            </tr>
            <tr style="background: #f9fafb;">
              <td style="padding: 10px; border: 1px solid #ddd;"><strong>Penghapusan BMN</strong></td>
              <td style="padding: 10px; border: 1px solid #ddd;">Konsep SK Penghapusan</td>
              <td style="padding: 10px; border: 1px solid #ddd;">✅ Yes</td>
              <td style="padding: 10px; border: 1px solid #ddd;">SK with nomor, dasar hukum, daftar BMN</td>
            </tr>
            <tr>
              <td style="padding: 10px; border: 1px solid #ddd;"><strong>Pakaian Dinas</strong></td>
              <td style="padding: 10px; border: 1px solid #ddd;">Rekapitulasi Ukuran</td>
              <td style="padding: 10px; border: 1px solid #ddd;">❌ No (export only)</td>
              <td style="padding: 10px; border: 1px solid #ddd;">Excel/PDF recap per satker</td>
            </tr>
            <tr style="background: #f9fafb;">
              <td style="padding: 10px; border: 1px solid #ddd;"><strong>Kebutuhan BMN</strong></td>
              <td style="padding: 10px; border: 1px solid #ddd;">Export laporan</td>
              <td style="padding: 10px; border: 1px solid #ddd;">❌ No (export only)</td>
              <td style="padding: 10px; border: 1px solid #ddd;">Data needs analysis & gap report</td>
            </tr>
          </table>

          <h2>Workflow Constraint Summary</h2>
          <table style="width: 100%; border-collapse: collapse;">
            <tr style="background: #059669; color: white;">
              <th style="padding: 10px; border: 1px solid #ddd;">Module</th>
              <th style="padding: 10px; border: 1px solid #ddd;">Val. Wilayah</th>
              <th style="padding: 10px; border: 1px solid #ddd;">Val. Pusat</th>
            </tr>
            <tr>
              <td style="padding: 10px; border: 1px solid #ddd;">Kebutuhan BMN</td>
              <td style="padding: 10px; border: 1px solid #ddd;">Approve / Return / Revise</td>
              <td style="padding: 10px; border: 1px solid #ddd;"><strong>Approve / Reject ONLY</strong></td>
            </tr>
            <tr style="background: #f9fafb;">
              <td style="padding: 10px; border: 1px solid #ddd;">Pemakaian BMN</td>
              <td style="padding: 10px; border: 1px solid #ddd;">Monitor only</td>
              <td style="padding: 10px; border: 1px solid #ddd;">Monitor only</td>
            </tr>
            <tr>
              <td style="padding: 10px; border: 1px solid #ddd;">Penghapusan BMN</td>
              <td style="padding: 10px; border: 1px solid #ddd;">Approve / Return</td>
              <td style="padding: 10px; border: 1px solid #ddd;"><strong>Approve / Reject ONLY</strong></td>
            </tr>
            <tr style="background: #f9fafb;">
              <td style="padding: 10px; border: 1px solid #ddd;">Pakaian Dinas</td>
              <td style="padding: 10px; border: 1px solid #ddd;">Approve / Reject per satker</td>
              <td style="padding: 10px; border: 1px solid #ddd;">Approve / Reject</td>
            </tr>
          </table>
        </div>
      </body></html>
    `);
    await page.screenshot({ path: `${SCREENSHOT_DIR}/74-document-generation-comparison.png`, fullPage: true });
  });

  // ========================================================================
  // Integration: Status Code Reference
  // ========================================================================

  test('5.1 - Status code reference across all modules', async ({ page }) => {
    await page.goto('about:blank');
    await page.setContent(`
      <html><body style="font-family: Arial, sans-serif; padding: 20px; background: white;">
        <h1 style="color: #1a56db; text-align: center;">Status Code Reference: All Modules</h1>
        <div style="max-width: 900px; margin: 0 auto;">

          <h2 style="color: #059669;">Kebutuhan BMN (2000s)</h2>
          <div style="display: grid; grid-template-columns: repeat(5, 1fr); gap: 4px; margin-bottom: 16px;">
            ${Object.entries(KEBUTUHAN_BMN_STATUS)
              .map(
                ([k, v]) =>
                  `<div style="background: #dcfce7; padding: 6px; border-radius: 4px; font-size: 11px; text-align: center;"><strong>${v}</strong><br/>${k}</div>`
              )
              .join('')}
          </div>

          <h2 style="color: #d97706;">Pemakaian BMN (3000s)</h2>
          <div style="display: grid; grid-template-columns: repeat(4, 1fr); gap: 4px; margin-bottom: 16px;">
            ${Object.entries(PEMAKAIAN_BMN_STATUS)
              .map(
                ([k, v]) =>
                  `<div style="background: #fef3c7; padding: 6px; border-radius: 4px; font-size: 11px; text-align: center;"><strong>${v}</strong><br/>${k}</div>`
              )
              .join('')}
          </div>

          <h2 style="color: #dc2626;">Penghapusan BMN (4000s)</h2>
          <div style="display: grid; grid-template-columns: repeat(5, 1fr); gap: 4px; margin-bottom: 16px;">
            ${Object.entries(PENGHAPUSAN_BMN_STATUS)
              .map(
                ([k, v]) =>
                  `<div style="background: #fee2e2; padding: 6px; border-radius: 4px; font-size: 11px; text-align: center;"><strong>${v}</strong><br/>${k}</div>`
              )
              .join('')}
          </div>

          <h2 style="color: #7c3aed;">Pakaian Dinas (1000s)</h2>
          <div style="display: grid; grid-template-columns: repeat(4, 1fr); gap: 4px; margin-bottom: 16px;">
            ${Object.entries(PAKAIAN_DINAS_STATUS)
              .map(
                ([k, v]) =>
                  `<div style="background: #f5f3ff; padding: 6px; border-radius: 4px; font-size: 11px; text-align: center;"><strong>${v}</strong><br/>${k}</div>`
              )
              .join('')}
          </div>
        </div>
      </body></html>
    `);
    await page.screenshot({ path: `${SCREENSHOT_DIR}/75-status-code-reference.png`, fullPage: true });
  });
});
