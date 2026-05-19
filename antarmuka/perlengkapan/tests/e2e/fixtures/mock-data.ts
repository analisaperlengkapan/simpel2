/**
 * Mock Users & Roles for E2E Testing
 *
 * Defines mock user profiles for each role in the business process:
 * - operator_satker: Creates submissions, uploads documents
 * - validator_wilayah: Reviews and forwards/returns submissions
 * - validator_pusat: Final approval/rejection, generates SK documents
 * - admin: Full access, system administration
 */

export interface MockUser {
  id: string;
  nip: string;
  nama: string;
  pangkat: string;
  jabatan: string;
  role: string;
  satker_id: string;
  satker_nama: string;
  wilayah_id: string;
  wilayah_nama: string;
  email: string;
}

export const MOCK_USERS: Record<string, MockUser> = {
  operator_satker: {
    id: '00000000-0000-0000-0000-000000000001',
    nip: '198501012010011001',
    nama: 'Budi Santoso',
    pangkat: 'III/c',
    jabatan: 'Pengelola BMN',
    role: 'operator_satker',
    satker_id: 'SKR001',
    satker_nama: 'Kejaksaan Negeri Jakarta Selatan',
    wilayah_id: 'WIL001',
    wilayah_nama: 'Kejaksaan Tinggi DKI Jakarta',
    email: 'budi.santoso@kejaksaan.go.id',
  },
  validator_wilayah: {
    id: '00000000-0000-0000-0000-000000000002',
    nip: '197801012005011001',
    nama: 'Siti Rahmawati',
    pangkat: 'IV/a',
    jabatan: 'Kabag Perlengkapan',
    role: 'validator_wilayah',
    satker_id: 'WIL001',
    satker_nama: 'Kejaksaan Tinggi DKI Jakarta',
    wilayah_id: 'WIL001',
    wilayah_nama: 'Kejaksaan Tinggi DKI Jakarta',
    email: 'siti.rahmawati@kejaksaan.go.id',
  },
  validator_pusat: {
    id: '00000000-0000-0000-0000-000000000003',
    nip: '197001012000011001',
    nama: 'Ahmad Dharma',
    pangkat: 'IV/b',
    jabatan: 'Kasubdir Perlengkapan',
    role: 'validator_pusat',
    satker_id: 'PUSAT001',
    satker_nama: 'Kejaksaan Agung RI',
    wilayah_id: 'PUSAT001',
    wilayah_nama: 'Kejaksaan Agung RI',
    email: 'ahmad.dharma@kejaksaan.go.id',
  },
  admin: {
    id: '00000000-0000-0000-0000-000000000004',
    nip: '196501011990011001',
    nama: 'Admin Sistem',
    pangkat: 'IV/c',
    jabatan: 'Administrator Sistem',
    role: 'admin',
    satker_id: 'PUSAT001',
    satker_nama: 'Kejaksaan Agung RI',
    wilayah_id: 'PUSAT001',
    wilayah_nama: 'Kejaksaan Agung RI',
    email: 'admin@kejaksaan.go.id',
  },
};

/**
 * API base path for perlengkapan service
 */
export const API_ROOT = (process.env.API_BASE_URL || 'http://localhost:8093').replace(/\/$/, '');
export const API_BASE = `${API_ROOT}/api/pembinaan/perlengkapan`;

/**
 * Workflow status codes
 */
export const KEBUTUHAN_BMN_STATUS = {
  DRAFT: 2000,
  INPUT_BARANG: 2001,
  SUBMIT_WILAYAH: 2002,
  REVISI_SATKER: 2003,
  SUBMIT_PUSAT: 2004,
  ANALISIS_KELAYAKAN: 2005,
  APPROVED: 2006,
  REJECTED: 2007,
  COMPLETED: 2008,
  CANCELLED: 2009,
} as const;

export const PEMAKAIAN_BMN_STATUS = {
  DRAFT: 3000,
  SUBMITTED: 3001,
  APPROVED: 3002,
  REJECTED: 3003,
  ACTIVE: 3004,
  EXPIRED: 3005,
  REVOKED: 3006,
  CANCELLED: 3007,
} as const;

export const PENGHAPUSAN_BMN_STATUS = {
  DRAFT: 4000,
  SUBMIT_WILAYAH: 4001,
  RETURNED_TO_OPERATOR: 4002,
  SUBMIT_PUSAT: 4003,
  VERIFIKASI_PUSAT: 4004,
  KONSEP_SK_GENERATED: 4005,
  SK_SIGNED: 4006,
  COMPLETED: 4007,
  REJECTED: 4008,
} as const;

