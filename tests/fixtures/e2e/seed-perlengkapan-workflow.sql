-- ============================================================================
-- E2E perlengkapan workflow fixture (F-E2E E-1/E-2/E-3)
-- ============================================================================
-- Business-workflow preconditions for the comprehensive Playwright suite:
-- Kebutuhan BMN (E-1), Pakaian Dinas (E-2), Penghapusan + Pemakaian BMN (E-3).
-- SYNTHETIC, deterministic data only — no real PII.
--
-- SPLIT FROM seed-multisatker.sql (which is applied by EVERY e2e job): these
-- rows live in the `perlengkapan` schema, which only exists once
-- layanan-perlengkapan has run its refinery migrations. Jobs whose stack has no
-- layanan-perlengkapan (e2e-portal, e2e-integrasi-authenc,
-- e2e-simpelv1-integration) previously failed the shared seed outright with
-- `relation "perlengkapan.pengajuan_kebutuhan_bmn" does not exist`. Apply this
-- file ONLY in jobs that bring layanan-perlengkapan up, AFTER
-- seed-multisatker.sql (it references the users/satkers seeded there).
--
-- Idempotent: PK rows use ON CONFLICT DO NOTHING.
-- ============================================================================

BEGIN;

-- Rows below are schema-qualified, but perlengkapan triggers/defaults may
-- resolve unqualified relations via search_path (see seed-multisatker.sql).
SET search_path TO perlengkapan, authenc, integrasi, public;

-- ----------------------------------------------------------------------------
-- 5. perlengkapan: Kebutuhan BMN workflow preconditions (F-E2E E-1).
--    ONE open campaign (periode, status 2000) + per-satker response rows seeded
--    at DISTINCT workflow statuses so each role's transition is exercised
--    independently & deterministically (rather than one brittle long chain).
--    The satker-detail component (/kebutuhan-bmn/satker/:id) gates actions by
--    status_kode: operator submits at 2000/2001; validator_wilayah forwards at
--    2002; validator_pusat decides at 2005.
--      S1 0200010 @2001 Input Barang  -> operator_a: Tambah Barang + Submit ke Wilayah
--      S2 0200020 @2002 →Wilayah      -> validator_wilayah (DKI): Teruskan ke Pusat
--      S3 0300010 @2005 Analisis      -> validator_pusat: Setujui
--    ms_workflow_status(kebutuhan_bmn): 2000 Draft, 2001 Input, 2002 →Wilayah,
--    2003 Revisi, 2004 →Pusat, 2005 Analisis, 2006 Disetujui, 2007 Ditolak.
--    Known UUIDs so specs can deep-link to the satker detail without list-click.
-- ----------------------------------------------------------------------------
INSERT INTO perlengkapan.pengajuan_kebutuhan_bmn
  (id, nama, deskripsi, tahun, tgl_mulai, tgl_selesai, pilihan_satker, status_kode, created_by, scope_satker)
VALUES
  ('c1000000-0000-4c00-8c00-000000000001', 'E2E Periode Kebutuhan BMN 2026', 'Seed F-E2E kebutuhan workflow', 2026, '2026-01-01', '2026-12-31', 'semua', 2000, '44444444-4444-4444-8444-444444444444', 'semua')
ON CONFLICT (id) DO NOTHING;

INSERT INTO perlengkapan.pengajuan_kebutuhan_bmn_satker
  (id, pengajuan_id, satker_id, satker_nama, status_kode, created_by)
VALUES
  ('c1000000-0000-4c00-8c00-0000000a0001', 'c1000000-0000-4c00-8c00-000000000001', '0200010', 'KEJAKSAAN NEGERI JAKARTA PUSAT',   2001, '11111111-1111-4111-8111-111111111111'),
  ('c1000000-0000-4c00-8c00-0000000a0002', 'c1000000-0000-4c00-8c00-000000000001', '0200020', 'KEJAKSAAN NEGERI JAKARTA SELATAN', 2002, '22222222-2222-4222-8222-222222222222'),
  ('c1000000-0000-4c00-8c00-0000000a0003', 'c1000000-0000-4c00-8c00-000000000001', '0300010', 'KEJAKSAAN NEGERI BANDUNG',         2005, '44444444-4444-4444-8444-444444444444')
ON CONFLICT (id) DO NOTHING;

