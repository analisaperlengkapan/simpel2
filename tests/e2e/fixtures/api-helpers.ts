/**
 * API Helper for E2E Tests
 *
 * Provides typed API call wrappers for all perlengkapan endpoints.
 * Routes requests through the backend REST API at the configured baseURL.
 */
import { APIRequestContext, expect } from '@playwright/test';
import { API_BASE, MockUser } from './mock-data';

// ============================================================================
// Generic API Helpers
// ============================================================================

export interface ApiResponse<T> {
  success: boolean;
  message?: string;
  data: T;
}

export interface PaginatedResponse<T> {
  success: boolean;
  data: T[];
  total: number;
  page: number;
  per_page: number;
}

/**
 * Make an authenticated API request with user context headers
 */
function authHeaders(user: MockUser): Record<string, string> {
  return {
    'X-User-Id': user.id,
    'X-User-NIP': user.nip,
    'X-User-Name': user.nama,
    'X-User-Role': user.role,
    'X-User-Pangkat': user.pangkat,
    'X-User-Jabatan': user.jabatan,
    'X-Satker-Id': user.satker_id,
    'X-Satker-Name': user.satker_nama,
    'Authorization': `Bearer mock-jwt-token-${user.role}`,
  };
}

/**
 * Safely parse JSON from a response, falling back to { error: responseText }
 * when the response body isn't valid JSON.
 */
async function safeJson(res: { text: () => Promise<string>; json: () => Promise<any> }): Promise<any> {
  try {
    return await res.json();
  } catch {
    return { error: await res.text().catch(() => 'Non-JSON response') };
  }
}

// ============================================================================
// Health Check
// ============================================================================

export async function checkHealth(request: APIRequestContext): Promise<boolean> {
  try {
    const res = await request.get('/health');
    return res.status() === 200;
  } catch {
    return false;
  }
}

// ============================================================================
// Kebutuhan BMN API
// ============================================================================

export async function createKebutuhanBmnPengajuan(
  request: APIRequestContext,
  user: MockUser,
  data: {
    nama: string;
    deskripsi?: string;
    tahun: number;
    tgl_mulai: string;
    tgl_selesai: string;
    pilihan_satker?: string;
    satker_ids?: string[];
    asset_types?: Array<{ kode_barang: string; nm_barang: string }>;
  }
) {
  const res = await request.post(`${API_BASE}/kebutuhan-bmn/pengajuan`, {
    data,
    headers: authHeaders(user),
  });
  return { status: res.status(), body: await safeJson(res) };
}

export async function getKebutuhanBmnPengajuan(
  request: APIRequestContext,
  user: MockUser,
  id: string
) {
  const res = await request.get(`${API_BASE}/kebutuhan-bmn/pengajuan/${id}`, {
    headers: authHeaders(user),
  });
  return { status: res.status(), body: await safeJson(res) };
}

export async function listKebutuhanBmnPengajuan(
  request: APIRequestContext,
  user: MockUser,
  params?: { page?: number; per_page?: number; status?: number; tahun?: number }
) {
  const searchParams = new URLSearchParams();
  if (params?.page) searchParams.set('page', String(params.page));
  if (params?.per_page) searchParams.set('per_page', String(params.per_page));
  if (params?.status) searchParams.set('status', String(params.status));
  if (params?.tahun) searchParams.set('tahun', String(params.tahun));

  const res = await request.get(`${API_BASE}/kebutuhan-bmn/pengajuan?${searchParams}`, {
    headers: authHeaders(user),
  });
  return { status: res.status(), body: await safeJson(res) };
}

export async function addSatkerToPengajuan(
  request: APIRequestContext,
  user: MockUser,
  pengajuanId: string,
  satkerData: { satker_id: string; satker_nama?: string }
) {
  const res = await request.post(
    `${API_BASE}/kebutuhan-bmn/pengajuan/${pengajuanId}/satker`,
    { data: satkerData, headers: authHeaders(user) }
  );
  return { status: res.status(), body: await safeJson(res) };
}

export async function createBarangForSatker(
  request: APIRequestContext,
  user: MockUser,
  satkerId: string,
  data: {
    nama: string;
    kode_barang?: string;
    jumlah: number;
    satuan: string;
    alasan?: string;
    keterangan?: string;
  }
) {
  const res = await request.post(
    `${API_BASE}/kebutuhan-bmn/satker/${satkerId}/barang`,
    { data, headers: authHeaders(user) }
  );
  return { status: res.status(), body: await safeJson(res) };
}

export async function submitSatkerToWilayah(
  request: APIRequestContext,
  user: MockUser,
  satkerId: string,
  data: {
    catatan_satker?: string;
    lampiran_surat_permohonan?: string;
    lampiran_pendukung?: string[];
  }
) {
  const res = await request.post(
    `${API_BASE}/kebutuhan-bmn/satker/${satkerId}/submit-wilayah`,
    { data, headers: authHeaders(user) }
  );
  return { status: res.status(), body: await safeJson(res) };
}

export async function validatorWilayahAction(
  request: APIRequestContext,
  user: MockUser,
  satkerId: string,
  data: { action: 'forward' | 'return'; catatan?: string }
) {
  const res = await request.post(
    `${API_BASE}/kebutuhan-bmn/satker/${satkerId}/validator-wilayah`,
    { data, headers: authHeaders(user) }
  );
  return { status: res.status(), body: await safeJson(res) };
}

export async function validatorPusatKeputusan(
  request: APIRequestContext,
  user: MockUser,
  satkerId: string,
  data: { is_approved: boolean; alasan_keputusan: string }
) {
  const res = await request.post(
    `${API_BASE}/kebutuhan-bmn/satker/${satkerId}/validator-pusat`,
    { data, headers: authHeaders(user) }
  );
  return { status: res.status(), body: await safeJson(res) };
}

export async function getAnalisisKelayakan(
  request: APIRequestContext,
  user: MockUser,
  satkerId: string
) {
  const res = await request.get(
    `${API_BASE}/kebutuhan-bmn/satker/${satkerId}/analisis`,
    { headers: authHeaders(user) }
  );
  return { status: res.status(), body: await safeJson(res) };
}

export async function transitionPengajuanStatus(
  request: APIRequestContext,
  user: MockUser,
  pengajuanId: string,
  data: { target_status: number; komentar?: string }
) {
  const res = await request.post(
    `${API_BASE}/kebutuhan-bmn/pengajuan/${pengajuanId}/transition`,
    { data, headers: authHeaders(user) }
  );
  return { status: res.status(), body: await safeJson(res) };
}

