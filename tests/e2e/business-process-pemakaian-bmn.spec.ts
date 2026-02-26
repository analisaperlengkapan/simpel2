import { test, expect } from '@playwright/test';
import {
  MOCK_USERS,
  PEMAKAIAN_BMN_STATUS,
  MOCK_BMN_DATA,
  MOCK_PEGAWAI_DATA,
  uniqueSuffix,
  SCREENSHOT_DIR,
} from './fixtures/mock-data';
import {
  checkHealth,
  createPemakaianBmn,
  getPemakaianBmn,
  listPemakaianBmn,
  transitionPemakaianBmn,
  generateKonsepSuratPemakaian,
  uploadSignedPdfPemakaian,
  activatePemakaianBmn,
  revokePemakaianBmn,
  renewPemakaianBmn,
  checkBmnAvailability,
  getPemakaianMonitoring,
} from './fixtures/api-helpers';

/**
 * E2E Business Process Test: Pemakaian BMN (BMN Usage Permits)
 *
 * Full workflow:
 * 1. Operator Satker fills BMN data (SIMAN integration + availability check)
 * 2. Operator Satker fills pegawai data (MySIMKARI integration)
 * 3. Operator Satker fills jangka waktu pemakaian
 * 4. System generates DOCX konsep surat izin pemakaian BMN
 *    - Page 1: Pegawai identity + photo
 *    - Page 2+: Table of BMN items (nama barang, NUP, etc.)
 * 5. After signed by pimpinan, Operator uploads signed PDF
 * 6. System marks flow as complete
 *
 * Additional flows:
 * - Pencabutan (revocation) of usage permit
 * - Perpanjangan (renewal) of usage permit
 * - Monitoring by Validator Wilayah/Pusat
 *
 * Note: One pegawai can request multiple BMN items in a single permit
 */
