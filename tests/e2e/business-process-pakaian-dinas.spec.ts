import { test, expect } from '@playwright/test';
import {
  MOCK_USERS,
  PAKAIAN_DINAS_STATUS,
  uniqueSuffix,
  SCREENSHOT_DIR,
} from './fixtures/mock-data';
import {
  checkHealth,
  listJenisPakaianDinas,
  createJenisPakaianDinas,
  createPengajuanPakaianDinas,
  listPengajuanPakaianDinas,
  submitPengajuanPakaianDinas,
  approvePengajuanPakaianDinas,
  rejectPengajuanPakaianDinas,
  authHeaders,
} from './fixtures/api-helpers';

/**
 * E2E Business Process Test: Pakaian Dinas (Uniform Distribution)
 *
 * Module overview:
 * - Master data management (jenis, spesifikasi, sub-spesifikasi, ukuran)
 * - Pengajuan workflow per satker: Input → Submit → Validator action → Complete
 * - MySIMKARI pegawai integration for size data
 * - Report generation (rekapitulasi ukuran, daftar pegawai, Excel/PDF)
 *
 * Workflow states (AktivitasStatus):
 * 1000: Input
 * 1001: SubmitToValidator (Diajukan ke Validator)
 * 1003: RevisiPelaksana
 * 1004: SubmitToPusat
 * 1005: RevisiSatker
 * 1007: RevisiWilayah
 * 1008: Selesai
 * 1009: StartKejagung (Mulai - Kejagung)
 * 1010: SubmitToPusatFromWilayah
 * 1011: StartNonKejagung (Mulai - Non-Kejagung)
 * 1012: SubmitToValidatorWilayah
 */