export async function getDashboardStats(
  request: APIRequestContext,
  user: MockUser
) {
  const res = await request.get(`${API_BASE}/kebutuhan-bmn/dashboard`, {
    headers: authHeaders(user),
  });
  return { status: res.status(), body: await safeJson(res) };
}

// ============================================================================
// Pemakaian BMN API
// ============================================================================

export async function createPemakaianBmn(
  request: APIRequestContext,
  user: MockUser,
  data: {
    pegawai_nip: string;
    pegawai_nama: string;
    pegawai_pangkat?: string;
    pegawai_jabatan?: string;
    pegawai_satker_id?: string;
    pegawai_satker_nama?: string;
    foto_pegawai_url?: string;
    jenis_bmn: string;
    kode_barang: string;
    nama_barang: string;
    nup: string;
    satker_id: string;
    satker_nama?: string;
    tanggal_mulai: string;
    tanggal_selesai: string;
    keperluan?: string;
    additional_bmn_items?: Array<{
      kode_barang: string;
      nama_barang: string;
      nup: string;
      detail_bmn?: Record<string, unknown>;
    }>;
    // Vehicle-specific
    nomor_polisi?: string;
    nomor_mesin?: string;
    nomor_rangka?: string;
    // Housing-specific
    alamat_rumah?: string;
    tipe_rumah?: string;
    luas_bangunan?: number;
    // Laptop-specific
    serial_number?: string;
    merk_tipe?: string;
  }
) {
  const res = await request.post(`${API_BASE}/pemakaian-bmn`, {
    data,
    headers: authHeaders(user),
  });
  return { status: res.status(), body: await safeJson(res) };
}

export async function getPemakaianBmn(
  request: APIRequestContext,
  user: MockUser,
  id: string
) {
  const res = await request.get(`${API_BASE}/pemakaian-bmn/${id}`, {
    headers: authHeaders(user),
  });
  return { status: res.status(), body: await safeJson(res) };
}

export async function listPemakaianBmn(
  request: APIRequestContext,
  user: MockUser,
  params?: { page?: number; per_page?: number; status?: string; jenis_bmn?: string }
) {
  const searchParams = new URLSearchParams();
  if (params?.page) searchParams.set('page', String(params.page));
  if (params?.per_page) searchParams.set('per_page', String(params.per_page));
  if (params?.status) searchParams.set('status', params.status);
  if (params?.jenis_bmn) searchParams.set('jenis_bmn', params.jenis_bmn);

  const res = await request.get(`${API_BASE}/pemakaian-bmn?${searchParams}`, {
    headers: authHeaders(user),
  });
  return { status: res.status(), body: await safeJson(res) };
}

export async function transitionPemakaianBmn(
  request: APIRequestContext,
  user: MockUser,
  id: string,
  data: { target_status: string; catatan?: string }
) {
  const res = await request.post(`${API_BASE}/pemakaian-bmn/${id}/transition`, {
    data,
    headers: authHeaders(user),
  });
  return { status: res.status(), body: await safeJson(res) };
}

export async function generateKonsepSuratPemakaian(
  request: APIRequestContext,
  user: MockUser,
  id: string
) {
  const res = await request.post(`${API_BASE}/pemakaian-bmn/${id}/generate-konsep-surat`, {
    headers: authHeaders(user),
  });
  return { status: res.status(), body: await safeJson(res) };
}

export async function uploadSignedPdfPemakaian(
  request: APIRequestContext,
  user: MockUser,
  id: string,
  data: { signed_pdf_url: string; catatan?: string }
) {
  const res = await request.post(`${API_BASE}/pemakaian-bmn/${id}/upload-signed-pdf`, {
    data,
    headers: authHeaders(user),
  });
  return { status: res.status(), body: await safeJson(res) };
}

export async function activatePemakaianBmn(
  request: APIRequestContext,
  user: MockUser,
  id: string
) {
  const res = await request.post(`${API_BASE}/pemakaian-bmn/${id}/activate`, {
    headers: authHeaders(user),
  });
  return { status: res.status(), body: await safeJson(res) };
}

export async function revokePemakaianBmn(
  request: APIRequestContext,
  user: MockUser,
  id: string,
  data: { alasan: string }
) {
  const res = await request.post(`${API_BASE}/pemakaian-bmn/${id}/revoke`, {
    data,
    headers: authHeaders(user),
  });
  return { status: res.status(), body: await safeJson(res) };
}

export async function renewPemakaianBmn(
  request: APIRequestContext,
  user: MockUser,
  id: string,
  data: { tanggal_mulai: string; tanggal_selesai: string; keperluan?: string }
) {
  const res = await request.post(`${API_BASE}/pemakaian-bmn/${id}/renew`, {
    data,
    headers: authHeaders(user),
  });
  return { status: res.status(), body: await safeJson(res) };
}

export async function checkBmnAvailability(
  request: APIRequestContext,
  user: MockUser,
  nup: string
) {
  const res = await request.get(
    `${API_BASE}/pemakaian-bmn/bmn/${nup}/availability`,
    { headers: authHeaders(user) }
  );
  return { status: res.status(), body: await safeJson(res) };
}

export async function getPemakaianMonitoring(
  request: APIRequestContext,
  user: MockUser
) {
  const res = await request.get(`${API_BASE}/pemakaian-bmn/monitoring/active-usage`, {
    headers: authHeaders(user),
  });
  return { status: res.status(), body: await safeJson(res) };
}

// ============================================================================
// Penghapusan BMN API
// ============================================================================

export async function createPenghapusanBmn(
  request: APIRequestContext,
  user: MockUser,
  data: Record<string, any>
) {
  const res = await request.post(`${API_BASE}/penghapusan-bmn`, {
    data,
    headers: authHeaders(user),
  });
  return { status: res.status(), body: await safeJson(res) };
}

export async function getPenghapusanBmn(
  request: APIRequestContext,
  user: MockUser,
  id: string
) {
  const res = await request.get(`${API_BASE}/penghapusan-bmn/${id}`, {
    headers: authHeaders(user),
  });
  return { status: res.status(), body: await safeJson(res) };
}