-- One pre-existing barang per satker row so (a) the operator's "Submit ke
-- Wilayah" is valid without depending on the add-barang modal, and (b) the
-- wilayah/pusat detail views are non-empty. Operator ALSO adds one via the UI
-- modal during the test (asserting the count goes 1 -> 2).
INSERT INTO perlengkapan.pengajuan_kebutuhan_bmn_satker_barang
  (id, pengajuan_satker_id, nama, kode_barang, jumlah, satuan, alasan)
VALUES
  ('c1000000-0000-4c00-8c00-00000000b001', 'c1000000-0000-4c00-8c00-0000000a0001', 'E2E Barang Seed A', '3.10.01.02.003', 3, 'Unit', 'Seed justifikasi kebutuhan'),
  ('c1000000-0000-4c00-8c00-00000000b002', 'c1000000-0000-4c00-8c00-0000000a0002', 'E2E Barang Seed B', '3.05.02.01.002', 2, 'Unit', 'Seed justifikasi kebutuhan'),
  ('c1000000-0000-4c00-8c00-00000000b003', 'c1000000-0000-4c00-8c00-0000000a0003', 'E2E Barang Seed C', '3.10.01.05.010', 1, 'Unit', 'Seed justifikasi kebutuhan')
ON CONFLICT (id) DO NOTHING;

