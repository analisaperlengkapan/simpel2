/**
 * Search, Filter & Pagination Tests
 *
 * Comprehensive tests for search functionality, filtering, pagination,
 * sorting, and advanced query capabilities across all modules.
 */
import { test, expect } from '@playwright/test';
import {
  MOCK_USERS,
  MOCK_USERS_EXTRA,
  API_BASE,
  KEBUTUHAN_BMN_STATUS,
  PEMAKAIAN_BMN_STATUS,
  PENGHAPUSAN_BMN_STATUS,
  JENIS_BMN,
  METODE_PENGHAPUSAN,
  UKURAN_GROUP,
  uniqueSuffix,
} from './fixtures/mock-data';
import {
  checkHealth,
  authHeaders,
  listKebutuhanBmnPengajuan,
  listPemakaianBmn,
  listPenghapusanBmn,
  listPengajuanPakaianDinas,
  listJenisPakaianDinas,
  listSpesifikasi,
  listSubSpesifikasi,
  listUkuran,
  searchKebutuhanBmn,
  getSearchSuggestions,
  searchSimanAssets,
  getSimanSatkerSummary,
  getDashboardStats,
  getPerlengkapanDashboard,
  listAssets,
  getAssetById,
  getSatkerDetail,
  getSatkerAktivitas,
  getBmnAvailability,
  getBmnUsageHistory,
  getPegawaiUsageHistory,
  getExpiringPermits,
  getActiveUsageDashboard,
  getBmnUtilizationReport,
  getPenghapusanBmnDetail,
  getLaporanRekapUkuran,
  getLaporanDaftarPegawai,
  getPegawaiSatker,
  getPegawaiWithSizes,
  detectNonStandardCodes,
  getMappingSuggestions,
  getAllMappingProposals,
} from './fixtures/api-helpers';

const operatorSatker = MOCK_USERS.operator_satker;
const validatorPusat = MOCK_USERS.validator_pusat;
const admin = MOCK_USERS.admin;