export async function getPenghapusanBmnDetail(
  request: APIRequestContext,
  user: MockUser,
  id: string
) {
  const res = await request.get(`${API_BASE}/penghapusan-bmn/${id}/detail`, {
    headers: authHeaders(user),
  });
  return { status: res.status(), body: await safeJson(res) };
}

export async function listPenghapusanBmn(
  request: APIRequestContext,
  user: MockUser,
  params?: { page?: number; per_page?: number; status?: string; satker_id?: string }
) {
  const searchParams = new URLSearchParams();
  if (params?.page) searchParams.set('page', String(params.page));
  if (params?.per_page) searchParams.set('per_page', String(params.per_page));
  if (params?.status) searchParams.set('status', params.status);
  if (params?.satker_id) searchParams.set('satker_id', params.satker_id);

  const res = await request.get(`${API_BASE}/penghapusan-bmn?${searchParams}`, {
    headers: authHeaders(user),
  });
  return { status: res.status(), body: await safeJson(res) };
}

export async function submitPenghapusanToWilayah(
  request: APIRequestContext,
  user: MockUser,
  id: string,
  data: { catatan?: string }
) {
  const res = await request.post(`${API_BASE}/penghapusan-bmn/${id}/submit-wilayah`, {
    data,
    headers: authHeaders(user),
  });
  return { status: res.status(), body: await safeJson(res) };
}

export async function forwardPenghapusanToPusat(
  request: APIRequestContext,
  user: MockUser,
  id: string,
  data: { catatan?: string }
) {
  const res = await request.post(`${API_BASE}/penghapusan-bmn/${id}/forward-pusat`, {
    data,
    headers: authHeaders(user),
  });
  return { status: res.status(), body: await safeJson(res) };
}

export async function returnPenghapusanToOperator(
  request: APIRequestContext,
  user: MockUser,
  id: string,
  data: { catatan: string }
) {
  const res = await request.post(`${API_BASE}/penghapusan-bmn/${id}/return-operator`, {
    data,
    headers: authHeaders(user),
  });
  return { status: res.status(), body: await safeJson(res) };
}

export async function generateKonsepSk(
  request: APIRequestContext,
  user: MockUser,
  id: string
) {
  const res = await request.post(`${API_BASE}/penghapusan-bmn/${id}/generate-konsep-sk`, {
    headers: authHeaders(user),
  });
  return { status: res.status(), body: await safeJson(res) };
}

export async function uploadSignedSk(
  request: APIRequestContext,
  user: MockUser,
  id: string,
  data: { signed_sk_pdf_url: string; catatan?: string }
) {
  const res = await request.post(`${API_BASE}/penghapusan-bmn/${id}/upload-signed-sk`, {
    data,
    headers: authHeaders(user),
  });
  return { status: res.status(), body: await safeJson(res) };
}

export async function transitionPenghapusanBmn(
  request: APIRequestContext,
  user: MockUser,
  id: string,
  actionOrData: string | { target_status: string; catatan?: string },
  extraData?: Record<string, any>
) {
  // Support both calling styles:
  // transitionPenghapusanBmn(req, user, id, { target_status, catatan })
  // transitionPenghapusanBmn(req, user, id, 'action', { catatan })
  const payload = typeof actionOrData === 'string'
    ? { target_status: actionOrData, ...(extraData || {}) }
    : actionOrData;

  const res = await request.post(`${API_BASE}/penghapusan-bmn/${id}/transition`, {
    data: payload,
    headers: authHeaders(user),
  });
  return { status: res.status(), body: await safeJson(res) };
}

// ============================================================================
// Pakaian Dinas API
// ============================================================================

export async function listJenisPakaianDinas(
  request: APIRequestContext,
  user: MockUser
) {
  const res = await request.get(`${API_BASE}/pakaian-dinas/jenis`, {
    headers: authHeaders(user),
  });
  return { status: res.status(), body: await safeJson(res) };
}

export async function createJenisPakaianDinas(
  request: APIRequestContext,
  user: MockUser,
  data: {
    nama: string;
    kode: string;
    deskripsi?: string;
    kategori: string;
    gender?: string;
  }
) {
  const res = await request.post(`${API_BASE}/pakaian-dinas/jenis`, {
    data,
    headers: authHeaders(user),
  });
  return { status: res.status(), body: await safeJson(res) };
}

export async function createPengajuanPakaianDinas(
  request: APIRequestContext,
  user: MockUser,
  data: {
    satker_id: string;
    tahun_anggaran: number;
    jenis_pakaian_id: string;
    jumlah: number;
    keterangan?: string;
  }
) {
  const res = await request.post(`${API_BASE}/pakaian-dinas/pengajuan`, {
    data,
    headers: authHeaders(user),
  });
  return { status: res.status(), body: await safeJson(res) };
}

export async function listPengajuanPakaianDinas(
  request: APIRequestContext,
  user: MockUser,
  params?: { page?: number; per_page?: number; status?: string }
) {
  const searchParams = new URLSearchParams();
  if (params?.page) searchParams.set('page', String(params.page));
  if (params?.per_page) searchParams.set('per_page', String(params.per_page));
  if (params?.status) searchParams.set('status', params.status);

  const res = await request.get(`${API_BASE}/pakaian-dinas/pengajuan?${searchParams}`, {
    headers: authHeaders(user),
  });
  return { status: res.status(), body: await safeJson(res) };
}

export async function submitPengajuanPakaianDinas(
  request: APIRequestContext,
  user: MockUser,
  id: string,
  data?: { catatan?: string }
) {
  const res = await request.post(`${API_BASE}/pakaian-dinas/pengajuan/${id}/submit`, {
    data: data || {},
    headers: authHeaders(user),
  });
  return { status: res.status(), body: await safeJson(res) };
}

export async function approvePengajuanPakaianDinas(
  request: APIRequestContext,
  user: MockUser,
  id: string,
  data?: { catatan?: string }
) {
  const res = await request.post(`${API_BASE}/pakaian-dinas/pengajuan/${id}/approve`, {
    data: data || {},
    headers: authHeaders(user),
  });
  return { status: res.status(), body: await safeJson(res) };
}

export async function rejectPengajuanPakaianDinas(
  request: APIRequestContext,
  user: MockUser,
  id: string,
  data: { catatan: string }
) {
  const res = await request.post(`${API_BASE}/pakaian-dinas/pengajuan/${id}/reject`, {
    data,
    headers: authHeaders(user),
  });
  return { status: res.status(), body: await safeJson(res) };
}