-- ----------------------------------------------------------------------------
-- 6. perlengkapan: Pakaian Dinas workflow preconditions (F-E2E E-2).
--    One jenis (master) + one open campaign targeting the DKI + Bandung satkers,
--    with per-satker rows at DISTINCT aktivitas so each validator tier's action
--    is exercised independently. `satker_id` is the MySIMKARI `kode_satker`
--    (V006/#94) — the same key kebutuhan uses — so it is written literally here
--    rather than resolved through a surrogate id.
--    aktivitas (ms_aktivitas_bmn 1000-series): 1000 INPUT, 1001 SUBMIT_TO_VALIDATOR
--    (validator_wilayah acts), 1004 SUBMIT_TO_PUSAT (validator_pusat acts), 1008 SELESAI.
--      P1 0200010 @1001 -> validator_wilayah (DKI): Teruskan ke Pusat
--      P2 0200020 @1004 -> validator_pusat: decide
-- ----------------------------------------------------------------------------
INSERT INTO perlengkapan.ms_jenis_pakaian_dinas (id, nama, deskripsi, is_active)
VALUES ('d1000000-0000-4d00-8d00-000000000001', 'PDH E2E', 'Seed F-E2E pakaian dinas', true)
ON CONFLICT (id) DO NOTHING;

INSERT INTO perlengkapan.pengajuan_pakaian_dinas
  (id, nama, tahun, pilihan_satker, scope_satker, jenis_pakaian_dinas_id, aktivitas_id, created_by, tgl_mulai, tgl_selesai)
VALUES
  ('d1000000-0000-4d00-8d00-0000000000c1', 'E2E Pengajuan Pakaian Dinas 2026', 2026, 'semua', 'semua', 'd1000000-0000-4d00-8d00-000000000001', 1000, '44444444-4444-4444-8444-444444444444', '2026-01-01', '2026-12-31')
ON CONFLICT (id) DO NOTHING;

-- Per-satker rows (restored by V006/#94 — they could not be seeded before,
-- because `satker_id` was a uuid the BE compared against a bigint PK).
INSERT INTO perlengkapan.pengajuan_pakaian_dinas_satker
  (id, pengajuan_id, satker_id, aktivitas_id, created_by)
VALUES
  ('d1000000-0000-4d00-8d00-0000000a0001', 'd1000000-0000-4d00-8d00-0000000000c1', '0200010', 1001, '11111111-1111-4111-8111-111111111111'),
  ('d1000000-0000-4d00-8d00-0000000a0002', 'd1000000-0000-4d00-8d00-0000000000c1', '0200020', 1004, '22222222-2222-4222-8222-222222222222')
ON CONFLICT (id) DO NOTHING;

-- The campaign explicitly targets both satkers, so the scope.rs satker tier has
-- a row to match (its predicate now compares kode_satker directly).
INSERT INTO perlengkapan.pengajuan_pakaian_dinas_satker_terpilih
  (pengajuan_id, satker_id, is_show_in_form)
VALUES
  ('d1000000-0000-4d00-8d00-0000000000c1', '0200010', true),
  ('d1000000-0000-4d00-8d00-0000000000c1', '0200020', true)
ON CONFLICT (pengajuan_id, satker_id) DO NOTHING;

-- One employee row per satker so the rekap/daftar reports have something to
-- aggregate over (they JOIN pengajuan_pakaian_dinas_satker → mysimkari_satker).
INSERT INTO perlengkapan.pengajuan_pakaian_dinas_satker_pegawai
  (id, pengajuan_satker_id, nip, nama, jenis_kelamin, jabatan, pangkat)
VALUES
  ('d1000000-0000-4d00-8d00-00000000e001', 'd1000000-0000-4d00-8d00-0000000a0001', '200000000000000001', 'E2E Operator Jakpus', 'L', 'Operator Satker', 'Penata Muda'),
  ('d1000000-0000-4d00-8d00-00000000e002', 'd1000000-0000-4d00-8d00-0000000a0002', '200000000000000002', 'E2E Operator Jaksel', 'P', 'Operator Satker', 'Penata Muda')
ON CONFLICT (id) DO NOTHING;

-- ----------------------------------------------------------------------------
-- 6b. perlengkapan: a SECOND campaign, for the laporan page only.
--
--     The reports read `..._satker` rows at aktivitas 1008 (SELESAI) and JOIN
--     through to the size rows. Campaign 6 above has neither: its satker rows
--     sit at 1001/1004 precisely so the workflow tests can advance them, and it
--     carries no `..._pakaian` or `..._pegawai_ukuran` rows at all. Every
--     laporan query against it therefore returns zero rows — the tabs render
--     empty and prove nothing.
--
--     It cannot simply be extended. `pakaian-workflow.spec.ts` asserts the
--     campaign's satker code list with `toEqual`, so extra rows break it; and
--     the workflow tests move P2 to 1008 mid-run, which would make the report
--     contents depend on test order. Hence a separate, static campaign.
--
--     Shape chosen so each filter can be shown to NARROW, not merely to parse:
--       2 satker  × 2 jenis pakaian × 1 pegawai each
--       satker 0200010 → pegawai L, satker 0200020 → pegawai P
--     so `satker_id` halves the rows and flips the gender split, and
--     `jenis_pakaian_id` halves them along the other axis.
-- ----------------------------------------------------------------------------
INSERT INTO perlengkapan.ms_jenis_pakaian_dinas (id, nama, deskripsi, is_active)
VALUES ('d1000000-0000-4d00-8d00-000000000002', 'PDL E2E', 'Seed F-E2E pakaian dinas (jenis kedua)', true)
ON CONFLICT (id) DO NOTHING;

INSERT INTO perlengkapan.pengajuan_pakaian_dinas
  (id, nama, tahun, pilihan_satker, scope_satker, jenis_pakaian_dinas_id, aktivitas_id, created_by, tgl_mulai, tgl_selesai)
VALUES
  ('d1000000-0000-4d00-8d00-0000000000c2', 'E2E Laporan Pakaian Dinas 2026', 2026, 'semua', 'semua', 'd1000000-0000-4d00-8d00-000000000001', 1008, '44444444-4444-4444-8444-444444444444', '2026-01-01', '2026-12-31')
ON CONFLICT (id) DO NOTHING;

INSERT INTO perlengkapan.pengajuan_pakaian_dinas_satker_terpilih
  (pengajuan_id, satker_id, is_show_in_form)
VALUES
  ('d1000000-0000-4d00-8d00-0000000000c2', '0200010', true),
  ('d1000000-0000-4d00-8d00-0000000000c2', '0200020', true)
ON CONFLICT (pengajuan_id, satker_id) DO NOTHING;

-- Both at 1008: the reports only count satkers that finished.
INSERT INTO perlengkapan.pengajuan_pakaian_dinas_satker
  (id, pengajuan_id, satker_id, aktivitas_id, created_by)
VALUES
  ('d1000000-0000-4d00-8d00-0000000a1001', 'd1000000-0000-4d00-8d00-0000000000c2', '0200010', 1008, '11111111-1111-4111-8111-111111111111'),
  ('d1000000-0000-4d00-8d00-0000000a1002', 'd1000000-0000-4d00-8d00-0000000000c2', '0200020', 1008, '22222222-2222-4222-8222-222222222222')
ON CONFLICT (id) DO NOTHING;

INSERT INTO perlengkapan.pengajuan_pakaian_dinas_satker_pegawai
  (id, pengajuan_satker_id, nip, nama, jenis_kelamin, jabatan, pangkat, eselon, jenis)
VALUES
  ('d1000000-0000-4d00-8d00-00000000e101', 'd1000000-0000-4d00-8d00-0000000a1001', '200000000000000101', 'E2E Laporan Jakpus', 'L', 'Operator Satker', 'Penata Muda', 'IV', '0'),
  ('d1000000-0000-4d00-8d00-00000000e102', 'd1000000-0000-4d00-8d00-0000000a1002', '200000000000000102', 'E2E Laporan Jaksel', 'P', 'Operator Satker', 'Penata Muda', 'IV', '0')
ON CONFLICT (id) DO NOTHING;

-- The campaign's clothing items. `jenis_pakaian_nama` is denormalised here by
-- design — the report header resolves the selected type's label from it.
INSERT INTO perlengkapan.pengajuan_pakaian_dinas_pakaian
  (id, pengajuan_id, jenis_pakaian_id, jenis_pakaian_nama, spesifikasi_id, spesifikasi_nama, spesifikasi_ukuran_group)
VALUES
  ('d1000000-0000-4d00-8d00-00000000b001', 'd1000000-0000-4d00-8d00-0000000000c2', 'd1000000-0000-4d00-8d00-000000000001', 'PDH E2E', 'd1000000-0000-4d00-8d00-00000000f001', 'Kemeja PDH E2E', 'BAJU'),
  ('d1000000-0000-4d00-8d00-00000000b002', 'd1000000-0000-4d00-8d00-0000000000c2', 'd1000000-0000-4d00-8d00-000000000002', 'PDL E2E', 'd1000000-0000-4d00-8d00-00000000f002', 'Kemeja PDL E2E', 'BAJU')
ON CONFLICT (id) DO NOTHING;

-- Every (pegawai × item) has a size, so an unfiltered rekap has 2 rows per item
-- and each filter can be seen to cut the count.
INSERT INTO perlengkapan.pengajuan_pakaian_dinas_satker_pegawai_ukuran
  (pengajuan_satker_id, pegawai_id, pakaian_id, ukuran)
VALUES
  ('d1000000-0000-4d00-8d00-0000000a1001', 'd1000000-0000-4d00-8d00-00000000e101', 'd1000000-0000-4d00-8d00-00000000b001', 'L'),
  ('d1000000-0000-4d00-8d00-0000000a1001', 'd1000000-0000-4d00-8d00-00000000e101', 'd1000000-0000-4d00-8d00-00000000b002', 'L'),
  ('d1000000-0000-4d00-8d00-0000000a1002', 'd1000000-0000-4d00-8d00-00000000e102', 'd1000000-0000-4d00-8d00-00000000b001', 'M'),
  ('d1000000-0000-4d00-8d00-0000000a1002', 'd1000000-0000-4d00-8d00-00000000e102', 'd1000000-0000-4d00-8d00-00000000b002', 'M')
ON CONFLICT (pegawai_id, pakaian_id) DO NOTHING;

-- ----------------------------------------------------------------------------
-- 7. perlengkapan: Penghapusan BMN workflow preconditions (F-E2E E-3).
--    Usulan rows at DISTINCT statuses (known UUIDs, deep-linked by the spec) so
--    each role's transition is exercised independently. RBAC visibility is the
--    authoritative MySIMKARI `satker_code` (V003, SatkerScope) — NO mysimkari
--    join, so this module is #94-safe. `satker_id`/`asset_id` are free uuids
--    (no FK).
--    Status model (4000-series): 4000 DRAFT → 4001 SUBMIT_WILAYAH →
--    4003 SUBMIT_PUSAT → 4004 VERIFIKASI_PUSAT → 4005 KONSEP_SK_GENERATED →
--    4006 SK_SIGNED → 4007 COMPLETED.
--      H1 0200010 @4000 -> operator_a: "Ajukan ke Validator Wilayah"
--      H2 0200010 @4001 -> validator_wilayah (DKI): "Teruskan ke Validator Pusat"
--      H3 0200020 @4003 -> validator_pusat: verifikasi (API; FE gap) + Generate SK (UI)
-- ----------------------------------------------------------------------------
INSERT INTO perlengkapan.penghapusan_bmn
  (id, satker_id, asset_id, kode_barang, nama_barang, nup,
   tanggal_penghapusan, alasan, metode_penghapusan, nilai_perolehan,
   status, status_kode, catatan_operator, kewenangan_penetap_sk,
   created_by, satker_code)
VALUES
  ('e1000000-0000-4e00-8e00-0000000a0001', 'e1a00000-0000-4e00-8e00-000000000001', 'e1b00000-0000-4e00-8e00-000000000001', '3.10.01.02.003', 'E2E Laptop Hapus A', 'E2E-H-1',
   '2026-06-01', 'Rusak berat, tidak ekonomis diperbaiki', 'DIMUSNAHKAN', 15000000,
   'DRAFT', 4000, 'Seed F-E2E penghapusan (operator step)', 'PUSAT',
   '11111111-1111-4111-8111-111111111111', '0200010'),
  ('e1000000-0000-4e00-8e00-0000000a0002', 'e1a00000-0000-4e00-8e00-000000000002', 'e1b00000-0000-4e00-8e00-000000000002', '3.10.01.05.010', 'E2E Printer Hapus B', 'E2E-H-2',
   '2026-06-01', 'Rusak berat, biaya perbaikan melebihi nilai', 'DIMUSNAHKAN', 4000000,
   'SUBMIT_WILAYAH', 4001, 'Seed F-E2E penghapusan (wilayah step)', 'PUSAT',
   '11111111-1111-4111-8111-111111111111', '0200010'),
  ('e1000000-0000-4e00-8e00-0000000a0003', 'e1a00000-0000-4e00-8e00-000000000003', 'e1b00000-0000-4e00-8e00-000000000003', '3.05.02.01.002', 'E2E Motor Hapus C', 'E2E-H-3',
   '2026-06-01', 'Usia teknis terlampaui, akan dilelang', 'DIJUAL', 22000000,
   'SUBMIT_PUSAT', 4003, 'Seed F-E2E penghapusan (pusat step)', 'PUSAT',
   '22222222-2222-4222-8222-222222222222', '0200020')
ON CONFLICT (id) DO NOTHING;

-- One item per usulan (Fase 2.8 multi-item; the seed bypasses the create
-- endpoint that would normally insert these, and the detail + generated SK
-- lampiran render from this table).
INSERT INTO perlengkapan.penghapusan_bmn_item
  (id, penghapusan_id, asset_id, kode_barang, nama_barang, nup, nilai_perolehan, kondisi, urutan)
VALUES
  ('e1c00000-0000-4e00-8e00-000000000001', 'e1000000-0000-4e00-8e00-0000000a0001', 'e1b00000-0000-4e00-8e00-000000000001', '3.10.01.02.003', 'E2E Laptop Hapus A', 'E2E-H-1', 15000000, 'RUSAK BERAT', 1),
  ('e1c00000-0000-4e00-8e00-000000000002', 'e1000000-0000-4e00-8e00-0000000a0002', 'e1b00000-0000-4e00-8e00-000000000002', '3.10.01.05.010', 'E2E Printer Hapus B', 'E2E-H-2', 4000000, 'RUSAK BERAT', 1),
  ('e1c00000-0000-4e00-8e00-000000000003', 'e1000000-0000-4e00-8e00-0000000a0003', 'e1b00000-0000-4e00-8e00-000000000003', '3.05.02.01.002', 'E2E Motor Hapus C', 'E2E-H-3', 22000000, 'RUSAK RINGAN', 1)
ON CONFLICT (id) DO NOTHING;

-- ----------------------------------------------------------------------------
-- 8. perlengkapan: Izin Pemakaian BMN precondition (F-E2E E-3).
--    ONE ACTIVE (3004) izin at satker 0200010. Also satker_code-scoped (V003,
--    #94-safe). The satker-internal approval chain
--    (Submitted→SubmittedApproverSatker→Approved) is now reachable: #96 seeded
--    the `validator_satker` / `approver_satker` roles the policy requires
--    (authenc migration 004), gave two 0200010 users those roles
--    (seed-multisatker.sql), and wired the FE buttons. This row stays ACTIVE so
--    E-3 keeps covering LIST/DETAIL render + scoping + the revoke policy
--    rejections (approver-only, admin explicitly blocked); the chain itself is
--    driven from a DRAFT permit created by the spec.
--    jenis_bmn LAPTOP avoids the vehicle/housing CHECK constraints.
-- ----------------------------------------------------------------------------
INSERT INTO perlengkapan.izin_pemakaian_bmn
  (id, nomor_izin, jenis_bmn, bmn_nup, bmn_kode_barang, bmn_nama_barang,
   serial_number, pegawai_nip, pegawai_nama, pegawai_jabatan,
   pegawai_satker_id, pegawai_satker_nama, tanggal_mulai, tanggal_selesai,
   status, status_kode, keperluan, satker_code)
VALUES
  ('e2000000-0000-4e00-8e00-0000000b0001', 'E2E-IZIN-001', 'LAPTOP', 'E2E-A-2', '3.10.01.02.003', 'E2E Laptop Dinas Pinjam',
   'SN-E2E-0001', '200000000000000001', 'E2E Operator Jakpus', 'Operator Satker',
   'e2a00000-0000-4e00-8e00-000000000001', 'KEJAKSAAN NEGERI JAKARTA PUSAT', '2026-01-01', '2026-12-31',
   'ACTIVE', 3004, 'Penunjang tugas kedinasan harian', '0200010'),
  -- I2 @SUBMITTED(3001): the entry point of the satker approval chain (#96).
  -- validator_satker forwards it, then approver_satker approves it, both
  -- through the UI.
  ('e2000000-0000-4e00-8e00-0000000b0002', 'E2E-IZIN-002', 'LAPTOP', 'E2E-A-1', '3.10.01.02.003', 'E2E Laptop Ajuan Satker',
   'SN-E2E-0002', '200000000000000001', 'E2E Operator Jakpus', 'Operator Satker',
   'e2a00000-0000-4e00-8e00-000000000001', 'KEJAKSAAN NEGERI JAKARTA PUSAT', '2026-02-01', '2026-11-30',
   'SUBMITTED', 3001, 'Penunjang tugas operasional', '0200010'),
  -- I3 @ACTIVE, satker 0200020 (operator_b). Without a permit in a SECOND
  -- satker every cross-satker monitoring assertion is vacuous: one satker's
  -- rows and "all rows" are the same set, so a scope that does nothing passes.
  -- 0200020 shares operator_a's wilayah (both kdsatker_keu carry 9999), which
  -- is what makes the wilayah tier distinguishable from the satker tier.
  ('e2000000-0000-4e00-8e00-0000000b0003', 'E2E-IZIN-003', 'LAPTOP', 'E2E-B-1', '3.10.01.02.003', 'E2E Laptop Jaksel',
   'SN-E2E-0003', '200000000000000002', 'E2E Operator Jaksel', 'Operator Satker',
   'e2a00000-0000-4e00-8e00-000000000002', 'KEJAKSAAN NEGERI JAKARTA SELATAN', '2026-03-01', '2026-10-31',
   'ACTIVE', 3004, 'Penunjang tugas kedinasan Jaksel', '0200020')
ON CONFLICT (id) DO NOTHING;

-- ----------------------------------------------------------------------------
-- 9. notifikasi: per-user in-app inbox (F-E2E E-4).
--    The inbox is scoped by `claims.user_id` (notifikasi/api.rs: get_notifications
--    /mark_as_read/mark_all_read all take claims.user_id) — NOT by satker. So the
--    fixture gives operator_a a mixed read/unread inbox and operator_b one row of
--    its own, which makes BOTH assertions possible:
--      - read-state transitions (Tandai dibaca / Tandai semua dibaca)
--      - per-user isolation (operator_b never sees operator_a's rows, and cannot
--        mark them read — mark_as_read is keyed on (id, user_id))
--    N1 + N2 unread, N3 already read (so "Hanya yang belum dibaca" filter is
--    observable), N4 belongs to operator_b.
-- ----------------------------------------------------------------------------
INSERT INTO notifikasi.in_app_notifications
  (id, user_id, notification_type, title, message, priority, category, action_url, read, read_at)
VALUES
  ('f1000000-0000-4f00-8f00-0000000c0001', '11111111-1111-4111-8111-111111111111',
   'workflow', 'E2E Notifikasi Satu', 'Usulan penghapusan menunggu tindakan Anda.',
   'normal', 'info', '/perlengkapan/simpel/v2/pengelolaan/penghapusan', false, NULL),
  ('f1000000-0000-4f00-8f00-0000000c0002', '11111111-1111-4111-8111-111111111111',
   'workflow', 'E2E Notifikasi Dua', 'Periode kebutuhan BMN 2026 telah dibuka.',
   'high', 'warning', NULL, false, NULL),
  ('f1000000-0000-4f00-8f00-0000000c0003', '11111111-1111-4111-8111-111111111111',
   'sistem', 'E2E Notifikasi Terbaca', 'Notifikasi ini sudah dibaca sejak awal.',
   'normal', 'info', NULL, true, now()),
  ('f1000000-0000-4f00-8f00-0000000c0004', '22222222-2222-4222-8222-222222222222',
   'workflow', 'E2E Notifikasi Operator B', 'Hanya untuk operator_b (isolasi per-user).',
   'normal', 'info', NULL, false, NULL)
ON CONFLICT (id) DO NOTHING;

COMMIT;