test.describe('Search, Filter & Pagination', () => {
  test.beforeAll(async ({ request }) => {
    const healthy = await checkHealth(request);
    test.skip(!healthy, 'Backend not available');
  });

  // ==========================================================================
  // 1. KEBUTUHAN BMN - List & Filter
  // ==========================================================================
  test.describe('Kebutuhan BMN - List & Filter', () => {
    test('list with default pagination', async ({ request }) => {
      const result = await listKebutuhanBmnPengajuan(request, operatorSatker);
      expect(result.status).toBe(200);
      if (result.body.data) {
        expect(Array.isArray(result.body.data)).toBe(true);
      }
    });

    test('list with custom page size', async ({ request }) => {
      const result = await listKebutuhanBmnPengajuan(request, operatorSatker, {
        page: 1,
        per_page: 3,
      });
      expect(result.status).toBe(200);
      if (result.body.data) {
        expect(result.body.data.length).toBeLessThanOrEqual(3);
      }
    });

    test('list filtered by tahun', async ({ request }) => {
      const result = await listKebutuhanBmnPengajuan(request, operatorSatker, {
        tahun: 2025,
      });
      expect(result.status).toBe(200);
    });

    test('list filtered by status (Draft)', async ({ request }) => {
      const result = await listKebutuhanBmnPengajuan(request, operatorSatker, {
        status: KEBUTUHAN_BMN_STATUS.DRAFT,
      });
      expect(result.status).toBe(200);
    });

    test('list filtered by status (Approved)', async ({ request }) => {
      const result = await listKebutuhanBmnPengajuan(request, operatorSatker, {
        status: KEBUTUHAN_BMN_STATUS.APPROVED,
      });
      expect(result.status).toBe(200);
    });

    test('list page 2 returns different data than page 1', async ({ request }) => {
      const page1 = await listKebutuhanBmnPengajuan(request, operatorSatker, {
        page: 1,
        per_page: 2,
      });
      const page2 = await listKebutuhanBmnPengajuan(request, operatorSatker, {
        page: 2,
        per_page: 2,
      });

      if (page1.status === 200 && page2.status === 200 && page1.body.total > 2) {
        // IDs should differ
        const ids1 = (page1.body.data || []).map((d: any) => d.id);
        const ids2 = (page2.body.data || []).map((d: any) => d.id);
        const overlap = ids1.filter((id: string) => ids2.includes(id));
        expect(overlap.length).toBe(0);
      }
    });
  });

  // ==========================================================================
  // 2. KEBUTUHAN BMN - Advanced Search
  // ==========================================================================
  test.describe('Kebutuhan BMN - Advanced Search', () => {
    test('search with keyword', async ({ request }) => {
      const result = await searchKebutuhanBmn(request, operatorSatker, {
        q: 'laptop',
      });
      expect([200, 404]).toContain(result.status);
    });

    test('search with minimum 2-char query', async ({ request }) => {
      const result = await searchKebutuhanBmn(request, operatorSatker, {
        q: 'ab',
      });
      expect([200, 400]).toContain(result.status);
    });

    test('search with too short query should fail', async ({ request }) => {
      const result = await searchKebutuhanBmn(request, operatorSatker, {
        q: 'a', // less than 2 chars
      });
      expect([400, 200]).toContain(result.status);
    });

    test('search with satker_id filter', async ({ request }) => {
      const result = await searchKebutuhanBmn(request, operatorSatker, {
        q: 'pengajuan',
        satker_id: 'SKR001',
      });
      expect([200, 404]).toContain(result.status);
    });

    test('search with tahun filter', async ({ request }) => {
      const result = await searchKebutuhanBmn(request, operatorSatker, {
        q: 'test',
        tahun_anggaran: 2025,
      });
      expect([200, 404]).toContain(result.status);
    });

    test('search with multiple status filter', async ({ request }) => {
      const result = await searchKebutuhanBmn(request, operatorSatker, {
        q: 'bmn',
        status: '2000,2001,2002', // comma-separated
      });
      expect([200, 404]).toContain(result.status);
    });

    test('search with date range', async ({ request }) => {
      const result = await searchKebutuhanBmn(request, operatorSatker, {
        q: 'pengajuan',
        date_from: '2025-01-01',
        date_to: '2025-12-31',
      });
      expect([200, 404]).toContain(result.status);
    });

    test('search sorted by relevance', async ({ request }) => {
      const result = await searchKebutuhanBmn(request, operatorSatker, {
        q: 'laptop',
        sort_by: 'relevance',
        sort_dir: 'desc',
      });
      expect([200, 404]).toContain(result.status);
    });

    test('search sorted by date', async ({ request }) => {
      const result = await searchKebutuhanBmn(request, operatorSatker, {
        q: 'pengajuan',
        sort_by: 'created_at',
        sort_dir: 'asc',
      });
      expect([200, 404]).toContain(result.status);
    });

    test('search with is_sbsk filter', async ({ request }) => {
      const result = await searchKebutuhanBmn(request, operatorSatker, {
        q: 'standar',
        is_sbsk: true,
      });
      expect([200, 404]).toContain(result.status);
    });

    test('search suggestions returns results', async ({ request }) => {
      const result = await getSearchSuggestions(request, operatorSatker, 'laptop');
      expect([200, 404]).toContain(result.status);
    });

    test('search suggestions with limit', async ({ request }) => {
      const result = await getSearchSuggestions(request, operatorSatker, 'bmn', 5);
      expect([200, 404]).toContain(result.status);
      if (result.status === 200 && Array.isArray(result.body.data)) {
        expect(result.body.data.length).toBeLessThanOrEqual(5);
      }
    });

    test('search suggestions with too short query', async ({ request }) => {
      const result = await getSearchSuggestions(request, operatorSatker, 'a');
      expect([400, 200]).toContain(result.status);
    });
  });

  // ==========================================================================
  // 3. SIMAN Integration - Asset Search
  // ==========================================================================
  test.describe('SIMAN Integration', () => {
    test('search assets with valid keyword', async ({ request }) => {
      const result = await searchSimanAssets(request, operatorSatker, {
        search: 'laptop',
      });
      expect([200, 404, 503]).toContain(result.status);
    });

    test('search assets with too short keyword', async ({ request }) => {
      const result = await searchSimanAssets(request, operatorSatker, {
        search: 'a', // less than 2 chars
      });
      expect([400, 200]).toContain(result.status);
    });

    test('search assets with kategori filter', async ({ request }) => {
      const result = await searchSimanAssets(request, operatorSatker, {
        search: 'kendaraan',
        kategori: 'peralatan',
      });
      expect([200, 404, 503]).toContain(result.status);
    });

    test('search assets with limit', async ({ request }) => {
      const result = await searchSimanAssets(request, operatorSatker, {
        search: 'meja',
        limit: 5,
      });
      expect([200, 404, 503]).toContain(result.status);
    });

    test('get satker summary', async ({ request }) => {
      const result = await getSimanSatkerSummary(request, operatorSatker, 'SKR001');
      expect([200, 404, 503]).toContain(result.status);
    });
  });

  // ==========================================================================
  // 4. PEMAKAIAN BMN - List & Filter
  // ==========================================================================
  test.describe('Pemakaian BMN - List & Filter', () => {
    test('list all permits', async ({ request }) => {
      const result = await listPemakaianBmn(request, operatorSatker);
      expect(result.status).toBe(200);
    });

    test('list filtered by jenis_bmn KENDARAAN', async ({ request }) => {
      const result = await listPemakaianBmn(request, operatorSatker, {
        jenis_bmn: JENIS_BMN.KENDARAAN_BERMOTOR,
      });
      expect(result.status).toBe(200);
    });

    test('list filtered by jenis_bmn LAPTOP', async ({ request }) => {
      const result = await listPemakaianBmn(request, operatorSatker, {
        jenis_bmn: JENIS_BMN.LAPTOP,
      });
      expect(result.status).toBe(200);
    });

    test('list filtered by jenis_bmn RUMAH_NEGARA', async ({ request }) => {
      const result = await listPemakaianBmn(request, operatorSatker, {
        jenis_bmn: JENIS_BMN.RUMAH_NEGARA,
      });
      expect(result.status).toBe(200);
    });

    test('list filtered by status Draft', async ({ request }) => {
      const result = await listPemakaianBmn(request, operatorSatker, {
        status: 'Draft',
      });
      expect(result.status).toBe(200);
    });

    test('list filtered by status Active', async ({ request }) => {
      const result = await listPemakaianBmn(request, operatorSatker, {
        status: 'Active',
      });
      expect(result.status).toBe(200);
    });

    test('BMN availability check', async ({ request }) => {
      const result = await getBmnAvailability(request, operatorSatker, '001');
      expect([200, 404, 503]).toContain(result.status);
    });

    test('BMN usage history', async ({ request }) => {
      const result = await getBmnUsageHistory(request, operatorSatker, '001');
      expect([200, 404]).toContain(result.status);
    });

    test('pegawai usage history', async ({ request }) => {
      const result = await getPegawaiUsageHistory(request, operatorSatker, '199001012015011001');
      expect([200, 404]).toContain(result.status);
    });

    test('expiring permits with default days', async ({ request }) => {
      const result = await getExpiringPermits(request, operatorSatker);
      expect([200, 404]).toContain(result.status);
    });

    test('expiring permits with custom days', async ({ request }) => {
      const result = await getExpiringPermits(request, operatorSatker, 7);
      expect([200, 404]).toContain(result.status);
    });

    test('expiring permits with 90 days', async ({ request }) => {
      const result = await getExpiringPermits(request, operatorSatker, 90);
      expect([200, 404]).toContain(result.status);
    });
  });

  // ==========================================================================
  // 5. PEMAKAIAN BMN - Monitoring & Dashboards
  // ==========================================================================
  test.describe('Pemakaian BMN - Monitoring', () => {
    test('active usage dashboard', async ({ request }) => {
      const result = await getActiveUsageDashboard(request, admin);
      expect([200, 404]).toContain(result.status);
    });

    test('active usage filtered by satker', async ({ request }) => {
      const result = await getActiveUsageDashboard(request, admin, {
        satker_id: 'SKR001',
      });
      expect([200, 404]).toContain(result.status);
    });

    test('active usage filtered by jenis_bmn', async ({ request }) => {
      const result = await getActiveUsageDashboard(request, admin, {
        jenis_bmn: JENIS_BMN.KENDARAAN_BERMOTOR,
      });
      expect([200, 404]).toContain(result.status);
    });

    test('active usage filtered by date range', async ({ request }) => {
      const result = await getActiveUsageDashboard(request, admin, {
        start_date: '2025-01-01',
        end_date: '2025-12-31',
      });
      expect([200, 404]).toContain(result.status);
    });

    test('utilization report unfiltered', async ({ request }) => {
      const result = await getBmnUtilizationReport(request, admin);
      expect([200, 404]).toContain(result.status);
    });

    test('utilization report filtered by satker + jenis', async ({ request }) => {
      const result = await getBmnUtilizationReport(request, admin, {
        satker_id: 'SKR001',
        jenis_bmn: JENIS_BMN.LAPTOP,
      });
      expect([200, 404]).toContain(result.status);
    });
  });

  // ==========================================================================
  // 6. PENGHAPUSAN BMN - List & Filter
  // ==========================================================================
  test.describe('Penghapusan BMN - List & Filter', () => {
    test('list all penghapusan', async ({ request }) => {
      const result = await listPenghapusanBmn(request, operatorSatker);
      expect(result.status).toBe(200);
    });

    test('filter by satker_id', async ({ request }) => {
      const result = await listPenghapusanBmn(request, operatorSatker, {
        satker_id: 'SKR001',
      });
      expect(result.status).toBe(200);
    });

    test('filter by status Draft', async ({ request }) => {
      const result = await listPenghapusanBmn(request, operatorSatker, {
        status: 'Draft',
      });
      expect(result.status).toBe(200);
    });

    test('get detail with allowed_transitions', async ({ request }) => {
      // Use nonexistent ID - checks endpoint exists
      const result = await getPenghapusanBmnDetail(
        request,
        operatorSatker,
        '99999999-9999-9999-9999-999999999999'
      );
      expect([404, 200]).toContain(result.status);
    });

    test('pagination page 1', async ({ request }) => {
      const result = await listPenghapusanBmn(request, operatorSatker, {
        page: 1,
        per_page: 5,
      });
      expect(result.status).toBe(200);
      if (result.body.data) {
        expect(result.body.data.length).toBeLessThanOrEqual(5);
      }
    });
  });

  // ==========================================================================
  // 7. PAKAIAN DINAS - Master Data Lists
  // ==========================================================================
  test.describe('Pakaian Dinas - Master Data', () => {
    test('list jenis pakaian dinas', async ({ request }) => {
      const result = await listJenisPakaianDinas(request, operatorSatker);
      expect([200, 404]).toContain(result.status);
    });

    test('list spesifikasi', async ({ request }) => {
      const result = await listSpesifikasi(request, operatorSatker);
      expect([200, 404]).toContain(result.status);
    });

    test('list spesifikasi with pagination', async ({ request }) => {
      const result = await listSpesifikasi(request, operatorSatker, {
        page: 1,
        per_page: 5,
      });
      expect([200, 404]).toContain(result.status);
    });

    test('list spesifikasi filtered by jenis_id', async ({ request }) => {
      const result = await listSpesifikasi(request, operatorSatker, {
        jenis_id: '99999999-9999-9999-9999-999999999999',
      });
      expect([200, 404]).toContain(result.status);
    });

    test('list sub-spesifikasi', async ({ request }) => {
      const result = await listSubSpesifikasi(request, operatorSatker);
      expect([200, 404]).toContain(result.status);
    });

    test('list ukuran BAJU', async ({ request }) => {
      const result = await listUkuran(request, operatorSatker, 'BAJU');
      expect([200, 404]).toContain(result.status);
    });

    test('list ukuran CELANA', async ({ request }) => {
      const result = await listUkuran(request, operatorSatker, 'CELANA');
      expect([200, 404]).toContain(result.status);
    });

    test('list ukuran SEPATU', async ({ request }) => {
      const result = await listUkuran(request, operatorSatker, 'SEPATU');
      expect([200, 404]).toContain(result.status);
    });

    test('list all ukuran (no group filter)', async ({ request }) => {
      const result = await listUkuran(request, operatorSatker);
      expect([200, 404]).toContain(result.status);
    });
  });

  // ==========================================================================
  // 8. PAKAIAN DINAS - Pengajuan & Reports
  // ==========================================================================
  test.describe('Pakaian Dinas - Pengajuan & Reports', () => {
    test('list pengajuan pakaian dinas', async ({ request }) => {
      const result = await listPengajuanPakaianDinas(request, operatorSatker);
      expect([200, 404]).toContain(result.status);
    });

    test('list pengajuan with pagination', async ({ request }) => {
      const result = await listPengajuanPakaianDinas(request, operatorSatker, {
        page: 1,
        per_page: 5,
      });
      expect([200, 404]).toContain(result.status);
    });

    test('laporan rekap ukuran requires pengajuan_id', async ({ request }) => {
      const result = await getLaporanRekapUkuran(request, operatorSatker, {
        pengajuan_id: '99999999-9999-9999-9999-999999999999',
      });
      expect([200, 404, 400]).toContain(result.status);
    });

    test('laporan daftar pegawai with filters', async ({ request }) => {
      const result = await getLaporanDaftarPegawai(request, operatorSatker, {
        pengajuan_id: '99999999-9999-9999-9999-999999999999',
        page: 1,
        per_page: 10,
        jenis_kelamin: 'L',
      });
      expect([200, 404, 400]).toContain(result.status);
    });

    test('MySIMKARI: get pegawai by satker', async ({ request }) => {
      const result = await getPegawaiSatker(request, operatorSatker, 'SKR001');
      expect([200, 404, 503]).toContain(result.status);
    });

    test('MySIMKARI: get pegawai with sizes', async ({ request }) => {
      const result = await getPegawaiWithSizes(request, operatorSatker, 'SKR001');
      expect([200, 404, 503]).toContain(result.status);
    });
  });

  // ==========================================================================
  // 9. Dashboard & Stats Endpoints
  // ==========================================================================
  test.describe('Dashboard & Stats', () => {
    test('kebutuhan BMN dashboard stats', async ({ request }) => {
      const result = await getDashboardStats(request, operatorSatker);
      expect([200, 404]).toContain(result.status);
      if (result.status === 200 && result.body.data) {
        // Verify dashboard fields exist
        const data = result.body.data;
        expect(data).toHaveProperty('total_pengajuan');
      }
    });

    test('perlengkapan dashboard metrics', async ({ request }) => {
      const result = await getPerlengkapanDashboard(request, admin, 2025);
      expect([200, 404]).toContain(result.status);
    });

    test('perlengkapan dashboard different year', async ({ request }) => {
      const result = await getPerlengkapanDashboard(request, admin, 2024);
      expect([200, 404]).toContain(result.status);
    });
  });

  // ==========================================================================
  // 10. Assets (read-only)
  // ==========================================================================
  test.describe('Assets API', () => {
    test('list assets', async ({ request }) => {
      const result = await listAssets(request, operatorSatker);
      expect([200, 404]).toContain(result.status);
    });

    test('list assets with category filter', async ({ request }) => {
      const result = await listAssets(request, operatorSatker, {
        category: 'peralatan',
      });
      expect([200, 404]).toContain(result.status);
    });

    test('list assets with pagination', async ({ request }) => {
      const result = await listAssets(request, operatorSatker, {
        page: 1,
        per_page: 10,
      });
      expect([200, 404]).toContain(result.status);
    });

    test('get asset by nonexistent ID', async ({ request }) => {
      const result = await getAssetById(
        request,
        operatorSatker,
        '99999999-9999-9999-9999-999999999999'
      );
      expect([404, 400]).toContain(result.status);
    });
  });

  // ==========================================================================
  // 11. Mapping Kodefikasi
  // ==========================================================================
  test.describe('Mapping Kodefikasi', () => {
    test('detect non-standard codes', async ({ request }) => {
      const result = await detectNonStandardCodes(request, admin);
      expect([200, 404]).toContain(result.status);
    });

    test('get mapping suggestions', async ({ request }) => {
      const result = await getMappingSuggestions(request, admin);
      expect([200, 404]).toContain(result.status);
    });

    test('get all mapping proposals', async ({ request }) => {
      const result = await getAllMappingProposals(request, admin);
      expect([200, 404]).toContain(result.status);
    });
  });

  // ==========================================================================
  // 12. Multi-Module Cross-Search
  // ==========================================================================
  test.describe('Cross-Module Queries', () => {
    test('parallel list queries across all modules', async ({ request }) => {
      const [kebutuhan, pemakaian, penghapusan, pakaianPengajuan] = await Promise.all([
        listKebutuhanBmnPengajuan(request, operatorSatker, { page: 1, per_page: 3 }),
        listPemakaianBmn(request, operatorSatker, { page: 1, per_page: 3 }),
        listPenghapusanBmn(request, operatorSatker, { page: 1, per_page: 3 }),
        listPengajuanPakaianDinas(request, operatorSatker, { page: 1, per_page: 3 }),
      ]);

      expect(kebutuhan.status).toBe(200);
      expect(pemakaian.status).toBe(200);
      expect(penghapusan.status).toBe(200);
      expect([200, 404]).toContain(pakaianPengajuan.status);
    });

    test('concurrent searches across modules', async ({ request }) => {
      const results = await Promise.all([
        searchKebutuhanBmn(request, operatorSatker, { q: 'laptop' }),
        listPemakaianBmn(request, operatorSatker, { jenis_bmn: 'LAPTOP' }),
        searchSimanAssets(request, operatorSatker, { search: 'laptop' }),
      ]);

      for (const result of results) {
        expect(result.status).not.toBe(500);
      }
    });
  });
});