// ============================================================================
// Export / Report API
// ============================================================================

export async function exportKebutuhanBmnReport(
  request: APIRequestContext,
  user: MockUser,
  pengajuanId: string,
  format: 'pdf' | 'docx' | 'xlsx' = 'pdf'
) {
  const res = await request.post(`${API_BASE}/kebutuhan-bmn/pengajuan/${pengajuanId}/export`, {
    data: { format },
    headers: authHeaders(user),
  });
  return { status: res.status(), body: await safeJson(res) };
}

// ============================================================================
// Aliases for cross-module test compatibility
// ============================================================================

/**
 * Alias: checkHealth that returns { status, body } instead of boolean
 */
export async function checkHealthDetailed(request: APIRequestContext) {
  try {
    const res = await request.get('/health');
    return { status: res.status(), body: await safeJson(res) };
  } catch {
    return { status: 0, body: { error: 'Connection failed' } };
  }
}

/**
 * Alias for createKebutuhanBmnPengajuan (shorter name used in integration tests)
 */
export async function createKebutuhanBmn(
  request: APIRequestContext,
  user: MockUser,
  data: Record<string, any>
) {
  const res = await request.post(`${API_BASE}/kebutuhan-bmn/pengajuan`, {
    data,
    headers: authHeaders(user),
  });
  return { status: res.status(), body: await safeJson(res) };
}

/**
 * Generic kebutuhan BMN workflow transition
 */
export async function transitionKebutuhanBmn(
  request: APIRequestContext,
  user: MockUser,
  id: string,
  action: string,
  data?: Record<string, any>
) {
  const res = await request.post(`${API_BASE}/kebutuhan-bmn/pengajuan/${id}/transition`, {
    data: { action, ...(data || {}) },
    headers: authHeaders(user),
  });
  return { status: res.status(), body: await safeJson(res) };
}

/**
 * Alias for generateKonsepSk (used in penghapusan tests)
 */
export const generateKonsepSKPenghapusan = generateKonsepSk;

/**
 * Alias for uploadSignedSk (used in penghapusan tests)
 */
export async function uploadSignedSKPenghapusan(
  request: APIRequestContext,
  user: MockUser,
  id: string,
  data: Record<string, any>
) {
  const res = await request.post(`${API_BASE}/penghapusan-bmn/${id}/upload-signed-sk`, {
    data,
    headers: authHeaders(user),
  });
  return { status: res.status(), body: await safeJson(res) };
}

/**
 * Export helpers for all modules
 */
export async function exportKebutuhanBmn(
  request: APIRequestContext,
  user: MockUser,
  params?: Record<string, any>
) {
  const searchParams = new URLSearchParams();
  if (params?.format) searchParams.set('format', params.format);
  if (params?.tahun) searchParams.set('tahun', String(params.tahun));
  const res = await request.get(`${API_BASE}/kebutuhan-bmn/export?${searchParams}`, {
    headers: authHeaders(user),
  });
  return { status: res.status(), body: await safeJson(res) };
}

export async function exportPemakaianBmn(
  request: APIRequestContext,
  user: MockUser,
  params?: Record<string, any>
) {
  const searchParams = new URLSearchParams();
  if (params?.format) searchParams.set('format', params.format);
  const res = await request.get(`${API_BASE}/pemakaian-bmn/export?${searchParams}`, {
    headers: authHeaders(user),
  });
  return { status: res.status(), body: await safeJson(res) };
}

export async function exportPenghapusanBmn(
  request: APIRequestContext,
  user: MockUser,
  params?: Record<string, any>
) {
  const searchParams = new URLSearchParams();
  if (params?.format) searchParams.set('format', params.format);
  const res = await request.get(`${API_BASE}/penghapusan-bmn/export?${searchParams}`, {
    headers: authHeaders(user),
  });
  return { status: res.status(), body: await safeJson(res) };
}

/**
 * Export authHeaders for direct use in test files
 */
export { authHeaders };

// ============================================================================
// Dashboard API
// ============================================================================

export async function getPerlengkapanDashboard(
  request: APIRequestContext,
  user: MockUser,
  tahunAnggaran: number
) {
  const res = await request.get(
    `${API_BASE}/dashboard/perlengkapan?tahun_anggaran=${tahunAnggaran}`,
    { headers: authHeaders(user) }
  );
  return { status: res.status(), body: await safeJson(res) };
}

export async function exportDashboardExcel(
  request: APIRequestContext,
  user: MockUser,
  tahunAnggaran: number
) {
  const res = await request.get(
    `${API_BASE}/dashboard/perlengkapan/export/excel?tahun_anggaran=${tahunAnggaran}`,
    { headers: authHeaders(user) }
  );
  return { status: res.status(), headers: res.headers(), body: await res.body() };
}

export async function exportDashboardPdf(
  request: APIRequestContext,
  user: MockUser,
  tahunAnggaran: number
) {
  const res = await request.get(
    `${API_BASE}/dashboard/perlengkapan/export/pdf?tahun_anggaran=${tahunAnggaran}`,
    { headers: authHeaders(user) }
  );
  return { status: res.status(), headers: res.headers(), body: await res.body() };
}

// ============================================================================
// SIMAN Integration API
// ============================================================================

export async function searchSimanAssets(
  request: APIRequestContext,
  user: MockUser,
  params: { search: string; kategori?: string; limit?: number }
) {
  const sp = new URLSearchParams({ search: params.search });
  if (params.kategori) sp.set('kategori', params.kategori);
  if (params.limit) sp.set('limit', String(params.limit));
  const res = await request.get(`${API_BASE}/kebutuhan-bmn/siman/search?${sp}`, {
    headers: authHeaders(user),
  });
  return { status: res.status(), body: await safeJson(res) };
}

export async function getSimanSatkerSummary(
  request: APIRequestContext,
  user: MockUser,
  satkerId: string
) {
  const res = await request.get(
    `${API_BASE}/kebutuhan-bmn/siman/summary/${satkerId}`,
    { headers: authHeaders(user) }
  );
  return { status: res.status(), body: await safeJson(res) };
}

// ============================================================================
// Advanced Search API
// ============================================================================

