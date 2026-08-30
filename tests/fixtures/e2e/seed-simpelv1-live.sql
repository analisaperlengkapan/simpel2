-- ============================================================================
-- simpelv1 IntegrationLive fixture (F-GW PR-C)
-- ============================================================================
-- Adds integrasi.mysimkari_pegawai rows so simpelv1's
-- IntegrasiGrpcClient::fetchEmployeeFromMySIMKARI has REAL data to return
-- through the gateway (→ integrasi GetMysimkariPegawai). Apply AFTER
-- seed-multisatker.sql, which seeds the matching satkers + siman_aset (the
-- SIMAN assertion uses the E2E-B-2 "Tanah" asset from that file — the only
-- seeded jenis_aset the gateway's 4-category inventory scan actually queries).
--
-- SYNTHETIC, deterministic, no real PII. Idempotent (ON CONFLICT (nip)).
-- ============================================================================

BEGIN;
SET search_path TO integrasi, public;

-- satker_id holds the satker's `api_id`, not its code — see the SHAPE NOTE in
-- seed-multisatker.sql, which seeds the satkers this joins to.
INSERT INTO integrasi.mysimkari_pegawai (nip, nama, satker_id, nama_satker, jabatan, status_pegawai)
SELECT v.nip, v.nama, s.api_id, s.nama_satker, v.jabatan, 'aktif'
  FROM (VALUES
    ('200000000000000001', 'E2E Pegawai Jakpus',  '0200010', 'Jaksa Fungsional'),
    ('200000000000000009', 'E2E Pegawai Bandung', '0300010', 'Kepala Sub Bagian')
  ) AS v(nip, nama, kode_satker, jabatan)
  JOIN integrasi.mysimkari_satker s ON s.kode_satker = v.kode_satker
ON CONFLICT (nip) DO UPDATE SET
  satker_id   = EXCLUDED.satker_id,
  nama_satker = EXCLUDED.nama_satker;

COMMIT;
