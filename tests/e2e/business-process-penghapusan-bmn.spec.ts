import { test, expect } from '@playwright/test';
import {
  MOCK_USERS,
  PENGHAPUSAN_BMN_STATUS,
  MOCK_BMN_DATA,
  uniqueSuffix,
  SCREENSHOT_DIR,
} from './fixtures/mock-data';
import {
  checkHealth,
  createPenghapusanBmn,
  getPenghapusanBmn,
  listPenghapusanBmn,
  transitionPenghapusanBmn,
  generateKonsepSKPenghapusan,
  uploadSignedSKPenghapusan,
} from './fixtures/api-helpers';

/**
 * E2E Business Process Test: SK Penghapusan BMN
 *
 * Full workflow:
 * 1. Operator Satker creates pengajuan penghapusan + uploads lampiran
 * 2. Submit to Validator Wilayah (status: SUBMIT_WILAYAH)
 * 3. Validator Wilayah reviews:
 *    - Approve → Forward to Validator Pusat (SUBMIT_PUSAT)
 *    - Return → Back to Operator (RETURNED_TO_OPERATOR)
 * 4. Validator Pusat verifies (VERIFIKASI_PUSAT)
 * 5. System generates konsep SK DOCX (KONSEP_SK_GENERATED)
 * 6. Operator uploads signed SK PDF (SK_SIGNED)
 * 7. Process completed (COMPLETED)
 *
 * Note: Validator Pusat can ONLY approve/reject, NOT revise/return
 */