export async function searchKebutuhanBmn(
  request: APIRequestContext,
  user: MockUser,
  params: {
    q: string;
    page?: number;
    per_page?: number;
    satker_id?: string;
    tahun_anggaran?: number;
    status?: string;
    kode_barang?: string;
    is_sbsk?: boolean;
    date_from?: string;
    date_to?: string;
    sort_by?: string;
    sort_dir?: string;
  }
) {
  const sp = new URLSearchParams({ q: params.q });
  if (params.page) sp.set('page', String(params.page));
  if (params.per_page) sp.set('per_page', String(params.per_page));
  if (params.satker_id) sp.set('satker_id', params.satker_id);
  if (params.tahun_anggaran) sp.set('tahun_anggaran', String(params.tahun_anggaran));
  if (params.status) sp.set('status', params.status);
  if (params.kode_barang) sp.set('kode_barang', params.kode_barang);
  if (params.is_sbsk !== undefined) sp.set('is_sbsk', String(params.is_sbsk));
  if (params.date_from) sp.set('date_from', params.date_from);
  if (params.date_to) sp.set('date_to', params.date_to);
  if (params.sort_by) sp.set('sort_by', params.sort_by);
  if (params.sort_dir) sp.set('sort_dir', params.sort_dir);
  const res = await request.get(`${API_BASE}/kebutuhan-bmn/search?${sp}`, {
    headers: authHeaders(user),
  });
  return { status: res.status(), body: await safeJson(res) };
}

export async function getSearchSuggestions(
  request: APIRequestContext,
  user: MockUser,
  q: string,
  limit?: number
) {
  const sp = new URLSearchParams({ q });
  if (limit) sp.set('limit', String(limit));
  const res = await request.get(`${API_BASE}/kebutuhan-bmn/search/suggestions?${sp}`, {
    headers: authHeaders(user),
  });
  return { status: res.status(), body: await safeJson(res) };
}

// ============================================================================
// Batch Operations API
// ============================================================================

export async function batchApproveKebutuhan(
  request: APIRequestContext,
  user: MockUser,
  data: { kebutuhan_ids: string[]; komentar?: string }
) {
  const res = await request.post(`${API_BASE}/kebutuhan-bmn/batch/approve`, {
    data,
    headers: authHeaders(user),
  });
  return { status: res.status(), body: await safeJson(res) };
}

export async function batchRejectKebutuhan(
  request: APIRequestContext,
  user: MockUser,
  data: { kebutuhan_ids: string[]; komentar: string }
) {
  const res = await request.post(`${API_BASE}/kebutuhan-bmn/batch/reject`, {
    data,
    headers: authHeaders(user),
  });
  return { status: res.status(), body: await safeJson(res) };
}

export async function batchUpdateStatus(
  request: APIRequestContext,
  user: MockUser,
  data: { kebutuhan_ids: string[]; target_status: number; komentar?: string }
) {
  const res = await request.post(`${API_BASE}/kebutuhan-bmn/batch/update-status`, {
    data,
    headers: authHeaders(user),
  });
  return { status: res.status(), body: await safeJson(res) };
}

// ============================================================================
// Kebutuhan BMN - Satker Operations (expanded)
// ============================================================================

export async function getSatkerDetail(
  request: APIRequestContext,
  user: MockUser,
  satkerId: string,
  params?: { page?: number; per_page?: number }
) {
  const sp = new URLSearchParams();
  if (params?.page) sp.set('page', String(params.page));
  if (params?.per_page) sp.set('per_page', String(params.per_page));
  const res = await request.get(`${API_BASE}/kebutuhan-bmn/satker/${satkerId}?${sp}`, {
    headers: authHeaders(user),
  });
  return { status: res.status(), body: await safeJson(res) };
}

export async function getSatkerAktivitas(
  request: APIRequestContext,
  user: MockUser,
  satkerId: string
) {
  const res = await request.get(`${API_BASE}/kebutuhan-bmn/satker/${satkerId}/aktivitas`, {
    headers: authHeaders(user),
  });
  return { status: res.status(), body: await safeJson(res) };
}

export async function setBarangPrioritas(
  request: APIRequestContext,
  user: MockUser,
  data: { items: Array<{ barang_id: string; prioritas: number; skor?: number }> }
) {
  const res = await request.post(`${API_BASE}/kebutuhan-bmn/prioritas`, {
    data,
    headers: authHeaders(user),
  });
  return { status: res.status(), body: await safeJson(res) };
}

export async function deleteBarang(
  request: APIRequestContext,
  user: MockUser,
  barangId: string
) {
  const res = await request.delete(`${API_BASE}/kebutuhan-bmn/barang/${barangId}`, {
    headers: authHeaders(user),
  });
  return { status: res.status(), body: await safeJson(res) };
}

export async function updateBarangApproval(
  request: APIRequestContext,
  user: MockUser,
  barangId: string,
  data: { jml_setuju: number; keterangan?: string }
) {
  const res = await request.put(`${API_BASE}/kebutuhan-bmn/barang/${barangId}/approval`, {
    data,
    headers: authHeaders(user),
  });
  return { status: res.status(), body: await safeJson(res) };
}

export async function updateKebutuhanBmnPengajuan(
  request: APIRequestContext,
  user: MockUser,
  id: string,
  data: { nama?: string; version: number; [key: string]: any }
) {
  const res = await request.put(`${API_BASE}/kebutuhan-bmn/pengajuan/${id}`, {
    data,
    headers: authHeaders(user),
  });
  return { status: res.status(), body: await safeJson(res) };
}

export async function deleteKebutuhanBmnPengajuan(
  request: APIRequestContext,
  user: MockUser,
  id: string
) {
  const res = await request.delete(`${API_BASE}/kebutuhan-bmn/pengajuan/${id}`, {
    headers: authHeaders(user),
  });
  return { status: res.status(), body: await safeJson(res) };
}

// ============================================================================
// Pemakaian BMN - Expanded endpoints
// ============================================================================

export async function updatePemakaianBmn(
  request: APIRequestContext,
  user: MockUser,
  id: string,
  data: Record<string, any>
) {
  const res = await request.put(`${API_BASE}/pemakaian-bmn/${id}`, {
    data,
    headers: authHeaders(user),
  });
  return { status: res.status(), body: await safeJson(res) };
}

export async function getBmnAvailability(
  request: APIRequestContext,
  user: MockUser,
  bmnNup: string
) {
  const res = await request.get(`${API_BASE}/pemakaian-bmn/bmn/${bmnNup}/availability`, {
    headers: authHeaders(user),
  });
  return { status: res.status(), body: await safeJson(res) };
}