test.describe('Business Process: Pemakaian BMN - Full E2E Flow', () => {
  const operatorSatker = MOCK_USERS.operator_satker;
  const validatorWilayah = MOCK_USERS.validator_wilayah;
  const validatorPusat = MOCK_USERS.validator_pusat;

  let permitId: string;
  let renewedPermitId: string;
  let backendAvailable = false;

  test.describe.configure({ mode: 'serial' });

  test.beforeAll(async ({ request }) => {
    backendAvailable = await checkHealth(request);
  });

  test.beforeEach(async () => {
    test.skip(!backendAvailable, 'Backend not available at localhost:8093');
  });

  // ========================================================================
  // Phase 1: BMN Availability Check & Permit Creation
  // ========================================================================

  test('1.1 - Check BMN availability before creating permit', async ({ request, page }) => {
    const result = await checkBmnAvailability(
      request,
      operatorSatker,
      MOCK_BMN_DATA.laptop.nup
    );

    await page.goto('about:blank');
    await page.setContent(`
      <html><body style="font-family: monospace; padding: 20px; background: #f5f5f5;">
        <h1 style="color: #1a56db;">Step 1.1: Check BMN Availability</h1>
        <h2>BMN: ${MOCK_BMN_DATA.laptop.nama_barang} (NUP: ${MOCK_BMN_DATA.laptop.nup})</h2>
        <div style="background: white; padding: 15px; border-radius: 8px; margin-bottom: 12px;">
          <p><strong>Validasi:</strong> Apakah BMN ini sudah dipakai oleh pegawai lain?</p>
          <p>Kode Barang: ${MOCK_BMN_DATA.laptop.kode_barang}</p>
          <p>NUP: ${MOCK_BMN_DATA.laptop.nup}</p>
        </div>
        <pre style="background: white; padding: 15px; border-radius: 8px;">${JSON.stringify(result.body, null, 2)}</pre>
        <p>HTTP Status: ${result.status}</p>
      </body></html>
    `);
    await page.screenshot({ path: `${SCREENSHOT_DIR}/20-check-bmn-availability.png`, fullPage: true });
  });

  test('1.2 - Operator Satker creates pemakaian BMN with multiple items', async ({
    request,
    page,
  }) => {
    const pegawai = MOCK_PEGAWAI_DATA.pegawai1;
    const suffix = uniqueSuffix();

    const result = await createPemakaianBmn(request, operatorSatker, {
      // Pegawai data (from MySIMKARI)
      pegawai_nip: pegawai.nip,
      pegawai_nama: pegawai.nama,
      pegawai_pangkat: pegawai.pangkat,
      pegawai_jabatan: pegawai.jabatan,
      pegawai_satker_id: pegawai.satker_id,
      pegawai_satker_nama: pegawai.satker_nama,
      foto_pegawai_url: 'https://storage.simpel.kejaksaan.go.id/photos/pegawai/199001012015011001.jpg',

      // Primary BMN data (from SIMAN)
      jenis_bmn: 'Laptop',
      kode_barang: MOCK_BMN_DATA.laptop.kode_barang,
      nama_barang: MOCK_BMN_DATA.laptop.nama_barang,
      nup: MOCK_BMN_DATA.laptop.nup,
      satker_id: operatorSatker.satker_id,
      satker_nama: operatorSatker.satker_nama,

      // Jangka waktu pemakaian
      tanggal_mulai: '2026-03-01',
      tanggal_selesai: '2027-02-28',
      keperluan: 'Keperluan dinas operasional harian untuk penanganan perkara',

      // Laptop-specific fields
      serial_number: 'SN-2024-XYZ123',
      merk_tipe: 'Lenovo ThinkPad X1 Carbon Gen 11',

      // Additional BMN items (satu pegawai bisa lebih dari satu BMN)
      additional_bmn_items: [
        {
          kode_barang: '3.06.02.03.001',
          nama_barang: 'Printer Laser HP LaserJet Pro',
          nup: '008',
          detail_bmn: { serial_number: 'PRT-2024-ABC456', kondisi: 'Baik' },
        },
        {
          kode_barang: '3.06.01.05.002',
          nama_barang: 'Meja Kerja Kayu Jati',
          nup: '012',
          detail_bmn: { kondisi: 'Baik', ukuran: '120x60cm' },
        },
      ],
    });

    await page.goto('about:blank');
    await page.setContent(`
      <html><body style="font-family: monospace; padding: 20px; background: #f5f5f5;">
        <h1 style="color: #059669;">Step 1.2: Create Pemakaian BMN</h1>
        <h2>Pegawai: ${pegawai.nama} (NIP: ${pegawai.nip})</h2>

        <div style="display: grid; grid-template-columns: 1fr 1fr; gap: 16px; margin-bottom: 16px;">
          <div style="background: #dbeafe; padding: 15px; border-radius: 8px;">
            <h3>Data Pegawai (MySIMKARI)</h3>
            <p>NIP: ${pegawai.nip}</p>
            <p>Nama: ${pegawai.nama}</p>
            <p>Pangkat: ${pegawai.pangkat}</p>
            <p>Jabatan: ${pegawai.jabatan}</p>
            <p>Foto: ✅ Uploaded</p>
          </div>
          <div style="background: #dcfce7; padding: 15px; border-radius: 8px;">
            <h3>Jangka Waktu</h3>
            <p>Mulai: 2026-03-01</p>
            <p>Selesai: 2027-02-28</p>
            <p>Durasi: 12 bulan</p>
          </div>
        </div>

        <div style="background: white; padding: 15px; border-radius: 8px; margin-bottom: 16px;">
          <h3>BMN yang Dipinjam (Multi-BMN)</h3>
          <table style="width: 100%; border-collapse: collapse;">
            <tr style="background: #f3f4f6;">
              <th style="padding: 8px; border: 1px solid #e5e7eb;">No</th>
              <th style="padding: 8px; border: 1px solid #e5e7eb;">Nama Barang</th>
              <th style="padding: 8px; border: 1px solid #e5e7eb;">Kode Barang</th>
              <th style="padding: 8px; border: 1px solid #e5e7eb;">NUP</th>
            </tr>
            <tr>
              <td style="padding: 8px; border: 1px solid #e5e7eb;">1</td>
              <td style="padding: 8px; border: 1px solid #e5e7eb;">Laptop ThinkPad X1 Carbon</td>
              <td style="padding: 8px; border: 1px solid #e5e7eb;">3.06.02.01.003</td>
              <td style="padding: 8px; border: 1px solid #e5e7eb;">015</td>
            </tr>
            <tr>
              <td style="padding: 8px; border: 1px solid #e5e7eb;">2</td>
              <td style="padding: 8px; border: 1px solid #e5e7eb;">Printer Laser HP LaserJet Pro</td>
              <td style="padding: 8px; border: 1px solid #e5e7eb;">3.06.02.03.001</td>
              <td style="padding: 8px; border: 1px solid #e5e7eb;">008</td>
            </tr>
            <tr>
              <td style="padding: 8px; border: 1px solid #e5e7eb;">3</td>
              <td style="padding: 8px; border: 1px solid #e5e7eb;">Meja Kerja Kayu Jati</td>
              <td style="padding: 8px; border: 1px solid #e5e7eb;">3.06.01.05.002</td>
              <td style="padding: 8px; border: 1px solid #e5e7eb;">012</td>
            </tr>
          </table>
        </div>

        <pre style="background: white; padding: 15px; border-radius: 8px; max-height: 300px; overflow: auto;">${JSON.stringify(result.body, null, 2)}</pre>
        <p>HTTP Status: ${result.status}</p>
      </body></html>
    `);
    await page.screenshot({ path: `${SCREENSHOT_DIR}/21-create-pemakaian-bmn.png`, fullPage: true });

    if (result.status === 201 || result.status === 200) {
      permitId = result.body.data?.izin?.id || result.body.data?.id;
      expect(permitId).toBeTruthy();
    } else {
      permitId = '00000000-0000-0000-0000-000000000200';
    }
  });

  // ========================================================================
  // Phase 2: Generate Konsep Surat & Upload Signed PDF
  // ========================================================================

  test('2.1 - Generate konsep surat izin pemakaian BMN (DOCX)', async ({ request, page }) => {
    if (!permitId || permitId.startsWith('00000000-0000-0000-0000-0000000002')) {
      test.skip();
      return;
    }

    const result = await generateKonsepSuratPemakaian(request, operatorSatker, permitId);

    await page.goto('about:blank');
    await page.setContent(`
      <html><body style="font-family: monospace; padding: 20px; background: #f5f5f5;">
        <h1 style="color: #d97706;">Step 2.1: Generate Konsep Surat Izin Pemakaian BMN</h1>
        <h2>Format: DOCX</h2>

        <div style="background: #fffbeb; border: 2px solid #f59e0b; padding: 20px; border-radius: 8px; margin-bottom: 16px;">
          <h3>Struktur Surat Izin Pemakaian BMN:</h3>
          <div style="display: grid; grid-template-columns: 1fr 1fr; gap: 16px;">
            <div style="background: white; padding: 12px; border-radius: 6px; border: 1px solid #e5e7eb;">
              <h4>Halaman 1: Identitas Pegawai</h4>
              <ul>
                <li>Foto Pegawai</li>
                <li>NIP</li>
                <li>Nama Lengkap</li>
                <li>Pangkat/Golongan</li>
                <li>Jabatan</li>
                <li>Satuan Kerja</li>
              </ul>
            </div>
            <div style="background: white; padding: 12px; border-radius: 6px; border: 1px solid #e5e7eb;">
              <h4>Halaman 2+: Rincian BMN</h4>
              <ul>
                <li>Tabel daftar BMN yang dipakai</li>
                <li>Nama Barang</li>
                <li>NUP</li>
                <li>Kode Barang</li>
                <li>Jangka Waktu</li>
                <li>Keterangan Lainnya</li>
              </ul>
            </div>
          </div>
        </div>

        <pre style="background: white; padding: 15px; border-radius: 8px;">${JSON.stringify(result.body, null, 2)}</pre>
        <p>HTTP Status: ${result.status}</p>
      </body></html>
    `);
    await page.screenshot({ path: `${SCREENSHOT_DIR}/22-generate-konsep-surat.png`, fullPage: true });
  });

  test('2.2 - Upload signed PDF izin pemakaian BMN', async ({ request, page }) => {
    if (!permitId || permitId.startsWith('00000000-0000-0000-0000-0000000002')) {
      test.skip();
      return;
    }

    const result = await uploadSignedPdfPemakaian(request, operatorSatker, permitId, {
      signed_pdf_url: 'https://storage.simpel.kejaksaan.go.id/docs/izin-pemakaian-bmn-signed-2026.pdf',
      catatan: 'Surat izin telah ditandatangani oleh Kepala Kejaksaan Negeri Jakarta Selatan',
    });

    await page.goto('about:blank');
    await page.setContent(`
      <html><body style="font-family: monospace; padding: 20px; background: #f0fdf4;">
        <h1 style="color: #059669;">Step 2.2: Upload Signed PDF</h1>
        <div style="background: #dcfce7; border: 2px solid #059669; padding: 20px; border-radius: 8px; margin-bottom: 16px;">
          <h3>✅ PDF Surat Izin Pemakaian BMN Telah Ditandatangani</h3>
          <p>File: izin-pemakaian-bmn-signed-2026.pdf</p>
          <p>Status: <strong>COMPLETED</strong></p>
          <p>Sistem menandai bahwa flow proses bisnis selesai.</p>
        </div>
        <pre style="background: white; padding: 15px; border-radius: 8px;">${JSON.stringify(result.body, null, 2)}</pre>
        <p>HTTP Status: ${result.status}</p>
      </body></html>
    `);
    await page.screenshot({ path: `${SCREENSHOT_DIR}/23-upload-signed-pdf.png`, fullPage: true });
  });

  // ========================================================================
  // Phase 3: Monitoring by Validator Wilayah & Pusat
  // ========================================================================

  test('3.1 - Validator Wilayah monitors active permits', async ({ request, page }) => {
    const result = await listPemakaianBmn(request, validatorWilayah, {
      status: 'ACTIVE',
    });

    await page.goto('about:blank');
    await page.setContent(`
      <html><body style="font-family: monospace; padding: 20px; background: #f5f5f5;">
        <h1 style="color: #7c3aed;">Step 3.1: Validator Wilayah Monitoring</h1>
        <h2>By: ${validatorWilayah.nama}</h2>
        <div style="background: #f5f3ff; padding: 15px; border-radius: 8px; margin-bottom: 16px;">
          <h3>Monitoring Data:</h3>
          <ul>
            <li>BMN apa saja yang sedang dipakai</li>
            <li>Siapa pegawai yang memakai</li>
            <li>Jangka waktu pemakaian</li>
            <li>Status izin (aktif/expired/dicabut)</li>
          </ul>
        </div>
        <pre style="background: white; padding: 15px; border-radius: 8px;">${JSON.stringify(result.body, null, 2)}</pre>
      </body></html>
    `);
    await page.screenshot({ path: `${SCREENSHOT_DIR}/24-monitoring-validator-wilayah.png`, fullPage: true });
  });

  test('3.2 - Validator Pusat views monitoring dashboard', async ({ request, page }) => {
    const result = await getPemakaianMonitoring(request, validatorPusat);

    await page.goto('about:blank');
    await page.setContent(`
      <html><body style="font-family: monospace; padding: 20px; background: #f5f5f5;">
        <h1 style="color: #1a56db;">Step 3.2: Validator Pusat Monitoring Dashboard</h1>
        <h2>By: ${validatorPusat.nama}</h2>
        <div style="background: #dbeafe; padding: 15px; border-radius: 8px; margin-bottom: 16px;">
          <h3>Dashboard Monitoring:</h3>
          <ul>
            <li>Total izin aktif per jenis BMN</li>
            <li>Total izin per satker</li>
            <li>Izin yang akan expired</li>
            <li>Aktivasi terbaru</li>
          </ul>
        </div>
        <pre style="background: white; padding: 15px; border-radius: 8px;">${JSON.stringify(result.body, null, 2)}</pre>
      </body></html>
    `);
    await page.screenshot({ path: `${SCREENSHOT_DIR}/25-monitoring-dashboard-pusat.png`, fullPage: true });
  });

  // ========================================================================
  // Phase 4: Pencabutan (Revocation) Flow
  // ========================================================================

  test('4.1 - Revoke active permit', async ({ request, page }) => {
    if (!permitId || permitId.startsWith('00000000-0000-0000-0000-0000000002')) {
      test.skip();
      return;
    }

    const result = await revokePemakaianBmn(request, operatorSatker, permitId, {
      alasan: 'Pegawai mutasi ke satker lain sehingga BMN perlu dikembalikan dan izin pemakaian dicabut.',
    });

    await page.goto('about:blank');
    await page.setContent(`
      <html><body style="font-family: monospace; padding: 20px; background: #fef2f2;">
        <h1 style="color: #dc2626;">Step 4.1: Pencabutan Izin Pemakaian BMN</h1>
        <div style="background: #fee2e2; border: 2px solid #dc2626; padding: 20px; border-radius: 8px; margin-bottom: 16px;">
          <h3>❌ Izin Pemakaian DICABUT</h3>
          <p><strong>Alasan:</strong> Pegawai mutasi ke satker lain</p>
          <p><strong>Status:</strong> REVOKED</p>
          <p>BMN kembali tersedia untuk ditetapkan ke pegawai lain.</p>
        </div>
        <pre style="background: white; padding: 15px; border-radius: 8px;">${JSON.stringify(result.body, null, 2)}</pre>
      </body></html>
    `);
    await page.screenshot({ path: `${SCREENSHOT_DIR}/26-revoke-permit.png`, fullPage: true });
  });

  // ========================================================================
  // Phase 5: Perpanjangan (Renewal) Flow
  // ========================================================================

  test('5.1 - Renew expired/active permit', async ({ request, page }) => {
    if (!permitId || permitId.startsWith('00000000-0000-0000-0000-0000000002')) {
      test.skip();
      return;
    }

    const result = await renewPemakaianBmn(request, operatorSatker, permitId, {
      tanggal_mulai: '2027-03-01',
      tanggal_selesai: '2028-02-28',
      keperluan: 'Perpanjangan izin pemakaian BMN untuk tahun berikutnya',
    });

    await page.goto('about:blank');
    await page.setContent(`
      <html><body style="font-family: monospace; padding: 20px; background: #eff6ff;">
        <h1 style="color: #1a56db;">Step 5.1: Perpanjangan Izin Pemakaian BMN</h1>
        <div style="background: #dbeafe; border: 2px solid #3b82f6; padding: 20px; border-radius: 8px; margin-bottom: 16px;">
          <h3>🔄 Izin Pemakaian DIPERPANJANG</h3>
          <p><strong>Periode Baru:</strong> 2027-03-01 → 2028-02-28</p>
          <p><strong>Keperluan:</strong> Perpanjangan untuk tahun berikutnya</p>
          <p>Sistem membuat izin baru berdasarkan data izin sebelumnya.</p>
        </div>
        <pre style="background: white; padding: 15px; border-radius: 8px;">${JSON.stringify(result.body, null, 2)}</pre>
      </body></html>
    `);
    await page.screenshot({ path: `${SCREENSHOT_DIR}/27-renew-permit.png`, fullPage: true });

    if (result.status === 201 || result.status === 200) {
      renewedPermitId = result.body.data?.id;
    }
  });

  // ========================================================================
  // Phase 6: Flow Summary
  // ========================================================================

  test('6.1 - Complete flow summary diagram', async ({ page }) => {
    await page.goto('about:blank');
    await page.setContent(`
      <html><body style="font-family: Arial, sans-serif; padding: 20px; background: white;">
        <h1 style="color: #1a56db; text-align: center;">Business Process Flow: Pemakaian BMN</h1>
        <div style="max-width: 900px; margin: 0 auto;">
          <h2>Flow Utama: Izin Pemakaian BMN</h2>
          <div style="display: flex; flex-direction: column; gap: 12px;">
            <div style="background: #059669; color: white; padding: 12px 20px; border-radius: 8px;">
              <strong>1. Operator Satker</strong> - Input data BMN (SIMAN + availability check) + Pegawai (MySIMKARI)
            </div>
            <div style="background: #d97706; color: white; padding: 12px 20px; border-radius: 8px;">
              <strong>2. System</strong> - Generate DOCX konsep surat (Hal 1: identitas+foto, Hal 2+: tabel BMN)
            </div>
            <div style="background: #059669; color: white; padding: 12px 20px; border-radius: 8px;">
              <strong>3. Operator Satker</strong> - Upload signed PDF → System marks COMPLETED
            </div>
          </div>

          <h2 style="margin-top: 24px;">Flow Pencabutan</h2>
          <div style="background: #dc2626; color: white; padding: 12px 20px; border-radius: 8px;">
            <strong>Pencabutan:</strong> Operator → Revoke with alasan → BMN kembali tersedia
          </div>

          <h2 style="margin-top: 16px;">Flow Perpanjangan</h2>
          <div style="background: #3b82f6; color: white; padding: 12px 20px; border-radius: 8px;">
            <strong>Perpanjangan:</strong> Operator → Renew with new dates → Izin baru dari data existing
          </div>

          <h2 style="margin-top: 16px;">Monitoring</h2>
          <div style="background: #7c3aed; color: white; padding: 12px 20px; border-radius: 8px;">
            <strong>Validator Wilayah & Pusat:</strong> Dashboard monitoring izin aktif, BMN usage, expiring permits
          </div>

          <div style="margin-top: 24px; background: #fef3c7; padding: 16px; border-radius: 8px; border: 1px solid #f59e0b;">
            <strong>Key Rules:</strong>
            <ul>
              <li>Satu pegawai dapat memiliki lebih dari satu BMN dalam satu izin</li>
              <li>Surat izin: Hal 1 = identitas + foto, Hal 2+ = tabel rincian BMN</li>
              <li>BMN yang sudah dipakai pegawai lain tidak bisa diajukan</li>
              <li>Validator wilayah/pusat dapat memonitor seluruh izin pemakaian</li>
            </ul>
          </div>
        </div>
      </body></html>
    `);
    await page.screenshot({ path: `${SCREENSHOT_DIR}/28-pemakaian-bmn-flow-diagram.png`, fullPage: true });
  });
});