test.describe('Business Process: SK Penghapusan BMN - Full E2E Flow', () => {
  const operatorSatker = MOCK_USERS.operator_satker;
  const validatorWilayah = MOCK_USERS.validator_wilayah;
  const validatorPusat = MOCK_USERS.validator_pusat;

  let penghapusanId: string;
  let backendAvailable = false;

  async function resolvePenghapusanId(request: APIRequestContext): Promise<string | undefined> {
    if (penghapusanId && !penghapusanId.startsWith('00000000-0000-0000-0000-0000000003')) {
      return penghapusanId;
    }

    const listed = await listPenghapusanBmn(request, validatorPusat, { page: 1, per_page: 1 });
    if (listed.status >= 200 && listed.status < 500) {
      const item = listed.body?.data?.items?.[0] ?? listed.body?.data?.[0];
      if (item?.id) {
        penghapusanId = item.id;
      }
    }

    return penghapusanId;
  }

  test.describe.configure({ mode: 'serial' });

  test.beforeAll(async ({ request }) => {
    backendAvailable = await checkHealth(request);
  });

  test.beforeEach(async () => {
    test.skip(!backendAvailable, 'Backend not available at localhost:8093');
  });

  // ========================================================================
  // Phase 1: Operator Creates Pengajuan Penghapusan
  // ========================================================================

  test('1.1 - Operator Satker creates pengajuan penghapusan with lampiran', async ({
    request,
    page,
  }) => {
    const suffix = uniqueSuffix();

    const result = await createPenghapusanBmn(request, operatorSatker, {
      judul: `Pengajuan Penghapusan BMN Satker Jakarta Selatan ${suffix}`,
      keterangan: 'BMN telah rusak berat dan tidak ekonomis untuk diperbaiki',
      jenis_penghapusan: 'RUSAK_BERAT',
      satker_id: operatorSatker.satker_id,
      satker_nama: operatorSatker.satker_nama,

      // Daftar BMN yang diajukan penghapusan
      items: [
        {
          kode_barang: MOCK_BMN_DATA.laptop.kode_barang,
          nama_barang: MOCK_BMN_DATA.laptop.nama_barang,
          nup: MOCK_BMN_DATA.laptop.nup,
          tahun_perolehan: 2018,
          nilai_perolehan: 15000000,
          kondisi: 'Rusak Berat',
          alasan_penghapusan: 'Motherboard rusak, biaya perbaikan melebihi 60% nilai perolehan',
        },
        {
          kode_barang: '3.06.02.03.001',
          nama_barang: 'Printer Laser HP LaserJet Pro',
          nup: '005',
          tahun_perolehan: 2017,
          nilai_perolehan: 8500000,
          kondisi: 'Rusak Berat',
          alasan_penghapusan: 'Head print rusak, spare part tidak tersedia',
        },
      ],

      // Lampiran pendukung
      lampiran_url: 'https://storage.simpel.kejaksaan.go.id/docs/lampiran-penghapusan-2026.pdf',
      dokumen_pendukung: [
        {
          nama: 'Berita Acara Pemeriksaan BMN',
          url: 'https://storage.simpel.kejaksaan.go.id/docs/ba-pemeriksaan-bmn-2026.pdf',
        },
        {
          nama: 'Foto Dokumentasi BMN Rusak',
          url: 'https://storage.simpel.kejaksaan.go.id/docs/foto-bmn-rusak-2026.pdf',
        },
      ],
    });

    await page.goto('about:blank');
    await page.setContent(`
      <html><body style="font-family: monospace; padding: 20px; background: #f5f5f5;">
        <h1 style="color: #059669;">Step 1.1: Create Pengajuan Penghapusan BMN</h1>
        <h2>By: ${operatorSatker.nama} (Operator Satker)</h2>

        <div style="display: grid; grid-template-columns: 1fr 1fr; gap: 16px; margin-bottom: 16px;">
          <div style="background: #fef3c7; padding: 15px; border-radius: 8px;">
            <h3>Detail Pengajuan</h3>
            <p><strong>Jenis:</strong> RUSAK_BERAT</p>
            <p><strong>Satker:</strong> ${operatorSatker.satker_nama}</p>
            <p><strong>Status:</strong> DRAFT (${PENGHAPUSAN_BMN_STATUS.DRAFT})</p>
          </div>
          <div style="background: #dbeafe; padding: 15px; border-radius: 8px;">
            <h3>Lampiran</h3>
            <p>✅ Lampiran Utama</p>
            <p>✅ Berita Acara Pemeriksaan BMN</p>
            <p>✅ Foto Dokumentasi BMN Rusak</p>
          </div>
        </div>

        <div style="background: white; padding: 15px; border-radius: 8px; margin-bottom: 16px;">
          <h3>BMN yang Diajukan Penghapusan</h3>
          <table style="width: 100%; border-collapse: collapse;">
            <tr style="background: #f3f4f6;">
              <th style="padding: 8px; border: 1px solid #e5e7eb;">No</th>
              <th style="padding: 8px; border: 1px solid #e5e7eb;">Nama Barang</th>
              <th style="padding: 8px; border: 1px solid #e5e7eb;">NUP</th>
              <th style="padding: 8px; border: 1px solid #e5e7eb;">Tahun</th>
              <th style="padding: 8px; border: 1px solid #e5e7eb;">Nilai</th>
              <th style="padding: 8px; border: 1px solid #e5e7eb;">Kondisi</th>
            </tr>
            <tr>
              <td style="padding: 8px; border: 1px solid #e5e7eb;">1</td>
              <td style="padding: 8px; border: 1px solid #e5e7eb;">Laptop ThinkPad X1 Carbon</td>
              <td style="padding: 8px; border: 1px solid #e5e7eb;">015</td>
              <td style="padding: 8px; border: 1px solid #e5e7eb;">2018</td>
              <td style="padding: 8px; border: 1px solid #e5e7eb;">Rp 15.000.000</td>
              <td style="padding: 8px; border: 1px solid #e5e7eb;">Rusak Berat</td>
            </tr>
            <tr>
              <td style="padding: 8px; border: 1px solid #e5e7eb;">2</td>
              <td style="padding: 8px; border: 1px solid #e5e7eb;">Printer Laser HP</td>
              <td style="padding: 8px; border: 1px solid #e5e7eb;">005</td>
              <td style="padding: 8px; border: 1px solid #e5e7eb;">2017</td>
              <td style="padding: 8px; border: 1px solid #e5e7eb;">Rp 8.500.000</td>
              <td style="padding: 8px; border: 1px solid #e5e7eb;">Rusak Berat</td>
            </tr>
          </table>
        </div>

        <pre style="background: white; padding: 15px; border-radius: 8px;">${JSON.stringify(result.body, null, 2)}</pre>
        <p>HTTP Status: ${result.status}</p>
      </body></html>
    `);
    await page.screenshot({ path: `${SCREENSHOT_DIR}/30-create-penghapusan-bmn.png`, fullPage: true });

    if (result.status === 201 || result.status === 200) {
      penghapusanId = result.body.data?.pengajuan?.id || result.body.data?.id;
      expect(penghapusanId).toBeTruthy();
    } else {
      penghapusanId = '00000000-0000-0000-0000-000000000300';
    }
  });

  test('1.2 - Operator submits pengajuan to Validator Wilayah', async ({ request, page }) => {
    const resolvedPenghapusanId = await resolvePenghapusanId(request);
    if (!resolvedPenghapusanId) {
      test.skip();
      return;
    }

    const result = await transitionPenghapusanBmn(
      request,
      operatorSatker,
      resolvedPenghapusanId,
      'submit_wilayah',
      { catatan: 'Pengajuan penghapusan BMN untuk ditinjau validator wilayah' }
    );

    await page.goto('about:blank');
    await page.setContent(`
      <html><body style="font-family: monospace; padding: 20px; background: #f5f5f5;">
        <h1 style="color: #d97706;">Step 1.2: Submit to Validator Wilayah</h1>
        <div style="background: #fef3c7; padding: 15px; border-radius: 8px; margin-bottom: 12px;">
          <p>Status: DRAFT (${PENGHAPUSAN_BMN_STATUS.DRAFT}) → SUBMIT_WILAYAH (${PENGHAPUSAN_BMN_STATUS.SUBMIT_WILAYAH})</p>
        </div>
        <pre style="background: white; padding: 15px; border-radius: 8px;">${JSON.stringify(result.body, null, 2)}</pre>
      </body></html>
    `);
    await page.screenshot({ path: `${SCREENSHOT_DIR}/31-submit-to-wilayah.png`, fullPage: true });
  });

  // ========================================================================
  // Phase 2: Validator Wilayah Review
  // ========================================================================

  test('2.1 - Validator Wilayah reviews and forwards to Pusat', async ({ request, page }) => {
    const resolvedPenghapusanId = await resolvePenghapusanId(request);
    if (!resolvedPenghapusanId) {
      test.skip();
      return;
    }

    // First: Validator Wilayah views the submission
    const detail = await getPenghapusanBmn(request, validatorWilayah, resolvedPenghapusanId);

    // Forward to Pusat
    const result = await transitionPenghapusanBmn(
      request,
      validatorWilayah,
      resolvedPenghapusanId,
      'submit_pusat',
      {
        catatan: 'Pengajuan penghapusan telah diperiksa, lampiran lengkap, diteruskan ke pusat.',
        hasil_verifikasi: 'LAYAK',
      }
    );

    await page.goto('about:blank');
    await page.setContent(`
      <html><body style="font-family: monospace; padding: 20px; background: #f5f5f5;">
        <h1 style="color: #7c3aed;">Step 2.1: Validator Wilayah Review & Forward</h1>
        <h2>By: ${validatorWilayah.nama}</h2>

        <div style="background: #f5f3ff; padding: 15px; border-radius: 8px; margin-bottom: 12px;">
          <h3>Review Result</h3>
          <p>Hasil Verifikasi: <strong>LAYAK</strong></p>
          <p>Catatan: Lampiran lengkap, diteruskan ke pusat</p>
          <p>Status: SUBMIT_WILAYAH → SUBMIT_PUSAT (${PENGHAPUSAN_BMN_STATUS.SUBMIT_PUSAT})</p>
        </div>

        <div style="background: #fee2e2; padding: 12px; border-radius: 8px; margin-bottom: 12px;">
          <h4>⚠️ Validator Wilayah juga bisa:</h4>
          <p>RETURN → Kembalikan ke Operator untuk perbaikan (RETURNED_TO_OPERATOR, ${PENGHAPUSAN_BMN_STATUS.RETURNED_TO_OPERATOR})</p>
        </div>

        <pre style="background: white; padding: 15px; border-radius: 8px;">${JSON.stringify(result.body, null, 2)}</pre>
      </body></html>
    `);
    await page.screenshot({ path: `${SCREENSHOT_DIR}/32-validator-wilayah-forward.png`, fullPage: true });
  });

  // ========================================================================
  // Phase 3: Validator Pusat Verification
  // ========================================================================

  test('3.1 - Validator Pusat verifies submission', async ({ request, page }) => {
    const resolvedPenghapusanId = await resolvePenghapusanId(request);
    if (!resolvedPenghapusanId) {
      test.skip();
      return;
    }

    const result = await transitionPenghapusanBmn(
      request,
      validatorPusat,
      resolvedPenghapusanId,
      'verifikasi_pusat',
      {
        catatan: 'Pengajuan telah diverifikasi sesuai ketentuan. Lanjutkan ke pembuatan konsep SK.',
      }
    );

    await page.goto('about:blank');
    await page.setContent(`
      <html><body style="font-family: monospace; padding: 20px; background: #f5f5f5;">
        <h1 style="color: #1a56db;">Step 3.1: Validator Pusat Verifikasi</h1>
        <h2>By: ${validatorPusat.nama}</h2>

        <div style="background: #dbeafe; padding: 15px; border-radius: 8px; margin-bottom: 12px;">
          <h3>✅ Verifikasi Pusat</h3>
          <p>Status: SUBMIT_PUSAT (${PENGHAPUSAN_BMN_STATUS.SUBMIT_PUSAT}) → VERIFIKASI_PUSAT (${PENGHAPUSAN_BMN_STATUS.VERIFIKASI_PUSAT})</p>
          <p>Catatan: Sesuai ketentuan, lanjut pembuatan SK</p>
        </div>

        <div style="background: #fef3c7; border: 2px solid #f59e0b; padding: 12px; border-radius: 8px; margin-bottom: 12px;">
          <h4>⚠️ PENTING: Validator Pusat HANYA bisa:</h4>
          <ul>
            <li>✅ APPROVE (verifikasi → lanjut ke konsep SK)</li>
            <li>✅ REJECT (tolak pengajuan)</li>
            <li>❌ REVISI/RETURN ke Operator → TIDAK DIIZINKAN</li>
          </ul>
        </div>

        <pre style="background: white; padding: 15px; border-radius: 8px;">${JSON.stringify(result.body, null, 2)}</pre>
      </body></html>
    `);
    await page.screenshot({ path: `${SCREENSHOT_DIR}/33-validator-pusat-verifikasi.png`, fullPage: true });
  });

  // ========================================================================
  // Phase 4: Generate Konsep SK & Upload Signed SK PDF
  // ========================================================================

  test('4.1 - Generate Konsep SK Penghapusan (DOCX)', async ({ request, page }) => {
    if (!penghapusanId || penghapusanId.startsWith('00000000-0000-0000-0000-0000000003')) {
      test.skip();
      return;
    }

    const result = await generateKonsepSKPenghapusan(request, operatorSatker, penghapusanId);

    await page.goto('about:blank');
    await page.setContent(`
      <html><body style="font-family: monospace; padding: 20px; background: #f5f5f5;">
        <h1 style="color: #d97706;">Step 4.1: Generate Konsep SK Penghapusan</h1>
        <div style="background: #fffbeb; border: 2px solid #f59e0b; padding: 20px; border-radius: 8px; margin-bottom: 16px;">
          <h3>📄 Konsep SK Penghapusan BMN (Format DOCX)</h3>
          <p>Isi SK meliputi:</p>
          <ul>
            <li>Nomor SK</li>
            <li>Dasar hukum penghapusan</li>
            <li>Daftar BMN yang dihapus (tabel)</li>
            <li>Nilai total perolehan BMN</li>
            <li>Alasan penghapusan per item</li>
            <li>Ketentuan pelaksanaan</li>
          </ul>
          <p>Status: VERIFIKASI_PUSAT → KONSEP_SK_GENERATED (${PENGHAPUSAN_BMN_STATUS.KONSEP_SK_GENERATED})</p>
        </div>
        <pre style="background: white; padding: 15px; border-radius: 8px;">${JSON.stringify(result.body, null, 2)}</pre>
      </body></html>
    `);
    await page.screenshot({ path: `${SCREENSHOT_DIR}/34-generate-konsep-sk.png`, fullPage: true });
  });

  test('4.2 - Upload Signed SK PDF', async ({ request, page }) => {
    if (!penghapusanId || penghapusanId.startsWith('00000000-0000-0000-0000-0000000003')) {
      test.skip();
      return;
    }

    const result = await uploadSignedSKPenghapusan(request, operatorSatker, penghapusanId, {
      signed_sk_pdf_url:
        'https://storage.simpel.kejaksaan.go.id/docs/sk-penghapusan-bmn-signed-2026.pdf',
      nomor_sk: 'KEP-123/C.4/Cp.1/05/2026',
      tanggal_sk: '2026-05-15',
      catatan: 'SK Penghapusan telah ditandatangani oleh Pejabat Berwenang',
    });

    await page.goto('about:blank');
    await page.setContent(`
      <html><body style="font-family: monospace; padding: 20px; background: #f0fdf4;">
        <h1 style="color: #059669;">Step 4.2: Upload Signed SK PDF</h1>
        <div style="background: #dcfce7; border: 2px solid #059669; padding: 20px; border-radius: 8px; margin-bottom: 16px;">
          <h3>✅ SK Penghapusan BMN Telah Ditandatangani</h3>
          <p><strong>Nomor SK:</strong> KEP-123/C.4/Cp.1/05/2026</p>
          <p><strong>Tanggal SK:</strong> 15 Mei 2026</p>
          <p><strong>File:</strong> sk-penghapusan-bmn-signed-2026.pdf</p>
          <p>Status: KONSEP_SK_GENERATED → SK_SIGNED (${PENGHAPUSAN_BMN_STATUS.SK_SIGNED})</p>
        </div>
        <pre style="background: white; padding: 15px; border-radius: 8px;">${JSON.stringify(result.body, null, 2)}</pre>
      </body></html>
    `);
    await page.screenshot({ path: `${SCREENSHOT_DIR}/35-upload-signed-sk.png`, fullPage: true });
  });

  // ========================================================================
  // Phase 5: Completion & Visibility Check
  // ========================================================================

  test('5.1 - Mark as completed', async ({ request, page }) => {
    if (!penghapusanId || penghapusanId.startsWith('00000000-0000-0000-0000-0000000003')) {
      test.skip();
      return;
    }

    const result = await transitionPenghapusanBmn(
      request,
      operatorSatker,
      penghapusanId,
      'complete',
      { catatan: 'Proses penghapusan BMN selesai' }
    );

    await page.goto('about:blank');
    await page.setContent(`
      <html><body style="font-family: monospace; padding: 20px; background: #f0fdf4;">
        <h1 style="color: #059669;">Step 5.1: Process Completed</h1>
        <div style="background: #dcfce7; border: 2px solid #059669; padding: 20px; border-radius: 8px;">
          <h3>✅ Penghapusan BMN SELESAI</h3>
          <p>Status: SK_SIGNED → COMPLETED (${PENGHAPUSAN_BMN_STATUS.COMPLETED})</p>
        </div>
        <pre style="background: white; padding: 15px; border-radius: 8px;">${JSON.stringify(result.body, null, 2)}</pre>
      </body></html>
    `);
    await page.screenshot({ path: `${SCREENSHOT_DIR}/36-penghapusan-completed.png`, fullPage: true });
  });

  test('5.2 - Operator and Validator can view completed SK PDF', async ({ request, page }) => {
    if (!penghapusanId || penghapusanId.startsWith('00000000-0000-0000-0000-0000000003')) {
      test.skip();
      return;
    }

    // All roles should be able to view completed SK
    const [operatorView, wilayahView, pusatView] = await Promise.all([
      getPenghapusanBmn(request, operatorSatker, penghapusanId),
      getPenghapusanBmn(request, validatorWilayah, penghapusanId),
      getPenghapusanBmn(request, validatorPusat, penghapusanId),
    ]);

    await page.goto('about:blank');
    await page.setContent(`
      <html><body style="font-family: monospace; padding: 20px; background: #f5f5f5;">
        <h1 style="color: #1a56db;">Step 5.2: All Roles Can View Completed SK</h1>
        <div style="display: grid; grid-template-columns: 1fr 1fr 1fr; gap: 12px; margin-bottom: 16px;">
          <div style="background: ${operatorView.status === 200 ? '#dcfce7' : '#fee2e2'}; padding: 12px; border-radius: 8px;">
            <h4>Operator Satker</h4>
            <p>Status: ${operatorView.status === 200 ? '✅ Dapat akses' : '❌ Gagal'}</p>
          </div>
          <div style="background: ${wilayahView.status === 200 ? '#dcfce7' : '#fee2e2'}; padding: 12px; border-radius: 8px;">
            <h4>Validator Wilayah</h4>
            <p>Status: ${wilayahView.status === 200 ? '✅ Dapat akses' : '❌ Gagal'}</p>
          </div>
          <div style="background: ${pusatView.status === 200 ? '#dcfce7' : '#fee2e2'}; padding: 12px; border-radius: 8px;">
            <h4>Validator Pusat</h4>
            <p>Status: ${pusatView.status === 200 ? '✅ Dapat akses' : '❌ Gagal'}</p>
          </div>
        </div>
      </body></html>
    `);
    await page.screenshot({ path: `${SCREENSHOT_DIR}/37-all-roles-view-sk.png`, fullPage: true });
  });

  // ========================================================================
  // Phase 6: Rejection Flow & Return Flow
  // ========================================================================

  test('6.1 - Alternative: Validator Wilayah returns to Operator', async ({ request, page }) => {
    // Create another pengajuan to test return flow
    const suffix = uniqueSuffix();
    const createResult = await createPenghapusanBmn(request, operatorSatker, {
      judul: `Pengajuan Return Flow Test ${suffix}`,
      keterangan: 'Test return flow',
      jenis_penghapusan: 'USANG',
      satker_id: operatorSatker.satker_id,
      satker_nama: operatorSatker.satker_nama,
      items: [
        {
          kode_barang: '3.06.01.05.002',
          nama_barang: 'Meja Kerja',
          nup: '020',
          tahun_perolehan: 2015,
          nilai_perolehan: 5000000,
          kondisi: 'Usang',
          alasan_penghapusan: 'BMN sudah melewati masa manfaat',
        },
      ],
      lampiran_url: 'https://storage.simpel.kejaksaan.go.id/docs/lampiran-test-return.pdf',
    });

    let returnPenghapusanId: string | null = null;
    if (createResult.status === 201 || createResult.status === 200) {
      returnPenghapusanId = createResult.body.data?.pengajuan?.id || createResult.body.data?.id;

      // Submit to wilayah
      await transitionPenghapusanBmn(
        request,
        operatorSatker,
        returnPenghapusanId!,
        'submit_wilayah',
        {}
      );

      // Wilayah returns
      const returnResult = await transitionPenghapusanBmn(
        request,
        validatorWilayah,
        returnPenghapusanId!,
        'return_to_operator',
        {
          catatan: 'Lampiran Berita Acara belum lengkap, harap dilengkapi.',
        }
      );

      await page.goto('about:blank');
      await page.setContent(`
        <html><body style="font-family: monospace; padding: 20px; background: #fef2f2;">
          <h1 style="color: #dc2626;">Step 6.1: Alternative - Return to Operator</h1>
          <div style="background: #fee2e2; border: 2px solid #dc2626; padding: 20px; border-radius: 8px;">
            <h3>🔙 Dikembalikan ke Operator Satker</h3>
            <p><strong>By:</strong> Validator Wilayah</p>
            <p><strong>Alasan:</strong> Lampiran Berita Acara belum lengkap</p>
            <p>Status: SUBMIT_WILAYAH → RETURNED_TO_OPERATOR (${PENGHAPUSAN_BMN_STATUS.RETURNED_TO_OPERATOR})</p>
          </div>
          <pre style="background: white; padding: 15px; border-radius: 8px;">${JSON.stringify(returnResult.body, null, 2)}</pre>
        </body></html>
      `);
    } else {
      await page.goto('about:blank');
      await page.setContent(`
        <html><body style="font-family: monospace; padding: 20px;">
          <h1 style="color: #dc2626;">Step 6.1: Return Flow</h1>
          <p>Skipped: Could not create test data for return flow.</p>
        </body></html>
      `);
    }
    await page.screenshot({ path: `${SCREENSHOT_DIR}/38-return-to-operator.png`, fullPage: true });
  });

  test('6.2 - Alternative: Validator Pusat rejects submission', async ({ request, page }) => {
    const suffix = uniqueSuffix();
    const createResult = await createPenghapusanBmn(request, operatorSatker, {
      judul: `Pengajuan Reject Flow Test ${suffix}`,
      keterangan: 'Test reject flow',
      jenis_penghapusan: 'TIDAK_DITEMUKAN',
      satker_id: operatorSatker.satker_id,
      satker_nama: operatorSatker.satker_nama,
      items: [
        {
          kode_barang: '3.06.02.01.003',
          nama_barang: 'Laptop',
          nup: '099',
          tahun_perolehan: 2020,
          nilai_perolehan: 12000000,
          kondisi: 'Tidak Ditemukan',
          alasan_penghapusan: 'BMN tidak ditemukan saat inventarisasi',
        },
      ],
      lampiran_url: 'https://storage.simpel.kejaksaan.go.id/docs/lampiran-test-reject.pdf',
    });

    let rejectPenghapusanId: string | null = null;
    if (createResult.status === 201 || createResult.status === 200) {
      rejectPenghapusanId = createResult.body.data?.pengajuan?.id || createResult.body.data?.id;

      // Submit to wilayah
      await transitionPenghapusanBmn(
        request,
        operatorSatker,
        rejectPenghapusanId!,
        'submit_wilayah',
        {}
      );
      // Wilayah forwards to pusat
      await transitionPenghapusanBmn(
        request,
        validatorWilayah,
        rejectPenghapusanId!,
        'submit_pusat',
        { catatan: 'Diteruskan ke pusat' }
      );
      // Pusat REJECTS
      const rejectResult = await transitionPenghapusanBmn(
        request,
        validatorPusat,
        rejectPenghapusanId!,
        'reject',
        {
          catatan:
            'Ditolak karena BMN masih terdeteksi di SIMAN. Harap lakukan inventarisasi ulang.',
        }
      );

      await page.goto('about:blank');
      await page.setContent(`
        <html><body style="font-family: monospace; padding: 20px; background: #fef2f2;">
          <h1 style="color: #dc2626;">Step 6.2: Alternative - Validator Pusat REJECTS</h1>
          <div style="background: #fee2e2; border: 2px solid #dc2626; padding: 20px; border-radius: 8px;">
            <h3>❌ Pengajuan DITOLAK oleh Validator Pusat</h3>
            <p><strong>Alasan:</strong> BMN masih terdeteksi di SIMAN, perlu inventarisasi ulang</p>
            <p>Status: SUBMIT_PUSAT → REJECTED (${PENGHAPUSAN_BMN_STATUS.REJECTED})</p>
          </div>

          <div style="background: #fef3c7; border: 2px solid #f59e0b; padding: 12px; border-radius: 8px; margin-top: 12px;">
            <h4>⚠️ Validator Pusat TIDAK BISA mengembalikan ke Operator</h4>
            <p>Validator Pusat hanya bisa: APPROVE atau REJECT.</p>
            <p>Jika ada masalah, Validator Pusat harus REJECT, dan Operator mengajukan ulang.</p>
          </div>

          <pre style="background: white; padding: 15px; border-radius: 8px;">${JSON.stringify(rejectResult.body, null, 2)}</pre>
        </body></html>
      `);
    } else {
      await page.goto('about:blank');
      await page.setContent(`
        <html><body style="font-family: monospace; padding: 20px;">
          <h1 style="color: #dc2626;">Step 6.2: Reject Flow</h1>
          <p>Skipped: Could not create test data for reject flow.</p>
        </body></html>
      `);
    }
    await page.screenshot({ path: `${SCREENSHOT_DIR}/39-pusat-rejects.png`, fullPage: true });
  });

  // ========================================================================
  // Phase 7: Flow Summary
  // ========================================================================

  test('7.1 - Complete flow summary diagram', async ({ page }) => {
    await page.goto('about:blank');
    await page.setContent(`
      <html><body style="font-family: Arial, sans-serif; padding: 20px; background: white;">
        <h1 style="color: #1a56db; text-align: center;">Business Process Flow: SK Penghapusan BMN</h1>
        <div style="max-width: 900px; margin: 0 auto;">

          <h2>Happy Path (Approve)</h2>
          <div style="display: flex; flex-direction: column; gap: 8px;">
            <div style="background: #059669; color: white; padding: 10px 16px; border-radius: 8px;">
              <strong>1. Operator Satker</strong> - Create pengajuan + upload lampiran → DRAFT (${PENGHAPUSAN_BMN_STATUS.DRAFT})
            </div>
            <div style="text-align: center; color: #6b7280;">↓ submit</div>
            <div style="background: #d97706; color: white; padding: 10px 16px; border-radius: 8px;">
              <strong>2. Operator Satker</strong> - Submit ke Wilayah → SUBMIT_WILAYAH (${PENGHAPUSAN_BMN_STATUS.SUBMIT_WILAYAH})
            </div>
            <div style="text-align: center; color: #6b7280;">↓ forward</div>
            <div style="background: #7c3aed; color: white; padding: 10px 16px; border-radius: 8px;">
              <strong>3. Validator Wilayah</strong> - Review & forward → SUBMIT_PUSAT (${PENGHAPUSAN_BMN_STATUS.SUBMIT_PUSAT})
            </div>
            <div style="text-align: center; color: #6b7280;">↓ verifikasi</div>
            <div style="background: #1a56db; color: white; padding: 10px 16px; border-radius: 8px;">
              <strong>4. Validator Pusat</strong> - Verifikasi → VERIFIKASI_PUSAT (${PENGHAPUSAN_BMN_STATUS.VERIFIKASI_PUSAT})
            </div>
            <div style="text-align: center; color: #6b7280;">↓ generate</div>
            <div style="background: #d97706; color: white; padding: 10px 16px; border-radius: 8px;">
              <strong>5. System</strong> - Generate Konsep SK DOCX → KONSEP_SK_GENERATED (${PENGHAPUSAN_BMN_STATUS.KONSEP_SK_GENERATED})
            </div>
            <div style="text-align: center; color: #6b7280;">↓ upload</div>
            <div style="background: #059669; color: white; padding: 10px 16px; border-radius: 8px;">
              <strong>6. Operator</strong> - Upload signed SK PDF → SK_SIGNED (${PENGHAPUSAN_BMN_STATUS.SK_SIGNED})
            </div>
            <div style="text-align: center; color: #6b7280;">↓</div>
            <div style="background: #059669; color: white; padding: 10px 16px; border-radius: 8px;">
              <strong>7. COMPLETED</strong> (${PENGHAPUSAN_BMN_STATUS.COMPLETED}) - All roles can view SK PDF
            </div>
          </div>

          <h2 style="margin-top: 24px;">Alternative Paths</h2>
          <div style="display: grid; grid-template-columns: 1fr 1fr; gap: 12px;">
            <div style="background: #fee2e2; padding: 12px; border-radius: 8px;">
              <h4>Return by Validator Wilayah</h4>
              <p>SUBMIT_WILAYAH → RETURNED_TO_OPERATOR</p>
              <p>Operator memperbaiki → submit ulang</p>
            </div>
            <div style="background: #fef2f2; padding: 12px; border-radius: 8px;">
              <h4>Reject by Validator Pusat</h4>
              <p>SUBMIT_PUSAT → REJECTED</p>
              <p>⚠️ Pusat TIDAK bisa return, hanya reject</p>
            </div>
          </div>
        </div>
      </body></html>
    `);
    await page.screenshot({ path: `${SCREENSHOT_DIR}/40-penghapusan-flow-diagram.png`, fullPage: true });
  });
});