export async function getBmnUsageHistory(
  request: APIRequestContext,
  user: MockUser,
  bmnNup: string
) {
  const res = await request.get(`${API_BASE}/pemakaian-bmn/bmn/${bmnNup}/history`, {
    headers: authHeaders(user),
  });
  return { status: res.status(), body: await safeJson(res) };
}

export async function getPegawaiUsageHistory(
  request: APIRequestContext,
  user: MockUser,
  pegawaiNip: string
) {
  const res = await request.get(`${API_BASE}/pemakaian-bmn/pegawai/${pegawaiNip}/history`, {
    headers: authHeaders(user),
  });
  return { status: res.status(), body: await safeJson(res) };
}

export async function getExpiringPermits(
  request: APIRequestContext,
  user: MockUser,
  days?: number
) {
  const sp = days ? `?days=${days}` : '';
  const res = await request.get(`${API_BASE}/pemakaian-bmn/expiring${sp}`, {
    headers: authHeaders(user),
  });
  return { status: res.status(), body: await safeJson(res) };
}

export async function autoExpirePermits(
  request: APIRequestContext,
  user: MockUser
) {
  const res = await request.post(`${API_BASE}/pemakaian-bmn/auto-expire`, {
    headers: authHeaders(user),
  });
  return { status: res.status(), body: await safeJson(res) };
}

export async function getActiveUsageDashboard(
  request: APIRequestContext,
  user: MockUser,
  params?: { satker_id?: string; jenis_bmn?: string; start_date?: string; end_date?: string }
) {
  const sp = new URLSearchParams();
  if (params?.satker_id) sp.set('satker_id', params.satker_id);
  if (params?.jenis_bmn) sp.set('jenis_bmn', params.jenis_bmn);
  if (params?.start_date) sp.set('start_date', params.start_date);
  if (params?.end_date) sp.set('end_date', params.end_date);
  const res = await request.get(`${API_BASE}/pemakaian-bmn/monitoring/active-usage?${sp}`, {
    headers: authHeaders(user),
  });
  return { status: res.status(), body: await safeJson(res) };
}

export async function getBmnUtilizationReport(
  request: APIRequestContext,
  user: MockUser,
  params?: { satker_id?: string; jenis_bmn?: string; start_date?: string; end_date?: string }
) {
  const sp = new URLSearchParams();
  if (params?.satker_id) sp.set('satker_id', params.satker_id);
  if (params?.jenis_bmn) sp.set('jenis_bmn', params.jenis_bmn);
  if (params?.start_date) sp.set('start_date', params.start_date);
  if (params?.end_date) sp.set('end_date', params.end_date);
  const res = await request.get(`${API_BASE}/pemakaian-bmn/monitoring/utilization-report?${sp}`, {
    headers: authHeaders(user),
  });
  return { status: res.status(), body: await safeJson(res) };
}

export async function getPermitDocument(
  request: APIRequestContext,
  user: MockUser,
  id: string
) {
  const res = await request.get(`${API_BASE}/pemakaian-bmn/${id}/document`, {
    headers: authHeaders(user),
  });
  return { status: res.status(), body: await safeJson(res) };
}

// ============================================================================
// Penghapusan BMN - Expanded endpoints
// ============================================================================

export async function updatePenghapusanBmn(
  request: APIRequestContext,
  user: MockUser,
  id: string,
  data: Record<string, any>
) {
  const res = await request.put(`${API_BASE}/penghapusan-bmn/${id}`, {
    data,
    headers: authHeaders(user),
  });
  return { status: res.status(), body: await safeJson(res) };
}

export async function deletePenghapusanBmn(
  request: APIRequestContext,
  user: MockUser,
  id: string
) {
  const res = await request.delete(`${API_BASE}/penghapusan-bmn/${id}`, {
    headers: authHeaders(user),
  });
  return { status: res.status(), body: await safeJson(res) };
}

export async function penghapusanValidatorWilayahAction(
  request: APIRequestContext,
  user: MockUser,
  id: string,
  data: { aksi: 'forward' | 'return'; catatan?: string }
) {
  const res = await request.post(`${API_BASE}/penghapusan-bmn/${id}/validator-wilayah`, {
    data,
    headers: authHeaders(user),
  });
  return { status: res.status(), body: await safeJson(res) };
}

export async function getPenghapusanDocument(
  request: APIRequestContext,
  user: MockUser,
  id: string
) {
  const res = await request.get(`${API_BASE}/penghapusan-bmn/${id}/document`, {
    headers: authHeaders(user),
  });
  return { status: res.status(), body: await safeJson(res) };
}

// ============================================================================
// Pakaian Dinas - Expanded endpoints
// ============================================================================

export async function getJenisPakaianById(
  request: APIRequestContext,
  user: MockUser,
  id: string
) {
  const res = await request.get(`${API_BASE}/pakaian-dinas/jenis/${id}`, {
    headers: authHeaders(user),
  });
  return { status: res.status(), body: await safeJson(res) };
}

export async function updateJenisPakaianDinas(
  request: APIRequestContext,
  user: MockUser,
  id: string,
  data: Record<string, any>
) {
  const res = await request.put(`${API_BASE}/pakaian-dinas/jenis/${id}`, {
    data,
    headers: authHeaders(user),
  });
  return { status: res.status(), body: await safeJson(res) };
}

export async function deleteJenisPakaianDinas(
  request: APIRequestContext,
  user: MockUser,
  id: string
) {
  const res = await request.delete(`${API_BASE}/pakaian-dinas/jenis/${id}`, {
    headers: authHeaders(user),
  });
  return { status: res.status(), body: await safeJson(res) };
}

// Spesifikasi
export async function listSpesifikasi(
  request: APIRequestContext,
  user: MockUser,
  params?: { page?: number; per_page?: number; jenis_id?: string }
) {
  const sp = new URLSearchParams();
  if (params?.page) sp.set('page', String(params.page));
  if (params?.per_page) sp.set('per_page', String(params.per_page));
  if (params?.jenis_id) sp.set('jenis_id', params.jenis_id);
  const res = await request.get(`${API_BASE}/pakaian-dinas/spesifikasi?${sp}`, {
    headers: authHeaders(user),
  });
  return { status: res.status(), body: await safeJson(res) };
}

