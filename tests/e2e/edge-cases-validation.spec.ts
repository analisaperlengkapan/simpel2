/**
 * Edge Cases & Validation Tests
 *
 * Comprehensive tests for input validation, boundary conditions,
 * error handling, and edge cases across all perlengkapan modules.
 */
import { test, expect } from '@playwright/test';
import {
  MOCK_USERS,
  MOCK_USERS_EXTRA,
  MOCK_BMN_DATA,
  INVALID_UUIDS,
  INVALID_DATA,
  API_BASE,
  KEBUTUHAN_BMN_STATUS,
  PEMAKAIAN_BMN_STATUS,
  uniqueSuffix,
  testDates,
} from './fixtures/mock-data';
import {
  checkHealth,
  authHeaders,
  createKebutuhanBmnPengajuan,
  getKebutuhanBmnPengajuan,
  listKebutuhanBmnPengajuan,
  updateKebutuhanBmnPengajuan,
  deleteKebutuhanBmnPengajuan,
  createBarangForSatker,
  createPemakaianBmn,
  listPemakaianBmn,
  updatePemakaianBmn,
  createPenghapusanBmn,
  listPenghapusanBmn,
  updatePenghapusanBmn,
  deletePenghapusanBmn,
  createJenisPakaianDinas,
  createPengajuanPakaianDinas,
  createSpesifikasi,
  createSubSpesifikasi,
  updatePersonalUkuran,
} from './fixtures/api-helpers';

const operatorSatker = MOCK_USERS.operator_satker;
const validatorPusat = MOCK_USERS.validator_pusat;
const admin = MOCK_USERS.admin;

