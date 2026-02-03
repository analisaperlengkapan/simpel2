import { test, expect } from '@playwright/test';

test.describe('Perlengkapan API Integration', () => {
  // Use baseURL from configuration (configured in playwright.config.ts)
  // or default to relative path which Playwright resolves against baseURL

  test('should create and retrieve Pengadaan and HPS', async ({ request }) => {
    // 1. Create Pengadaan
    // Assuming /api/pembinaan/perlengkapan is the prefix based on routes.rs and service mounting
    const pengadaanRes = await request.post('/api/pembinaan/perlengkapan/pengadaan', {
      data: {
        judul: 'Pengadaan E2E Test',
        jenis: 'TIK',
        anggaran: 10000000,
        deskripsi: 'Test Description'
      }
    });

    // Check if created (201)
    expect(pengadaanRes.status()).toBe(201);
    const pengadaan = await pengadaanRes.json();
    expect(pengadaan.data.judul).toBe('Pengadaan E2E Test');
    const pengadaanId = pengadaan.data.id;

    // 2. Create HPS for the Pengadaan
    const hpsRes = await request.post(`/api/pembinaan/perlengkapan/pengadaan/${pengadaanId}/hps`, {
      data: {
        pengadaan_id: pengadaanId,
        no_hps: 'HPS-E2E-001',
        tgl_hps: '2023-01-01',
        nip_penandatangan: '123456',
        nama_penandatangan: 'Tester',
        pangkat_penandatangan: 'IV/a',
        barang: [{ item: 'Laptop', price: 5000000 }]
      }
    });

    expect(hpsRes.status()).toBe(201);
    const hps = await hpsRes.json();
    expect(hps.data.no_hps).toBe('HPS-E2E-001');

    // 3. Retrieve HPS list
    const hpsListRes = await request.get(`/api/pembinaan/perlengkapan/pengadaan/${pengadaanId}/hps`);
    expect(hpsListRes.status()).toBe(200);
    const hpsList = await hpsListRes.json();
    expect(hpsList.data.length).toBeGreaterThan(0);
    expect(hpsList.data[0].no_hps).toBe('HPS-E2E-001');
  });
});