export async function createSpesifikasi(
  request: APIRequestContext,
  user: MockUser,
  data: {
    jenis_pakaian_dinas_id: string;
    nama: string;
    gender: string;
    ukuran_group: string;
    deskripsi?: string;
    is_active?: boolean;
  }
) {
  const res = await request.post(`${API_BASE}/pakaian-dinas/spesifikasi`, {
    data,
    headers: authHeaders(user),
  });
  return { status: res.status(), body: await safeJson(res) };
}

export async function getSpesifikasiById(
  request: APIRequestContext,
  user: MockUser,
  id: string
) {
  const res = await request.get(`${API_BASE}/pakaian-dinas/spesifikasi/${id}`, {
    headers: authHeaders(user),
  });
  return { status: res.status(), body: await safeJson(res) };
}

export async function updateSpesifikasi(
  request: APIRequestContext,
  user: MockUser,
  id: string,
  data: Record<string, any>
) {
  const res = await request.put(`${API_BASE}/pakaian-dinas/spesifikasi/${id}`, {
    data,
    headers: authHeaders(user),
  });
  return { status: res.status(), body: await safeJson(res) };
}

export async function deleteSpesifikasi(
  request: APIRequestContext,
  user: MockUser,
  id: string
) {
  const res = await request.delete(`${API_BASE}/pakaian-dinas/spesifikasi/${id}`, {
    headers: authHeaders(user),
  });
  return { status: res.status(), body: await safeJson(res) };
}

// Sub-Spesifikasi
export async function listSubSpesifikasi(
  request: APIRequestContext,
  user: MockUser,
  params?: { page?: number; per_page?: number; spesifikasi_id?: string }
) {
  const sp = new URLSearchParams();
  if (params?.page) sp.set('page', String(params.page));
  if (params?.per_page) sp.set('per_page', String(params.per_page));
  if (params?.spesifikasi_id) sp.set('spesifikasi_id', params.spesifikasi_id);
  const res = await request.get(`${API_BASE}/pakaian-dinas/subspesifikasi?${sp}`, {
    headers: authHeaders(user),
  });
  return { status: res.status(), body: await safeJson(res) };
}

export async function createSubSpesifikasi(
  request: APIRequestContext,
  user: MockUser,
  data: { spesifikasi_id: string; nama: string; gender: string; is_active?: boolean }
) {
  const res = await request.post(`${API_BASE}/pakaian-dinas/subspesifikasi`, {
    data,
    headers: authHeaders(user),
  });
  return { status: res.status(), body: await safeJson(res) };
}

export async function deleteSubSpesifikasi(
  request: APIRequestContext,
  user: MockUser,
  id: string
) {
  const res = await request.delete(`${API_BASE}/pakaian-dinas/subspesifikasi/${id}`, {
    headers: authHeaders(user),
  });
  return { status: res.status(), body: await safeJson(res) };
}

// Ukuran
export async function listUkuran(
  request: APIRequestContext,
  user: MockUser,
  group?: string
) {
  const sp = group ? `?group=${group}` : '';
  const res = await request.get(`${API_BASE}/pakaian-dinas/ukuran${sp}`, {
    headers: authHeaders(user),
  });
  return { status: res.status(), body: await safeJson(res) };
}

// Ukuran Pakaian Pegawai (self-service)
export async function getPersonalUkuran(
  request: APIRequestContext,
  user: MockUser
) {
  const res = await request.get(`${API_BASE}/pakaian-dinas/ukuran-pakaian-pegawai`, {
    headers: authHeaders(user),
  });
  return { status: res.status(), body: await safeJson(res) };
}

export async function updatePersonalUkuran(
  request: APIRequestContext,
  user: MockUser,
  data: { ukuran_baju: string; ukuran_celana: string; ukuran_sepatu: string; with_hijab: boolean }
) {
  const res = await request.post(`${API_BASE}/pakaian-dinas/ukuran-pakaian-pegawai`, {
    data,
    headers: authHeaders(user),
  });
  return { status: res.status(), body: await safeJson(res) };
}

// Pakaian Dinas Pengajuan Satker
export async function listPengajuanSatker(
  request: APIRequestContext,
  user: MockUser,
  pengajuanId: string,
  params?: { page?: number; per_page?: number }
) {
  const sp = new URLSearchParams();
  if (params?.page) sp.set('page', String(params.page));
  if (params?.per_page) sp.set('per_page', String(params.per_page));
  const res = await request.get(
    `${API_BASE}/pakaian-dinas/pengajuan/${pengajuanId}/satker?${sp}`,
    { headers: authHeaders(user) }
  );
  return { status: res.status(), body: await safeJson(res) };
}

export async function getPengajuanSatkerById(
  request: APIRequestContext,
  user: MockUser,
  id: string
) {
  const res = await request.get(`${API_BASE}/pakaian-dinas/satker/${id}`, {
    headers: authHeaders(user),
  });
  return { status: res.status(), body: await safeJson(res) };
}

// Validator action
export async function pakaianDinasValidatorAction(
  request: APIRequestContext,
  user: MockUser,
  data: { pengajuan_satker_id: string; aksi: 'approve' | 'reject'; komentar?: string }
) {
  const res = await request.post(`${API_BASE}/pakaian-dinas/validator-action`, {
    data,
    headers: authHeaders(user),
  });
  return { status: res.status(), body: await safeJson(res) };
}

export async function deletePengajuanPakaianDinas(
  request: APIRequestContext,
  user: MockUser,
  id: string
) {
  const res = await request.delete(`${API_BASE}/pakaian-dinas/pengajuan/${id}`, {
    headers: authHeaders(user),
  });
  return { status: res.status(), body: await safeJson(res) };
}

// Reports
export async function getLaporanRekapUkuran(
  request: APIRequestContext,
  user: MockUser,
  params: { pengajuan_id: string; jenis_kelamin?: string; eselon?: string; jenis?: string }
) {
  const sp = new URLSearchParams({ pengajuan_id: params.pengajuan_id });
  if (params.jenis_kelamin) sp.set('jenis_kelamin', params.jenis_kelamin);
  if (params.eselon) sp.set('eselon', params.eselon);
  if (params.jenis) sp.set('jenis', params.jenis);
  const res = await request.get(`${API_BASE}/pakaian-dinas/laporan/rekap-ukuran?${sp}`, {
    headers: authHeaders(user),
  });
  return { status: res.status(), body: await safeJson(res) };
}