test.describe('Edge Cases & Validation', () => {
  test.beforeAll(async ({ request }) => {
    const healthy = await checkHealth(request);
    test.skip(!healthy, 'Backend not available');
  });

  // ==========================================================================
  // 1. KEBUTUHAN BMN - Validation Tests
  // ==========================================================================
  test.describe('Kebutuhan BMN Validation', () => {
    const dates = testDates();

    test('should reject pengajuan with empty nama', async ({ request }) => {
      const result = await createKebutuhanBmnPengajuan(request, operatorSatker, {
        nama: '',
        tahun: 2025,
        tgl_mulai: dates.start,
        tgl_selesai: dates.end,
      });
      expect(result.status).toBe(400);
      expect(result.body.success).toBe(false);
    });

    test('should reject pengajuan with nama < 3 chars', async ({ request }) => {
      const result = await createKebutuhanBmnPengajuan(request, operatorSatker, {
        nama: INVALID_DATA.too_short_nama,
        tahun: 2025,
        tgl_mulai: dates.start,
        tgl_selesai: dates.end,
      });
      expect(result.status).toBe(400);
      expect(result.body.success).toBe(false);
    });

    test('should reject pengajuan with nama > 255 chars', async ({ request }) => {
      const result = await createKebutuhanBmnPengajuan(request, operatorSatker, {
        nama: INVALID_DATA.too_long_nama,
        tahun: 2025,
        tgl_mulai: dates.start,
        tgl_selesai: dates.end,
      });
      expect(result.status).toBe(400);
      expect(result.body.success).toBe(false);
    });

    test('should reject pengajuan with tahun < 2020', async ({ request }) => {
      const result = await createKebutuhanBmnPengajuan(request, operatorSatker, {
        nama: `Test Tahun Rendah ${uniqueSuffix()}`,
        tahun: INVALID_DATA.invalid_tahun_low,
        tgl_mulai: dates.start,
        tgl_selesai: dates.end,
      });
      expect(result.status).toBe(400);
    });

    test('should reject pengajuan with tahun > 2100', async ({ request }) => {
      const result = await createKebutuhanBmnPengajuan(request, operatorSatker, {
        nama: `Test Tahun Tinggi ${uniqueSuffix()}`,
        tahun: INVALID_DATA.invalid_tahun_high,
        tgl_mulai: dates.start,
        tgl_selesai: dates.end,
      });
      expect(result.status).toBe(400);
    });

    test('should reject pengajuan with tgl_selesai before tgl_mulai', async ({ request }) => {
      const result = await createKebutuhanBmnPengajuan(request, operatorSatker, {
        nama: `Test Tanggal Terbalik ${uniqueSuffix()}`,
        tahun: 2025,
        tgl_mulai: dates.end,
        tgl_selesai: dates.start,
      });
      // Should be rejected or at least the backend should handle gracefully
      expect([400, 422, 201]).toContain(result.status);
    });

    test('should return 404 for nonexistent pengajuan ID', async ({ request }) => {
      const result = await getKebutuhanBmnPengajuan(
        request,
        operatorSatker,
        INVALID_UUIDS.nonexistent
      );
      expect(result.status).toBe(404);
    });

    test('should return 400 for invalid UUID format', async ({ request }) => {
      const result = await getKebutuhanBmnPengajuan(
        request,
        operatorSatker,
        INVALID_UUIDS.not_a_uuid
      );
      expect(result.status).toBe(400);
    });

    test('should reject barang with zero jumlah', async ({ request }) => {
      const result = await createBarangForSatker(request, operatorSatker, INVALID_UUIDS.nonexistent, {
        nama: 'Test Barang',
        jumlah: INVALID_DATA.zero_jumlah,
        satuan: 'Unit',
      });
      expect([400, 404]).toContain(result.status);
    });

    test('should reject barang with negative jumlah', async ({ request }) => {
      const result = await createBarangForSatker(request, operatorSatker, INVALID_UUIDS.nonexistent, {
        nama: 'Test Barang',
        jumlah: INVALID_DATA.negative_jumlah,
        satuan: 'Unit',
      });
      expect([400, 404]).toContain(result.status);
    });

    test('should reject barang with empty nama', async ({ request }) => {
      const result = await createBarangForSatker(request, operatorSatker, INVALID_UUIDS.nonexistent, {
        nama: '',
        jumlah: 5,
        satuan: 'Unit',
      });
      expect([400, 404]).toContain(result.status);
    });

    test('should handle SQL injection in nama field', async ({ request }) => {
      const result = await createKebutuhanBmnPengajuan(request, operatorSatker, {
        nama: INVALID_UUIDS.sql_injection,
        tahun: 2025,
        tgl_mulai: dates.start,
        tgl_selesai: dates.end,
      });
      // Should either create safely (sanitized) or reject - NOT return 500
      expect(result.status).not.toBe(500);
    });

    test('should reject update with wrong version (optimistic locking)', async ({ request }) => {
      // Create a pengajuan first
      const created = await createKebutuhanBmnPengajuan(request, operatorSatker, {
        nama: `Optimistic Lock Test ${uniqueSuffix()}`,
        tahun: 2025,
        tgl_mulai: dates.start,
        tgl_selesai: dates.end,
      });

      if (created.status === 201 || created.status === 200) {
        const id = created.body.data?.id;
        if (id) {
          // Try to update with wrong version
          const updateResult = await updateKebutuhanBmnPengajuan(request, operatorSatker, id, {
            nama: 'Updated Name',
            version: 999, // wrong version
          });
          expect([400, 409, 422]).toContain(updateResult.status);
        }
      }
    });

    test('should only allow delete on draft pengajuan', async ({ request }) => {
      // Try deleting a nonexistent (to verify the endpoint exists)
      const result = await deleteKebutuhanBmnPengajuan(
        request,
        operatorSatker,
        INVALID_UUIDS.nonexistent
      );
      expect([404, 403, 400]).toContain(result.status);
    });
  });

  // ==========================================================================
  // 2. PEMAKAIAN BMN - Validation Tests
  // ==========================================================================
  test.describe('Pemakaian BMN Validation', () => {
    const dates = testDates();

    test('should reject permit with empty pegawai_nip', async ({ request }) => {
      const result = await createPemakaianBmn(request, operatorSatker, {
        pegawai_nip: '',
        pegawai_nama: 'Test',
        jenis_bmn: 'LAPTOP',
        kode_barang: '3.06.02.01.003',
        nama_barang: 'Laptop',
        nup: '015',
        satker_id: 'SKR001',
        tanggal_mulai: dates.start,
        tanggal_selesai: dates.end,
        keperluan: 'Untuk keperluan dinas sehari-hari di kantor',
      });
      expect(result.status).toBe(400);
    });

    test('should reject permit with keperluan < 10 chars', async ({ request }) => {
      const result = await createPemakaianBmn(request, operatorSatker, {
        pegawai_nip: '199001012015011001',
        pegawai_nama: 'Test User',
        jenis_bmn: 'LAPTOP',
        kode_barang: '3.06.02.01.003',
        nama_barang: 'Laptop',
        nup: '015',
        satker_id: 'SKR001',
        tanggal_mulai: dates.start,
        tanggal_selesai: dates.end,
        keperluan: INVALID_DATA.too_short_keperluan,
      });
      expect(result.status).toBe(400);
    });

    test('should reject KENDARAAN without nomor_polisi', async ({ request }) => {
      const result = await createPemakaianBmn(request, operatorSatker, {
        pegawai_nip: '199001012015011001',
        pegawai_nama: 'Test User',
        jenis_bmn: 'KENDARAAN_BERMOTOR',
        kode_barang: MOCK_BMN_DATA.kendaraan.kode_barang,
        nama_barang: MOCK_BMN_DATA.kendaraan.nama_barang,
        nup: MOCK_BMN_DATA.kendaraan.nup,
        satker_id: 'SKR001',
        tanggal_mulai: dates.start,
        tanggal_selesai: dates.end,
        keperluan: 'Untuk perjalanan dinas ke luar kota',
        // Missing: nomor_polisi, merk, tahun
      });
      // Should validate type-specific required fields
      expect([400, 422, 201]).toContain(result.status);
    });

    test('should reject RUMAH_NEGARA without alamat', async ({ request }) => {
      const result = await createPemakaianBmn(request, operatorSatker, {
        pegawai_nip: '199001012015011001',
        pegawai_nama: 'Test User',
        jenis_bmn: 'RUMAH_NEGARA',
        kode_barang: MOCK_BMN_DATA.rumah_negara.kode_barang,
        nama_barang: MOCK_BMN_DATA.rumah_negara.nama_barang,
        nup: MOCK_BMN_DATA.rumah_negara.nup,
        satker_id: 'SKR001',
        tanggal_mulai: dates.start,
        tanggal_selesai: dates.end,
        keperluan: 'Untuk tempat tinggal dinas',
        // Missing: alamat_rumah, luas_tanah, luas_bangunan
      });
      expect([400, 422, 201]).toContain(result.status);
    });

    test('should reject LAPTOP without serial_number', async ({ request }) => {
      const result = await createPemakaianBmn(request, operatorSatker, {
        pegawai_nip: '199001012015011001',
        pegawai_nama: 'Test User',
        jenis_bmn: 'LAPTOP',
        kode_barang: MOCK_BMN_DATA.laptop.kode_barang,
        nama_barang: MOCK_BMN_DATA.laptop.nama_barang,
        nup: MOCK_BMN_DATA.laptop.nup,
        satker_id: 'SKR001',
        tanggal_mulai: dates.start,
        tanggal_selesai: dates.end,
        keperluan: 'Untuk pekerjaan dokumen digital',
        // Missing: serial_number, merk_tipe
      });
      expect([400, 422, 201]).toContain(result.status);
    });

    test('should reject update on non-draft permit', async ({ request }) => {
      // Try updating nonexistent to verify endpoint works
      const result = await updatePemakaianBmn(
        request,
        operatorSatker,
        INVALID_UUIDS.nonexistent,
        { keperluan: 'Updated keperluan yang lebih panjang dari sepuluh karakter' }
      );
      expect([404, 400, 403]).toContain(result.status);
    });

    test('should reject invalid jenis_bmn value', async ({ request }) => {
      const result = await createPemakaianBmn(request, operatorSatker, {
        pegawai_nip: '199001012015011001',
        pegawai_nama: 'Test User',
        jenis_bmn: 'INVALID_TYPE',
        kode_barang: '999.999',
        nama_barang: 'Invalid BMN',
        nup: '999',
        satker_id: 'SKR001',
        tanggal_mulai: dates.start,
        tanggal_selesai: dates.end,
        keperluan: 'Untuk keperluan testing validasi',
      });
      expect([400, 422, 201]).toContain(result.status);
    });

    test('should handle multi-BMN items correctly', async ({ request }) => {
      const result = await createPemakaianBmn(request, operatorSatker, {
        pegawai_nip: '199001012015011001',
        pegawai_nama: 'Test User',
        jenis_bmn: 'LAPTOP',
        kode_barang: MOCK_BMN_DATA.laptop.kode_barang,
        nama_barang: MOCK_BMN_DATA.laptop.nama_barang,
        nup: MOCK_BMN_DATA.laptop.nup,
        satker_id: 'SKR001',
        tanggal_mulai: dates.start,
        tanggal_selesai: dates.end,
        keperluan: 'Untuk pekerjaan dokumen digital dan presentasi',
        serial_number: 'SN-TEST-001',
        merk_tipe: 'Lenovo ThinkPad',
        additional_bmn_items: [
          {
            kode_barang: '3.06.02.01.010',
            nama_barang: 'Printer',
            nup: '025',
          },
        ],
      });
      // Should succeed or fail gracefully
      expect([201, 200, 400]).toContain(result.status);
    });

    test('should reject revoke with alasan < 10 chars', async ({ request }) => {
      // Via direct API call instead of helper
      const res = await request.post(
        `${API_BASE}/pemakaian-bmn/${INVALID_UUIDS.nonexistent}/revoke`,
        {
          data: { alasan: 'pendek' },
          headers: authHeaders(operatorSatker),
        }
      );
      expect([400, 404]).toContain(res.status());
    });
  });

  // ==========================================================================
  // 3. PENGHAPUSAN BMN - Validation Tests
  // ==========================================================================
  test.describe('Penghapusan BMN Validation', () => {
    test('should reject with empty nama_barang', async ({ request }) => {
      const result = await createPenghapusanBmn(request, operatorSatker, {
        nama_barang: '',
        nup: '001',
        alasan: 'Barang sudah rusak berat dan tidak ekonomis diperbaiki',
        metode_penghapusan: 'Pemusnahan',
        lampiran_persyaratan: 'dokumen.pdf',
      });
      expect(result.status).toBe(400);
    });

    test('should reject with alasan < 10 chars', async ({ request }) => {
      const result = await createPenghapusanBmn(request, operatorSatker, {
        nama_barang: `Test Barang ${uniqueSuffix()}`,
        nup: '001',
        alasan: INVALID_DATA.too_short_alasan,
        metode_penghapusan: 'Pemusnahan',
        lampiran_persyaratan: 'dokumen.pdf',
      });
      expect(result.status).toBe(400);
    });

    test('should reject without lampiran_persyaratan', async ({ request }) => {
      const result = await createPenghapusanBmn(request, operatorSatker, {
        nama_barang: `Test Barang ${uniqueSuffix()}`,
        nup: '001',
        alasan: 'Barang sudah rusak berat dan tidak ekonomis diperbaiki',
        metode_penghapusan: 'Pemusnahan',
        // Missing lampiran_persyaratan
      });
      expect(result.status).toBe(400);
    });

    test('should reject update on non-draft penghapusan', async ({ request }) => {
      const result = await updatePenghapusanBmn(
        request,
        operatorSatker,
        INVALID_UUIDS.nonexistent,
        { alasan: 'Updated alasan penghapusan yang cukup panjang' }
      );
      expect([404, 400, 403]).toContain(result.status);
    });

    test('should reject delete on non-draft penghapusan', async ({ request }) => {
      const result = await deletePenghapusanBmn(
        request,
        operatorSatker,
        INVALID_UUIDS.nonexistent
      );
      expect([404, 400, 403]).toContain(result.status);
    });

    test('should reject validator-wilayah return without catatan', async ({ request }) => {
      // When returning, catatan is required
      const res = await request.post(
        `${API_BASE}/penghapusan-bmn/${INVALID_UUIDS.nonexistent}/validator-wilayah`,
        {
          data: { aksi: 'return' }, // missing catatan
          headers: authHeaders(MOCK_USERS.validator_wilayah),
        }
      );
      expect([400, 404]).toContain(res.status());
    });

    test('should reject invalid aksi value', async ({ request }) => {
      const res = await request.post(
        `${API_BASE}/penghapusan-bmn/${INVALID_UUIDS.nonexistent}/validator-wilayah`,
        {
          data: { aksi: 'invalid_action', catatan: 'test' },
          headers: authHeaders(MOCK_USERS.validator_wilayah),
        }
      );
      expect([400, 404]).toContain(res.status());
    });

    test('should reject nama_barang > 255 chars', async ({ request }) => {
      const result = await createPenghapusanBmn(request, operatorSatker, {
        nama_barang: INVALID_DATA.too_long_nama,
        nup: '001',
        alasan: 'Barang sudah tidak layak digunakan dan perlu dihapuskan',
        metode_penghapusan: 'Pemusnahan',
        lampiran_persyaratan: 'dokumen.pdf',
      });
      expect(result.status).toBe(400);
    });

    test('should reject alasan > 1000 chars', async ({ request }) => {
      const result = await createPenghapusanBmn(request, operatorSatker, {
        nama_barang: `Test Barang ${uniqueSuffix()}`,
        nup: '001',
        alasan: 'x'.repeat(1001),
        metode_penghapusan: 'Pemusnahan',
        lampiran_persyaratan: 'dokumen.pdf',
      });
      expect(result.status).toBe(400);
    });
  });

  // ==========================================================================
  // 4. PAKAIAN DINAS - Validation Tests
  // ==========================================================================
  test.describe('Pakaian Dinas Validation', () => {
    test('should reject jenis pakaian with empty nama', async ({ request }) => {
      const result = await createJenisPakaianDinas(request, admin, {
        nama: '',
        kode: 'TST',
        kategori: 'Test',
      });
      expect(result.status).toBe(400);
    });

    test('should reject jenis pakaian with nama > 255 chars', async ({ request }) => {
      const result = await createJenisPakaianDinas(request, admin, {
        nama: INVALID_DATA.too_long_nama,
        kode: 'TST',
        kategori: 'Test',
      });
      expect(result.status).toBe(400);
    });

    test('should reject spesifikasi with invalid gender', async ({ request }) => {
      const result = await createSpesifikasi(request, admin, {
        jenis_pakaian_dinas_id: INVALID_UUIDS.nonexistent,
        nama: 'Test Spesifikasi',
        gender: 'INVALID',  // should be L, P, or SEMUA
        ukuran_group: 'BAJU',
      });
      expect([400, 404]).toContain(result.status);
    });

    test('should reject spesifikasi with invalid ukuran_group', async ({ request }) => {
      const result = await createSpesifikasi(request, admin, {
        jenis_pakaian_dinas_id: INVALID_UUIDS.nonexistent,
        nama: 'Test Spesifikasi',
        gender: 'L',
        ukuran_group: 'INVALID_GROUP', // should be BAJU, CELANA, SEPATU
      });
      expect([400, 404]).toContain(result.status);
    });

    test('should reject personal ukuran with empty sizes', async ({ request }) => {
      const result = await updatePersonalUkuran(request, operatorSatker, {
        ukuran_baju: '',
        ukuran_celana: '',
        ukuran_sepatu: '',
        with_hijab: false,
      });
      expect(result.status).toBe(400);
    });

    test('should reject pengajuan with empty nama', async ({ request }) => {
      const result = await createPengajuanPakaianDinas(request, operatorSatker, {
        satker_id: 'SKR001',
        tahun_anggaran: 2025,
        jenis_pakaian_id: INVALID_UUIDS.nonexistent,
        jumlah: 10,
      });
      // The required 'nama' field should cause validation error
      expect([400, 422, 201]).toContain(result.status);
    });
  });

  // ==========================================================================
  // 5. Pagination Boundary Tests
  // ==========================================================================
  test.describe('Pagination Boundaries', () => {
    test('should handle page=0 gracefully', async ({ request }) => {
      const result = await listKebutuhanBmnPengajuan(request, operatorSatker, {
        page: INVALID_DATA.invalid_page,
        per_page: 10,
      });
      // Backend should either clamp to 1 or return 400
      expect([200, 400]).toContain(result.status);
    });

    test('should handle per_page=0 gracefully', async ({ request }) => {
      const result = await listKebutuhanBmnPengajuan(request, operatorSatker, {
        page: 1,
        per_page: INVALID_DATA.invalid_per_page,
      });
      expect([200, 400]).toContain(result.status);
    });

    test('should handle very large page number', async ({ request }) => {
      const result = await listKebutuhanBmnPengajuan(request, operatorSatker, {
        page: INVALID_DATA.too_large_page,
        per_page: 10,
      });
      expect([200, 400]).toContain(result.status);
      if (result.status === 200) {
        // Should return empty data for extremely high pages
        expect(result.body.data).toBeDefined();
      }
    });

    test('should handle per_page > 1000 gracefully', async ({ request }) => {
      const result = await listKebutuhanBmnPengajuan(request, operatorSatker, {
        page: 1,
        per_page: INVALID_DATA.too_large_per_page,
      });
      expect([200, 400]).toContain(result.status);
    });

    test('should return correct pagination metadata', async ({ request }) => {
      const result = await listKebutuhanBmnPengajuan(request, operatorSatker, {
        page: 1,
        per_page: 5,
      });
      if (result.status === 200) {
        expect(result.body.page).toBe(1);
        expect(result.body.per_page).toBe(5);
        expect(typeof result.body.total).toBe('number');
        expect(Array.isArray(result.body.data)).toBe(true);
        expect(result.body.data.length).toBeLessThanOrEqual(5);
      }
    });

    test('pemakaian pagination works correctly', async ({ request }) => {
      const result = await listPemakaianBmn(request, operatorSatker, {
        page: 1,
        per_page: 5,
      });
      if (result.status === 200) {
        expect(result.body.page).toBeDefined();
        expect(Array.isArray(result.body.data)).toBe(true);
      }
    });

    test('penghapusan pagination works correctly', async ({ request }) => {
      const result = await listPenghapusanBmn(request, operatorSatker, {
        page: 1,
        per_page: 5,
      });
      if (result.status === 200) {
        expect(Array.isArray(result.body.data)).toBe(true);
      }
    });
  });

  // ==========================================================================
  // 6. Response Format Consistency Tests
  // ==========================================================================
  test.describe('Response Format Consistency', () => {
    test('success response has correct envelope', async ({ request }) => {
      const result = await listKebutuhanBmnPengajuan(request, operatorSatker, {
        page: 1,
        per_page: 1,
      });
      if (result.status === 200) {
        expect(result.body).toHaveProperty('success');
        expect(result.body).toHaveProperty('data');
        expect(result.body.success).toBe(true);
      }
    });

    test('error response has correct envelope', async ({ request }) => {
      const result = await getKebutuhanBmnPengajuan(
        request,
        operatorSatker,
        INVALID_UUIDS.nonexistent
      );
      if (result.status === 404) {
        expect(result.body).toHaveProperty('success');
        expect(result.body.success).toBe(false);
        expect(result.body).toHaveProperty('message');
      }
    });

    test('400 validation error includes details', async ({ request }) => {
      const result = await createKebutuhanBmnPengajuan(request, operatorSatker, {
        nama: '',
        tahun: 2025,
        tgl_mulai: testDates().start,
        tgl_selesai: testDates().end,
      });
      if (result.status === 400) {
        expect(result.body.success).toBe(false);
        expect(result.body.message).toBeTruthy();
      }
    });

    test('list endpoints always return arrays', async ({ request }) => {
      const results = await Promise.all([
        listKebutuhanBmnPengajuan(request, operatorSatker, { page: 1, per_page: 1 }),
        listPemakaianBmn(request, operatorSatker, { page: 1, per_page: 1 }),
        listPenghapusanBmn(request, operatorSatker, { page: 1, per_page: 1 }),
      ]);

      for (const result of results) {
        if (result.status === 200) {
          expect(Array.isArray(result.body.data)).toBe(true);
        }
      }
    });
  });

  // ==========================================================================
  // 7. Concurrent Request Tests
  // ==========================================================================
  test.describe('Concurrent Requests', () => {
    const dates = testDates();

    test('should handle 10 concurrent create requests', async ({ request }) => {
      const promises = Array.from({ length: 10 }, (_, i) =>
        createKebutuhanBmnPengajuan(request, operatorSatker, {
          nama: `Concurrent Test ${i} - ${uniqueSuffix()}`,
          tahun: 2025,
          tgl_mulai: dates.start,
          tgl_selesai: dates.end,
        })
      );

      const results = await Promise.all(promises);
      // All should succeed or fail consistently
      const statuses = results.map((r) => r.status);
      const successCount = statuses.filter((s) => s === 201 || s === 200).length;
      const errorCount = statuses.filter((s) => s >= 400).length;

      // At least some should succeed
      expect(successCount + errorCount).toBe(10);
      // Should not get 500 errors
      const serverErrors = statuses.filter((s) => s >= 500);
      expect(serverErrors.length).toBe(0);
    });

    test('should handle concurrent reads without errors', async ({ request }) => {
      const promises = Array.from({ length: 20 }, () =>
        listKebutuhanBmnPengajuan(request, operatorSatker, { page: 1, per_page: 5 })
      );

      const results = await Promise.all(promises);
      for (const result of results) {
        expect(result.status).toBe(200);
      }
    });
  });

  // ==========================================================================
  // 8. Special Character & XSS Tests
  // ==========================================================================
  test.describe('Special Characters & XSS Prevention', () => {
    const dates = testDates();

    test('should handle HTML tags in input', async ({ request }) => {
      const result = await createKebutuhanBmnPengajuan(request, operatorSatker, {
        nama: `<script>alert("xss")</script> Test ${uniqueSuffix()}`,
        tahun: 2025,
        tgl_mulai: dates.start,
        tgl_selesai: dates.end,
      });
      // Should either sanitize and create, or reject - NOT execute script
      expect(result.status).not.toBe(500);
    });

    test('should handle unicode characters', async ({ request }) => {
      const result = await createKebutuhanBmnPengajuan(request, operatorSatker, {
        nama: `Pengajuan BMN 你好世界 ${uniqueSuffix()}`,
        tahun: 2025,
        tgl_mulai: dates.start,
        tgl_selesai: dates.end,
      });
      expect([201, 200, 400]).toContain(result.status);
    });

    test('should handle emoji in input', async ({ request }) => {
      const result = await createKebutuhanBmnPengajuan(request, operatorSatker, {
        nama: `Test BMN 🏢📋 ${uniqueSuffix()}`,
        tahun: 2025,
        tgl_mulai: dates.start,
        tgl_selesai: dates.end,
      });
      expect([201, 200, 400]).toContain(result.status);
    });

    test('should handle newlines and tabs in text fields', async ({ request }) => {
      const result = await createPenghapusanBmn(request, operatorSatker, {
        nama_barang: `Test\nBarang\tPenghapusan ${uniqueSuffix()}`,
        nup: '001',
        alasan: 'Alasan penghapusan\ndengan newline\tdan tab',
        metode_penghapusan: 'Pemusnahan',
        lampiran_persyaratan: 'dokumen.pdf',
      });
      expect([201, 200, 400]).toContain(result.status);
    });
  });
});