export const PAKAIAN_DINAS_STATUS = {
  INPUT: 1000,
  SUBMIT_TO_VALIDATOR: 1001,
  REVISI_PELAKSANA: 1003,
  SUBMIT_TO_PUSAT: 1004,
  REVISI_SATKER: 1005,
  REVISI_WILAYAH: 1007,
  SELESAI: 1008,
  START_KEJAGUNG: 1009,
  SUBMIT_TO_PUSAT_FROM_WILAYAH: 1010,
  START_NON_KEJAGUNG: 1011,
  SUBMIT_TO_VALIDATOR_WILAYAH: 1012,
} as const;

/**
 * Mock BMN data from SIMAN integration
 */
export const MOCK_BMN_DATA = {
  kendaraan: {
    kode_barang: '3.05.02.01.001',
    nama_barang: 'Kendaraan Bermotor Roda 4 (Sedan)',
    nup: '001',
    satker_id: 'SKR001',
    kondisi: 'Baik',
    lokasi: 'Jakarta Selatan',
    nilai_perolehan: 350000000,
  },
  laptop: {
    kode_barang: '3.06.02.01.003',
    nama_barang: 'Laptop',
    nup: '015',
    satker_id: 'SKR001',
    kondisi: 'Baik',
    lokasi: 'Jakarta Selatan',
    nilai_perolehan: 15000000,
  },
  rumah_negara: {
    kode_barang: '4.01.01.01.001',
    nama_barang: 'Rumah Negara Golongan I',
    nup: '003',
    satker_id: 'SKR001',
    kondisi: 'Baik',
    lokasi: 'Jakarta Selatan',
    nilai_perolehan: 1500000000,
  },
};

/**
 * Mock pegawai data from MySIMKARI integration
 */
export const MOCK_PEGAWAI_DATA = {
  pegawai1: {
    nip: '199001012015011001',
    nama: 'Rudi Hartono',
    pangkat: 'III/a',
    jabatan: 'Jaksa Muda',
    eselon: null,
    is_jaksa: true,
    foto_url: null,
    satker_id: 'SKR001',
    satker_nama: 'Kejaksaan Negeri Jakarta Selatan',
  },
  pegawai2: {
    nip: '199201012016011001',
    nama: 'Dewi Susanti',
    pangkat: 'III/b',
    jabatan: 'Kasubbag Umum',
    eselon: 'IV.a',
    is_jaksa: false,
    foto_url: null,
    satker_id: 'SKR001',
    satker_nama: 'Kejaksaan Negeri Jakarta Selatan',
  },
};

/**
 * Export entity types (for general export endpoint)
 */
export const EXPORT_ENTITY_TYPES = [
  'kebutuhan_bmn',
  'pakaian_dinas',
  'roadmap_sarpras',
  'riwayat_pemenuhan',
] as const;

/**
 * Jenis BMN for pemakaian
 */
export const JENIS_BMN = {
  KENDARAAN_BERMOTOR: 'KENDARAAN_BERMOTOR',
  RUMAH_NEGARA: 'RUMAH_NEGARA',
  LAPTOP: 'LAPTOP',
  LAINNYA: 'LAINNYA',
} as const;

/**
 * Metode penghapusan BMN
 */
export const METODE_PENGHAPUSAN = [
  'Pemusnahan',
  'Pemindahtanganan',
  'Hibah',
  'Tukar Menukar',
  'Penjualan',
] as const;

/**
 * Pakaian Dinas gender values
 */
export const GENDER_VALUES = ['L', 'P', 'SEMUA'] as const;

/**
 * Ukuran group values
 */
export const UKURAN_GROUP = ['BAJU', 'CELANA', 'SEPATU'] as const;

/**
 * Invalid UUIDs for testing
 */
export const INVALID_UUIDS = {
  not_a_uuid: 'not-a-valid-uuid',
  nonexistent: '99999999-9999-9999-9999-999999999999',
  empty: '',
  sql_injection: "'; DROP TABLE users; --",
} as const;

/**
 * Validation test data - intentionally invalid
 */
export const INVALID_DATA = {
  empty_string: '',
  too_short_nama: 'ab',              // min 3 chars
  too_long_nama: 'x'.repeat(300),    // max 255 chars
  too_short_alasan: 'pendek',        // min 10 chars for alasan field
  too_short_keperluan: 'short',      // min 10 chars for keperluan
  invalid_tahun_low: 2019,           // min 2020
  invalid_tahun_high: 2101,          // max 2100
  negative_jumlah: -1,
  zero_jumlah: 0,
  huge_jumlah: 999999999,
  invalid_page: 0,                   // min 1
  too_large_page: 100001,            // max 100000
  invalid_per_page: 0,               // min 1
  too_large_per_page: 1001,          // max 1000
} as const;

