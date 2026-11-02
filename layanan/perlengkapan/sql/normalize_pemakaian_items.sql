-- ============================================================================
-- Normalise pemakaian_bmn_items written by a release that predates V014
-- ============================================================================
-- Run by `migrations::run` on EVERY startup, after refinery and before the
-- service binds a port. Not a refinery migration, on purpose.
--
-- What it is for: rows that EXIST BEFORE the V014 triggers do — the legacy
-- items and the header-held first assets written by a release that predates
-- V014. Right after V014 they have no position and no item row. Refinery runs
-- V014 once, so this is what repairs them, and it must finish before any
-- request reads them: `PemakaianBmnItem::from_row` reads NULL into required
-- fields and panics.
--
-- What it is NOT for: writes that arrive later. During a rolling upgrade old
-- pods keep writing while new pods run, and a startup pass cannot see those.
-- V014 section 6 normalises them AT INSERT with triggers. This file also keeps
-- every start self-healing if some row slips past both.
--
-- Idempotent and cheap: both statements touch only rows still missing their
-- new fields, so a start with nothing to repair writes nothing. The advisory
-- lock serialises replicas that start together; without it two pods could both
-- insert a permit's slot 1 and one would die on the unique constraint.
--
-- Slot 1 is the asset that was sitting in the parent, which was always the
-- permit's first. It is inserted FIRST, so items that already exist are
-- numbered after it. A permit that already has a slot 1 (written by this
-- release) is left alone.
-- ============================================================================
SELECT pg_advisory_xact_lock(hashtext('perlengkapan.normalize_pemakaian_items'));

INSERT INTO perlengkapan.pemakaian_bmn_items (
    izin_pemakaian_id, urutan, jenis_bmn,
    bmn_nup, bmn_kode_barang, bmn_nama_barang, bmn_merk, bmn_tipe,
    bmn_tahun_perolehan,
    no_polisi, no_bpkb, no_rangka, no_mesin, no_stnk,
    alamat, luas_tanah, luas_bangunan,
    serial_number, spesifikasi, lokasi_pemakaian,
    tanggal_mulai, tanggal_selesai,
    created_at, updated_at
)
SELECT p.id, 1, p.jenis_bmn,
       p.bmn_nup, p.bmn_kode_barang, p.bmn_nama_barang, p.bmn_merk, p.bmn_tipe,
       p.bmn_tahun_perolehan,
       p.no_polisi, p.no_bpkb, p.no_rangka, p.no_mesin, p.no_stnk,
       p.alamat, p.luas_tanah, p.luas_bangunan,
       p.serial_number, p.spesifikasi, p.lokasi_pemakaian,
       p.tanggal_mulai, p.tanggal_selesai,
       COALESCE(p.created_at, now()), COALESCE(p.updated_at, now())
FROM perlengkapan.izin_pemakaian_bmn p
WHERE p.bmn_nup IS NOT NULL
  AND NOT EXISTS (
      SELECT 1 FROM perlengkapan.pemakaian_bmn_items it
      WHERE it.izin_pemakaian_id = p.id AND it.urutan = 1
  );

-- Items that have no position yet: the ones the old schema wrote. They keep
-- their order and continue after the highest slot already taken.
--
-- `jenis_bmn` is `LAINNYA`, NOT the parent's jenis. The old item table had no
-- jenis column at all, so there is no per-item value to recover and copying
-- the parent's would label a laptop KENDARAAN_BERMOTOR — a fabrication the
-- vehicle CHECK would then reject for having no plate. LAINNYA is the enum's
-- own "unclassified" member: it records that the old schema never knew, which
-- is the truth. (The first draft of this migration did copy the parent's
-- jenis; backfilling real rows is what caught it.)
UPDATE perlengkapan.pemakaian_bmn_items it
SET urutan           = s.dasar + s.posisi,
    jenis_bmn        = COALESCE(it.jenis_bmn, 'LAINNYA'),
    lokasi_pemakaian = COALESCE(it.lokasi_pemakaian, p.lokasi_pemakaian),
    tanggal_mulai    = COALESCE(it.tanggal_mulai, p.tanggal_mulai),
    tanggal_selesai  = COALESCE(it.tanggal_selesai, p.tanggal_selesai)
FROM (
    SELECT x.id, x.izin_pemakaian_id,
           COALESCE((
               SELECT MAX(n.urutan) FROM perlengkapan.pemakaian_bmn_items n
               WHERE n.izin_pemakaian_id = x.izin_pemakaian_id AND n.urutan IS NOT NULL
           ), 0) AS dasar,
           row_number() OVER (
               PARTITION BY x.izin_pemakaian_id ORDER BY x.created_at, x.id
           ) AS posisi
    FROM perlengkapan.pemakaian_bmn_items x
    WHERE x.urutan IS NULL
) s
JOIN perlengkapan.izin_pemakaian_bmn p ON p.id = s.izin_pemakaian_id
WHERE it.id = s.id;