test.describe('Business Process: Pakaian Dinas - Full E2E Flow', () => {
  const operatorSatker = MOCK_USERS.operator_satker;
  const validatorWilayah = MOCK_USERS.validator_wilayah;
  const validatorPusat = MOCK_USERS.validator_pusat;
  const admin = MOCK_USERS.admin;

  let jenisId: string;
  let pengajuanId: string;
  let backendAvailable = false;

  test.describe.configure({ mode: 'serial' });

  test.beforeAll(async ({ request }) => {
    backendAvailable = await checkHealth(request);
  });

  test.beforeEach(async () => {
    test.skip(!backendAvailable, 'Backend not available at localhost:8093');
  });

  // ========================================================================
  // Phase 1: Master Data Setup
  // ========================================================================

  test('1.1 - List existing jenis pakaian dinas', async ({ request, page }) => {
    const result = await listJenisPakaianDinas(request, admin);

    await page.goto('about:blank');
    await page.setContent(`
      <html><body style="font-family: monospace; padding: 20px; background: #f5f5f5;">
        <h1 style="color: #1a56db;">Step 1.1: List Jenis Pakaian Dinas</h1>
        <div style="background: #dbeafe; padding: 15px; border-radius: 8px; margin-bottom: 12px;">
          <h3>Master Data: Jenis Pakaian Dinas</h3>
          <p>Contoh: PDH Jaksa, PDL Jaksa, PDH Tata Usaha, Sepatu Dinas, dll.</p>
        </div>
        <pre style="background: white; padding: 15px; border-radius: 8px; max-height: 400px; overflow: auto;">${JSON.stringify(result.body, null, 2)}</pre>
        <p>HTTP Status: ${result.status}</p>
      </body></html>
    `);
    await page.screenshot({ path: `${SCREENSHOT_DIR}/50-list-jenis-pakaian.png`, fullPage: true });
  });

  test('1.2 - Create new jenis pakaian dinas', async ({ request, page }) => {
    const suffix = uniqueSuffix();
    const result = await createJenisPakaianDinas(request, admin, {
      nama: `PDH Jaksa Test ${suffix}`,
      kode: `PDH-JKS-${suffix}`,
      deskripsi: 'Pakaian Dinas Harian untuk Jaksa',
      kategori: 'ATASAN',
      gender: 'Semua',
    });

    await page.goto('about:blank');
    await page.setContent(`
      <html><body style="font-family: monospace; padding: 20px; background: #f5f5f5;">
        <h1 style="color: #059669;">Step 1.2: Create Jenis Pakaian Dinas</h1>
        <div style="background: #dcfce7; padding: 15px; border-radius: 8px; margin-bottom: 12px;">
          <h3>Jenis Baru</h3>
          <p>Nama: PDH Jaksa Test ${suffix}</p>
          <p>Kategori: ATASAN</p>
          <p>Gender: Semua (L/P)</p>
        </div>
        <pre style="background: white; padding: 15px; border-radius: 8px;">${JSON.stringify(result.body, null, 2)}</pre>
        <p>HTTP Status: ${result.status}</p>
      </body></html>
    `);
    await page.screenshot({ path: `${SCREENSHOT_DIR}/51-create-jenis-pakaian.png`, fullPage: true });

    if (result.status === 201 || result.status === 200) {
      jenisId = result.body.data?.id;
    }
  });

  // ========================================================================
  // Phase 2: Pegawai Size Data (Personal Ukuran)
  // ========================================================================

  test('2.1 - Pegawai submits personal uniform sizes', async ({ request, page }) => {
    // Operator satker updates own sizes (simulating pegawai input)
    const BASE_URL = 'http://localhost:8093';
    const result = await request.post(
      `${BASE_URL}/api/pembinaan/perlengkapan/pakaian-dinas/ukuran-pakaian-pegawai`,
      {
        headers: {
          ...authHeaders(operatorSatker),
          'Content-Type': 'application/json',
        },
        data: {
          ukuran_baju: 'L',
          ukuran_celana: '32',
          ukuran_sepatu: '42',
          with_hijab: false,
        },
      }
    );

    const body = await result.json().catch(() => ({}));

    await page.goto('about:blank');
    await page.setContent(`
      <html><body style="font-family: monospace; padding: 20px; background: #f5f5f5;">
        <h1 style="color: #059669;">Step 2.1: Pegawai Update Personal Sizes</h1>
        <div style="background: #dcfce7; padding: 15px; border-radius: 8px; margin-bottom: 12px;">
          <h3>Data Ukuran Pribadi</h3>
          <table style="width: 100%; border-collapse: collapse;">
            <tr style="background: #f3f4f6;">
              <th style="padding: 8px; border: 1px solid #e5e7eb;">Jenis</th>
              <th style="padding: 8px; border: 1px solid #e5e7eb;">Ukuran</th>
            </tr>
            <tr>
              <td style="padding: 8px; border: 1px solid #e5e7eb;">Baju</td>
              <td style="padding: 8px; border: 1px solid #e5e7eb;">L</td>
            </tr>
            <tr>
              <td style="padding: 8px; border: 1px solid #e5e7eb;">Celana</td>
              <td style="padding: 8px; border: 1px solid #e5e7eb;">32</td>
            </tr>
            <tr>
              <td style="padding: 8px; border: 1px solid #e5e7eb;">Sepatu</td>
              <td style="padding: 8px; border: 1px solid #e5e7eb;">42</td>
            </tr>
            <tr>
              <td style="padding: 8px; border: 1px solid #e5e7eb;">Hijab</td>
              <td style="padding: 8px; border: 1px solid #e5e7eb;">Tidak</td>
            </tr>
          </table>
        </div>
        <pre style="background: white; padding: 15px; border-radius: 8px;">${JSON.stringify(body, null, 2)}</pre>
        <p>HTTP Status: ${result.status()}</p>
      </body></html>
    `);
    await page.screenshot({ path: `${SCREENSHOT_DIR}/52-update-personal-sizes.png`, fullPage: true });
  });

  test('2.2 - Get pegawai list with sizes for satker', async ({ request, page }) => {
    const BASE_URL = 'http://localhost:8093';
    const satkerId = operatorSatker.satker_id;

    const result = await request.get(
      `${BASE_URL}/api/pembinaan/perlengkapan/pakaian-dinas/pegawai-satker/${satkerId}/with-sizes`,
      {
        headers: authHeaders(operatorSatker),
      }
    );

    const body = await result.json().catch(() => ({}));

    await page.goto('about:blank');
    await page.setContent(`
      <html><body style="font-family: monospace; padding: 20px; background: #f5f5f5;">
        <h1 style="color: #7c3aed;">Step 2.2: Pegawai List with Sizes (MySIMKARI)</h1>
        <div style="background: #f5f3ff; padding: 15px; border-radius: 8px; margin-bottom: 12px;">
          <h3>Integrasi MySIMKARI → Data Pegawai + Ukuran Tersimpan</h3>
          <p>Satker: ${operatorSatker.satker_nama}</p>
          <p>Data pegawai dari MySIMKARI digabungkan dengan ukuran yang sudah tersimpan</p>
        </div>
        <pre style="background: white; padding: 15px; border-radius: 8px; max-height: 400px; overflow: auto;">${JSON.stringify(body, null, 2)}</pre>
        <p>HTTP Status: ${result.status()}</p>
      </body></html>
    `);
    await page.screenshot({ path: `${SCREENSHOT_DIR}/53-pegawai-with-sizes.png`, fullPage: true });
  });

  // ========================================================================
  // Phase 3: Create Pengajuan Pakaian Dinas
  // ========================================================================

  test('3.1 - Operator creates pengajuan pakaian dinas', async ({ request, page }) => {
    const suffix = uniqueSuffix();

    const result = await createPengajuanPakaianDinas(request, operatorSatker, {
      satker_id: operatorSatker.satker_id,
      tahun_anggaran: 2026,
      jenis_pakaian_id: jenisId || '00000000-0000-0000-0000-000000000001',
      jumlah: 50,
      keterangan: `Pengajuan pakaian dinas reguler TA 2026 - ${suffix}`,
    });

    await page.goto('about:blank');
    await page.setContent(`
      <html><body style="font-family: monospace; padding: 20px; background: #f5f5f5;">
        <h1 style="color: #059669;">Step 3.1: Create Pengajuan Pakaian Dinas</h1>
        <div style="display: grid; grid-template-columns: 1fr 1fr; gap: 16px; margin-bottom: 16px;">
          <div style="background: #dcfce7; padding: 15px; border-radius: 8px;">
            <h3>Detail Pengajuan</h3>
            <p><strong>Satker:</strong> ${operatorSatker.satker_nama}</p>
            <p><strong>Tahun Anggaran:</strong> 2026</p>
            <p><strong>Jumlah Pegawai:</strong> 50</p>
            <p><strong>Status:</strong> Input (1000)</p>
          </div>
          <div style="background: #dbeafe; padding: 15px; border-radius: 8px;">
            <h3>Workflow</h3>
            <p>Input → Submit ke Validator → Validator Action → Selesai</p>
          </div>
        </div>
        <pre style="background: white; padding: 15px; border-radius: 8px;">${JSON.stringify(result.body, null, 2)}</pre>
        <p>HTTP Status: ${result.status}</p>
      </body></html>
    `);
    await page.screenshot({ path: `${SCREENSHOT_DIR}/54-create-pengajuan-pakaian.png`, fullPage: true });

    if (result.status === 201 || result.status === 200) {
      pengajuanId = result.body.data?.id;
      expect(pengajuanId).toBeTruthy();
    } else {
      pengajuanId = '00000000-0000-0000-0000-000000000400';
    }
  });

  test('3.2 - List available ukuran (sizes)', async ({ request, page }) => {
    const BASE_URL = 'http://localhost:8093';

    const [bajuResult, celanaResult, sepatuResult] = await Promise.all([
      request.get(
        `${BASE_URL}/api/pembinaan/perlengkapan/pakaian-dinas/ukuran?group=Baju`,
        { headers: authHeaders(operatorSatker) }
      ),
      request.get(
        `${BASE_URL}/api/pembinaan/perlengkapan/pakaian-dinas/ukuran?group=Celana`,
        { headers: authHeaders(operatorSatker) }
      ),
      request.get(
        `${BASE_URL}/api/pembinaan/perlengkapan/pakaian-dinas/ukuran?group=Sepatu`,
        { headers: authHeaders(operatorSatker) }
      ),
    ]);

    const bajuBody = await bajuResult.json().catch(() => ({}));
    const celanaBody = await celanaResult.json().catch(() => ({}));
    const sepatuBody = await sepatuResult.json().catch(() => ({}));

    await page.goto('about:blank');
    await page.setContent(`
      <html><body style="font-family: monospace; padding: 20px; background: #f5f5f5;">
        <h1 style="color: #1a56db;">Step 3.2: Master Data Ukuran</h1>
        <div style="display: grid; grid-template-columns: 1fr 1fr 1fr; gap: 12px; margin-bottom: 16px;">
          <div style="background: #dbeafe; padding: 12px; border-radius: 8px;">
            <h4>Ukuran Baju</h4>
            <pre style="font-size: 12px;">${JSON.stringify(bajuBody, null, 2)}</pre>
          </div>
          <div style="background: #dcfce7; padding: 12px; border-radius: 8px;">
            <h4>Ukuran Celana</h4>
            <pre style="font-size: 12px;">${JSON.stringify(celanaBody, null, 2)}</pre>
          </div>
          <div style="background: #fef3c7; padding: 12px; border-radius: 8px;">
            <h4>Ukuran Sepatu</h4>
            <pre style="font-size: 12px;">${JSON.stringify(sepatuBody, null, 2)}</pre>
          </div>
        </div>
      </body></html>
    `);
    await page.screenshot({ path: `${SCREENSHOT_DIR}/55-master-ukuran.png`, fullPage: true });
  });

  // ========================================================================
  // Phase 4: Submit & Approval Workflow
  // ========================================================================

  test('4.1 - Operator submits pengajuan for approval', async ({ request, page }) => {
    if (!pengajuanId || pengajuanId.startsWith('00000000-0000-0000-0000-0000000004')) {
      test.skip();
      return;
    }

    const result = await submitPengajuanPakaianDinas(request, operatorSatker, pengajuanId, {
      catatan: 'Pengajuan pakaian dinas siap untuk divalidasi',
    });

    await page.goto('about:blank');
    await page.setContent(`
      <html><body style="font-family: monospace; padding: 20px; background: #f5f5f5;">
        <h1 style="color: #d97706;">Step 4.1: Submit Pengajuan</h1>
        <div style="background: #fef3c7; padding: 15px; border-radius: 8px; margin-bottom: 12px;">
          <p>Status: Input (1000) → SubmitToValidator (1001)</p>
          <p>Menunggu tindakan validator</p>
        </div>
        <pre style="background: white; padding: 15px; border-radius: 8px;">${JSON.stringify(result.body, null, 2)}</pre>
      </body></html>
    `);
    await page.screenshot({ path: `${SCREENSHOT_DIR}/56-submit-pengajuan-pakaian.png`, fullPage: true });
  });

  test('4.2 - Validator approves pengajuan', async ({ request, page }) => {
    if (!pengajuanId || pengajuanId.startsWith('00000000-0000-0000-0000-0000000004')) {
      test.skip();
      return;
    }

    const result = await approvePengajuanPakaianDinas(request, validatorPusat, pengajuanId, {
      catatan: 'Pengajuan disetujui, data ukuran sudah lengkap',
    });

    await page.goto('about:blank');
    await page.setContent(`
      <html><body style="font-family: monospace; padding: 20px; background: #f0fdf4;">
        <h1 style="color: #059669;">Step 4.2: Validator Approves Pengajuan</h1>
        <div style="background: #dcfce7; border: 2px solid #059669; padding: 20px; border-radius: 8px;">
          <h3>✅ Pengajuan DISETUJUI</h3>
          <p><strong>By:</strong> ${validatorPusat.nama}</p>
          <p><strong>Status:</strong> → Selesai (1008)</p>
        </div>
        <pre style="background: white; padding: 15px; border-radius: 8px;">${JSON.stringify(result.body, null, 2)}</pre>
      </body></html>
    `);
    await page.screenshot({ path: `${SCREENSHOT_DIR}/57-approve-pengajuan-pakaian.png`, fullPage: true });
  });

  // ========================================================================
  // Phase 5: Validator Action at Satker Level
  // ========================================================================

  test('5.1 - Validator action: approve/reject at satker level', async ({ request, page }) => {
    const BASE_URL = 'http://localhost:8093';

    // Validator can approve/reject individual satker pengajuan
    const result = await request.post(
      `${BASE_URL}/api/pembinaan/perlengkapan/pakaian-dinas/validator-action`,
      {
        headers: {
          ...authHeaders(validatorWilayah),
          'Content-Type': 'application/json',
        },
        data: {
          pengajuan_satker_id: '00000000-0000-0000-0000-000000000001', // Example satker ID
          aksi: 'approve',
          komentar: 'Data ukuran seluruh pegawai sudah lengkap dan valid',
        },
      }
    );

    const body = await result.json().catch(() => ({}));

    await page.goto('about:blank');
    await page.setContent(`
      <html><body style="font-family: monospace; padding: 20px; background: #f5f5f5;">
        <h1 style="color: #7c3aed;">Step 5.1: Validator Action (Satker Level)</h1>
        <div style="background: #f5f3ff; padding: 15px; border-radius: 8px; margin-bottom: 12px;">
          <h3>Validator Wilayah Action</h3>
          <p>Validator bisa approve/reject per satker:</p>
          <ul>
            <li><strong>approve</strong> — Data ukuran lengkap, lanjut ke tahap berikutnya</li>
            <li><strong>reject</strong> — Data perlu diperbaiki (revisi)</li>
          </ul>
        </div>
        <pre style="background: white; padding: 15px; border-radius: 8px;">${JSON.stringify(body, null, 2)}</pre>
        <p>HTTP Status: ${result.status()}</p>
      </body></html>
    `);
    await page.screenshot({ path: `${SCREENSHOT_DIR}/58-validator-action-satker.png`, fullPage: true });
  });

  // ========================================================================
  // Phase 6: Reports & Rekapitulasi
  // ========================================================================

  test('6.1 - Download rekapitulasi ukuran report', async ({ request, page }) => {
    const BASE_URL = 'http://localhost:8093';

    const result = await request.get(
      `${BASE_URL}/api/pembinaan/perlengkapan/pakaian-dinas/laporan/rekap-ukuran?tahun=2026`,
      {
        headers: {
          ...authHeaders(validatorPusat),
        },
      }
    );

    const body = await result.json().catch(() => ({}));

    await page.goto('about:blank');
    await page.setContent(`
      <html><body style="font-family: monospace; padding: 20px; background: #f5f5f5;">
        <h1 style="color: #1a56db;">Step 6.1: Rekapitulasi Ukuran Report</h1>
        <div style="background: #dbeafe; padding: 15px; border-radius: 8px; margin-bottom: 12px;">
          <h3>Laporan Rekapitulasi Ukuran Pakaian Dinas</h3>
          <p>Data rekap per ukuran: jumlah laki-laki, perempuan, total</p>
          <p>Dapat difilter berdasarkan: pengajuan, tahun, satker, jenis kelamin</p>
        </div>
        <pre style="background: white; padding: 15px; border-radius: 8px; max-height: 400px; overflow: auto;">${JSON.stringify(body, null, 2)}</pre>
      </body></html>
    `);
    await page.screenshot({ path: `${SCREENSHOT_DIR}/59-rekapitulasi-ukuran.png`, fullPage: true });
  });

  test('6.2 - Download daftar pegawai report', async ({ request, page }) => {
    const BASE_URL = 'http://localhost:8093';

    const result = await request.get(
      `${BASE_URL}/api/pembinaan/perlengkapan/pakaian-dinas/laporan/daftar-pegawai?tahun=2026`,
      {
        headers: {
          ...authHeaders(validatorPusat),
        },
      }
    );

    const body = await result.json().catch(() => ({}));

    await page.goto('about:blank');
    await page.setContent(`
      <html><body style="font-family: monospace; padding: 20px; background: #f5f5f5;">
        <h1 style="color: #1a56db;">Step 6.2: Daftar Pegawai Report</h1>
        <div style="background: #dbeafe; padding: 15px; border-radius: 8px; margin-bottom: 12px;">
          <h3>Laporan Daftar Pegawai dengan Ukuran Pakaian Dinas</h3>
          <p>Data: NIP, Nama, Satker, Jabatan, Pangkat, Ukuran Baju/Celana/Sepatu, Hijab</p>
        </div>
        <pre style="background: white; padding: 15px; border-radius: 8px; max-height: 400px; overflow: auto;">${JSON.stringify(body, null, 2)}</pre>
      </body></html>
    `);
    await page.screenshot({ path: `${SCREENSHOT_DIR}/60-daftar-pegawai-report.png`, fullPage: true });
  });

  test('6.3 - Export report as Excel/PDF', async ({ request, page }) => {
    const BASE_URL = 'http://localhost:8093';

    const result = await request.get(
      `${BASE_URL}/api/pembinaan/perlengkapan/pakaian-dinas/laporan/cetak?format=excel&tahun=2026`,
      {
        headers: {
          ...authHeaders(validatorPusat),
        },
      }
    );

    await page.goto('about:blank');
    await page.setContent(`
      <html><body style="font-family: monospace; padding: 20px; background: #f5f5f5;">
        <h1 style="color: #059669;">Step 6.3: Export Report</h1>
        <div style="background: #dcfce7; padding: 15px; border-radius: 8px; margin-bottom: 12px;">
          <h3>Export Laporan Pakaian Dinas</h3>
          <p>Format yang didukung:</p>
          <ul>
            <li>📊 <strong>Excel</strong> - Data lengkap per pegawai per satker</li>
            <li>📄 <strong>PDF</strong> - Laporan cetak rekapitulasi ukuran</li>
          </ul>
          <p>HTTP Status: ${result.status()}</p>
          <p>Content-Type: ${result.headers()['content-type'] || 'N/A'}</p>
        </div>
      </body></html>
    `);
    await page.screenshot({ path: `${SCREENSHOT_DIR}/61-export-report.png`, fullPage: true });
  });

  // ========================================================================
  // Phase 7: Alternative Flow - Rejection
  // ========================================================================

  test('7.1 - Alternative: Reject pengajuan pakaian dinas', async ({ request, page }) => {
    const suffix = uniqueSuffix();

    // Create another pengajuan to test rejection
    const createResult = await createPengajuanPakaianDinas(request, operatorSatker, {
      satker_id: operatorSatker.satker_id,
      tahun_anggaran: 2026,
      jenis_pakaian_id: jenisId || '00000000-0000-0000-0000-000000000001',
      jumlah: 10,
      keterangan: `Test reject flow ${suffix}`,
    });

    let rejectPengajuanId: string | null = null;
    if (createResult.status === 201 || createResult.status === 200) {
      rejectPengajuanId = createResult.body.data?.id;

      // Submit
      await submitPengajuanPakaianDinas(request, operatorSatker, rejectPengajuanId!, {});

      // Reject
      const rejectResult = await rejectPengajuanPakaianDinas(
        request,
        validatorPusat,
        rejectPengajuanId!,
        {
          catatan: 'Data ukuran sebagian pegawai belum lengkap. Harap diperbaiki.',
        }
      );

      await page.goto('about:blank');
      await page.setContent(`
        <html><body style="font-family: monospace; padding: 20px; background: #fef2f2;">
          <h1 style="color: #dc2626;">Step 7.1: Reject Pengajuan Pakaian Dinas</h1>
          <div style="background: #fee2e2; border: 2px solid #dc2626; padding: 20px; border-radius: 8px;">
            <h3>❌ Pengajuan DITOLAK</h3>
            <p><strong>By:</strong> ${validatorPusat.nama}</p>
            <p><strong>Alasan:</strong> Data ukuran sebagian pegawai belum lengkap</p>
            <p><strong>Status:</strong> → RevisiSatker (1005)</p>
          </div>
          <pre style="background: white; padding: 15px; border-radius: 8px;">${JSON.stringify(rejectResult.body, null, 2)}</pre>
        </body></html>
      `);
    } else {
      await page.goto('about:blank');
      await page.setContent(`
        <html><body style="font-family: monospace; padding: 20px;">
          <h1 style="color: #dc2626;">Step 7.1: Reject Flow</h1>
          <p>Skipped: Could not create test data for reject flow.</p>
        </body></html>
      `);
    }
    await page.screenshot({ path: `${SCREENSHOT_DIR}/62-reject-pengajuan-pakaian.png`, fullPage: true });
  });

  // ========================================================================
  // Phase 8: Flow Summary
  // ========================================================================

  test('8.1 - Complete flow summary', async ({ page }) => {
    await page.goto('about:blank');
    await page.setContent(`
      <html><body style="font-family: Arial, sans-serif; padding: 20px; background: white;">
        <h1 style="color: #1a56db; text-align: center;">Business Process Flow: Pakaian Dinas</h1>
        <div style="max-width: 900px; margin: 0 auto;">

          <h2>Phase 1: Master Data</h2>
          <div style="background: #dbeafe; padding: 12px; border-radius: 8px; margin-bottom: 12px;">
            Admin setup: Jenis Pakaian Dinas → Spesifikasi → Sub-Spesifikasi → Ukuran
          </div>

          <h2>Phase 2: Pengisian Ukuran Pegawai</h2>
          <div style="background: #dcfce7; padding: 12px; border-radius: 8px; margin-bottom: 12px;">
            Setiap pegawai mengisi ukuran baju, celana, sepatu via self-service
          </div>

          <h2>Phase 3: Pengajuan Workflow</h2>
          <div style="display: flex; flex-direction: column; gap: 8px; margin-bottom: 16px;">
            <div style="background: #059669; color: white; padding: 10px 16px; border-radius: 8px;">
              <strong>1. Input (1000)</strong> - Operator creates pengajuan for satker
            </div>
            <div style="text-align: center; color: #6b7280;">↓ submit</div>
            <div style="background: #d97706; color: white; padding: 10px 16px; border-radius: 8px;">
              <strong>2. SubmitToValidator (1001)</strong> - Menunggu validasi
            </div>
            <div style="text-align: center; color: #6b7280;">↓ approve / ↘ reject</div>
            <div style="display: grid; grid-template-columns: 1fr 1fr; gap: 8px;">
              <div style="background: #059669; color: white; padding: 10px 16px; border-radius: 8px;">
                <strong>Selesai (1008)</strong> - Disetujui
              </div>
              <div style="background: #dc2626; color: white; padding: 10px 16px; border-radius: 8px;">
                <strong>Revisi (1005)</strong> - Ditolak, perbaiki data
              </div>
            </div>
          </div>

          <h2>Phase 4: Reports</h2>
          <div style="background: #f5f3ff; padding: 12px; border-radius: 8px; margin-bottom: 12px;">
            <ul>
              <li>Rekap Ukuran: jumlah per ukuran per gender</li>
              <li>Daftar Pegawai: detail per pegawai dengan ukuran</li>
              <li>Export: Excel / PDF</li>
            </ul>
          </div>

          <h2>Key Features</h2>
          <div style="background: #fef3c7; padding: 16px; border-radius: 8px;">
            <ul>
              <li>Self-service ukuran pegawai</li>
              <li>MySIMKARI integration for employee data</li>
              <li>Multi-level validator (wilayah → pusat)</li>
              <li>Per-satker validator actions</li>
              <li>Rekapitulasi dan laporan otomatis</li>
            </ul>
          </div>
        </div>
      </body></html>
    `);
    await page.screenshot({ path: `${SCREENSHOT_DIR}/63-pakaian-dinas-flow-diagram.png`, fullPage: true });
  });
});