/**
 * Mock data for additional satker (second satker for multi-satker tests)
 */
export const MOCK_USERS_EXTRA: Record<string, MockUser> = {
  operator_satker_2: {
    id: '00000000-0000-0000-0000-000000000005',
    nip: '199001012015011002',
    nama: 'Andi Pratama',
    pangkat: 'III/b',
    jabatan: 'Pengelola BMN',
    role: 'operator_satker',
    satker_id: 'SKR002',
    satker_nama: 'Kejaksaan Negeri Jakarta Utara',
    wilayah_id: 'WIL001',
    wilayah_nama: 'Kejaksaan Tinggi DKI Jakarta',
    email: 'andi.pratama@kejaksaan.go.id',
  },
  pimpinan_satker: {
    id: '00000000-0000-0000-0000-000000000006',
    nip: '197501011999011001',
    nama: 'Hendra Wijaya',
    pangkat: 'IV/a',
    jabatan: 'Kepala Kejaksaan Negeri',
    role: 'pimpinan_satker',
    satker_id: 'SKR001',
    satker_nama: 'Kejaksaan Negeri Jakarta Selatan',
    wilayah_id: 'WIL001',
    wilayah_nama: 'Kejaksaan Tinggi DKI Jakarta',
    email: 'hendra.wijaya@kejaksaan.go.id',
  },
  pegawai_biasa: {
    id: '00000000-0000-0000-0000-000000000007',
    nip: '199501012020011001',
    nama: 'Rizki Maulana',
    pangkat: 'III/a',
    jabatan: 'Jaksa Muda',
    role: 'pegawai',
    satker_id: 'SKR001',
    satker_nama: 'Kejaksaan Negeri Jakarta Selatan',
    wilayah_id: 'WIL001',
    wilayah_nama: 'Kejaksaan Tinggi DKI Jakarta',
    email: 'rizki.maulana@kejaksaan.go.id',
  },
  unauthorized_user: {
    id: '00000000-0000-0000-0000-000000000099',
    nip: '200001012025011001',
    nama: 'Tamu Luar',
    pangkat: '-',
    jabatan: '-',
    role: 'guest',
    satker_id: 'EXTERNAL',
    satker_nama: 'External',
    wilayah_id: 'EXTERNAL',
    wilayah_nama: 'External',
    email: 'tamu@external.id',
  },
};

/**
 * Additional BMN items for multi-BMN tests
 */
export const MOCK_BMN_EXTRA = {
  printer: {
    kode_barang: '3.06.02.01.010',
    nama_barang: 'Printer',
    nup: '025',
    satker_id: 'SKR001',
    kondisi: 'Baik',
    lokasi: 'Jakarta Selatan',
    nilai_perolehan: 5000000,
  },
  ac: {
    kode_barang: '3.06.03.01.005',
    nama_barang: 'AC Split',
    nup: '010',
    satker_id: 'SKR001',
    kondisi: 'Rusak Ringan',
    lokasi: 'Jakarta Selatan',
    nilai_perolehan: 8000000,
  },
  meja: {
    kode_barang: '3.06.01.01.001',
    nama_barang: 'Meja Kerja',
    nup: '050',
    satker_id: 'SKR001',
    kondisi: 'Baik',
    lokasi: 'Jakarta Selatan',
    nilai_perolehan: 3000000,
  },
  rusak_berat: {
    kode_barang: '3.06.02.01.099',
    nama_barang: 'Komputer Desktop Rusak',
    nup: '042',
    satker_id: 'SKR001',
    kondisi: 'Rusak Berat',
    lokasi: 'Jakarta Selatan',
    nilai_perolehan: 12000000,
  },
};

/**
 * Screenshot directory
 */
export const SCREENSHOT_DIR = 'test-results/screenshots';

/**
 * Helper function to generate a unique test name suffix
 */
export function uniqueSuffix(): string {
  return `${Date.now()}-${Math.random().toString(36).substring(2, 8)}`;
}

/**
 * Generate test date strings
 */
export function testDates(daysFromNow = 30) {
  const start = new Date();
  const end = new Date(start.getTime() + daysFromNow * 24 * 60 * 60 * 1000);
  return {
    start: start.toISOString().split('T')[0],
    end: end.toISOString().split('T')[0],
    past: '2020-01-01',
    far_future: '2099-12-31',
  };
}

/**
 * Generate N test items for batch operations
 */
export function generateBatchIds(n: number): string[] {
  return Array.from({ length: n }, (_, i) =>
    `00000000-0000-0000-0000-${String(i + 100).padStart(12, '0')}`
  );
}
