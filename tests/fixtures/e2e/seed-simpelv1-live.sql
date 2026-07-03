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

INSERT INTO integrasi.mysimkari_pegawai (nip, nama, satker_id, nama_satker, jabatan, status_pegawai)
VALUES
  ('200000000000000001', 'E2E Pegawai Jakpus',  '0200010', 'KEJAKSAAN NEGERI JAKARTA PUSAT', 'Jaksa Fungsional',  'aktif'),
  ('200000000000000009', 'E2E Pegawai Bandung', '0300010', 'KEJAKSAAN NEGERI BANDUNG',       'Kepala Sub Bagian', 'aktif')
ON CONFLICT (nip) DO NOTHING;

COMMIT;