export async function getLaporanDaftarPegawai(
  request: APIRequestContext,
  user: MockUser,
  params: {
    pengajuan_id: string;
    page?: number;
    per_page?: number;
    satker_id?: string;
    jenis_kelamin?: string;
    eselon?: string;
    jenis?: string;
  }
) {
  const sp = new URLSearchParams({ pengajuan_id: params.pengajuan_id });
  if (params.page) sp.set('page', String(params.page));
  if (params.per_page) sp.set('per_page', String(params.per_page));
  if (params.satker_id) sp.set('satker_id', params.satker_id);
  if (params.jenis_kelamin) sp.set('jenis_kelamin', params.jenis_kelamin);
  if (params.eselon) sp.set('eselon', params.eselon);
  if (params.jenis) sp.set('jenis', params.jenis);
  const res = await request.get(`${API_BASE}/pakaian-dinas/laporan/daftar-pegawai?${sp}`, {
    headers: authHeaders(user),
  });
  return { status: res.status(), body: await safeJson(res) };
}

export async function cetakLaporan(
  request: APIRequestContext,
  user: MockUser,
  params: {
    jenis_laporan: 'rekap' | 'daftar';
    jenis_file: 'excel' | 'pdf';
    pengajuan_id: string;
    satker_id?: string;
    jenis_kelamin?: string;
    eselon?: string;
    jenis?: string;
  }
) {
  const sp = new URLSearchParams({
    jenis_laporan: params.jenis_laporan,
    jenis_file: params.jenis_file,
    pengajuan_id: params.pengajuan_id,
  });
  if (params.satker_id) sp.set('satker_id', params.satker_id);
  if (params.jenis_kelamin) sp.set('jenis_kelamin', params.jenis_kelamin);
  if (params.eselon) sp.set('eselon', params.eselon);
  if (params.jenis) sp.set('jenis', params.jenis);
  const res = await request.get(`${API_BASE}/pakaian-dinas/laporan/cetak?${sp}`, {
    headers: authHeaders(user),
  });
  return { status: res.status(), headers: res.headers(), body: await res.body() };
}

export async function downloadRekapitulasi(
  request: APIRequestContext,
  user: MockUser,
  pengajuanId: string
) {
  const res = await request.get(`${API_BASE}/pakaian-dinas/pengajuan/${pengajuanId}/rekapitulasi`, {
    headers: authHeaders(user),
  });
  return { status: res.status(), headers: res.headers(), body: await res.body() };
}

// MySIMKARI integration
export async function getPegawaiSatker(
  request: APIRequestContext,
  user: MockUser,
  satkerId: string
) {
  const res = await request.get(`${API_BASE}/pakaian-dinas/pegawai-satker/${satkerId}`, {
    headers: authHeaders(user),
  });
  return { status: res.status(), body: await safeJson(res) };
}

export async function getPegawaiWithSizes(
  request: APIRequestContext,
  user: MockUser,
  satkerId: string
) {
  const res = await request.get(`${API_BASE}/pakaian-dinas/pegawai-satker/${satkerId}/with-sizes`, {
    headers: authHeaders(user),
  });
  return { status: res.status(), body: await safeJson(res) };
}

// ============================================================================
// General Export API
// ============================================================================

export async function generalExport(
  request: APIRequestContext,
  user: MockUser,
  params: {
    entity_type: string;
    filters?: string;
    limit?: number;
    tahun_anggaran?: number;
    satker_id?: string;
    status?: string;
  }
) {
  const sp = new URLSearchParams({ entity_type: params.entity_type });
  if (params.filters) sp.set('filters', params.filters);
  if (params.limit) sp.set('limit', String(params.limit));
  if (params.tahun_anggaran) sp.set('tahun_anggaran', String(params.tahun_anggaran));
  if (params.satker_id) sp.set('satker_id', params.satker_id);
  if (params.status) sp.set('status', params.status);
  const res = await request.get(`${API_BASE}/export/excel?${sp}`, {
    headers: authHeaders(user),
  });
  return { status: res.status(), headers: res.headers(), body: await res.body() };
}

export async function getExportJobStatus(
  request: APIRequestContext,
  user: MockUser,
  jobId: string
) {
  const res = await request.get(`${API_BASE}/export/jobs/${jobId}/status`, {
    headers: authHeaders(user),
  });
  return { status: res.status(), body: await safeJson(res) };
}

export async function downloadExportJob(
  request: APIRequestContext,
  user: MockUser,
  jobId: string
) {
  const res = await request.get(`${API_BASE}/export/jobs/${jobId}/download`, {
    headers: authHeaders(user),
  });
  return { status: res.status(), headers: res.headers(), body: await res.body() };
}

// ============================================================================
// Assets API
// ============================================================================

export async function listAssets(
  request: APIRequestContext,
  user: MockUser,
  params?: { page?: number; per_page?: number; category?: string }
) {
  const sp = new URLSearchParams();
  if (params?.page) sp.set('page', String(params.page));
  if (params?.per_page) sp.set('per_page', String(params.per_page));
  if (params?.category) sp.set('category', params.category);
  const res = await request.get(`${API_BASE}/assets?${sp}`, {
    headers: authHeaders(user),
  });
  return { status: res.status(), body: await safeJson(res) };
}

export async function getAssetById(
  request: APIRequestContext,
  user: MockUser,
  id: string
) {
  const res = await request.get(`${API_BASE}/assets/${id}`, {
    headers: authHeaders(user),
  });
  return { status: res.status(), body: await safeJson(res) };
}

// ============================================================================
// Mapping Kodefikasi API
// ============================================================================

export async function detectNonStandardCodes(
  request: APIRequestContext,
  user: MockUser
) {
  const res = await request.get(`${API_BASE}/mapping/detect`, {
    headers: authHeaders(user),
  });
  return { status: res.status(), body: await safeJson(res) };
}

export async function getMappingSuggestions(
  request: APIRequestContext,
  user: MockUser
) {
  const res = await request.get(`${API_BASE}/mapping/suggestions`, {
    headers: authHeaders(user),
  });
  return { status: res.status(), body: await safeJson(res) };
}

export async function getAllMappingProposals(
  request: APIRequestContext,
  user: MockUser
) {
  const res = await request.get(`${API_BASE}/mapping/proposals`, {
    headers: authHeaders(user),
  });
  return { status: res.status(), body: await safeJson(res) };
}

