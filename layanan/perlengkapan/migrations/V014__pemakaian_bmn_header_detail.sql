-- ============================================================================
-- Migration V014: pemakaian BMN becomes header/detail — every asset is equal
--                 (EXPAND phase; the contract phase is a later migration)
-- ============================================================================
-- One permohonan = one pegawai + a LIST of BMN. Until now the FIRST asset lived
-- in columns on `izin_pemakaian_bmn` and assets two and up lived in
-- `pemakaian_bmn_items`. Two shapes for one thing, and everything written since
-- only ever learned the first:
--
--   * the SK printed ONE asset — `sk_izin_pdf.rs` said so in a comment
--     ("V1 model: single BMN per permit"), so item rows never reached page 2;
--   * `find_booking_conflict` / `find_renewal_conflict` read only the parent,
--     so an asset held as an item looked FREE to the next applicant and could
--     be lent out twice;
--   * `renew_permit` rebuilt the request with `additional_bmn_items: vec![]`,
--     dropping every extra asset on renewal;
--   * the item table could not even hold the data: no no_polisi, no serial
--     number, no period — so a second vehicle had nowhere to record its plate.
--
-- After this migration `pemakaian_bmn_items` is the ONLY place an asset lives
-- as far as the application is concerned.
--
-- EXPAND ONLY. layanan/AGENTS.md: "JANGAN ALTER/DROP destruktif di rilis yang
-- sama dengan kode yang masih bisa di-rollback". An earlier draft of this file
-- dropped the parent's asset columns and the view over them in the same
-- release as the code that stopped reading them, arguing the data was mock and
-- production a pre-Helm relic. That argument is about how much would be lost;
-- the rule is about whether the PREVIOUS release can still read what this one
-- wrote, and it could not. So nothing the previous release reads is removed:
--
--   * the parent's asset columns stay, and are kept equal to item #1 by
--     `sync_izin_pemakaian_compat()` (below), called by the repository after
--     every item write and by seeds. They are a MIRROR, written here and read
--     by nobody in this release's code;
--   * `v_izin_pemakaian_aktif` and the parent CHECKs stay;
--   * the new item columns are NULLABLE, so the previous release can still
--     insert an item row that knows nothing of them.
--
-- The parent's asset NOT NULLs are relaxed, because the header is inserted
-- before its items; a header with no items yet has no asset to mirror. After
-- the transaction commits that only remains true for a permit with no items,
-- which the API cannot produce.
--
-- CONTRACT (a later migration, NOT this one, at least one release after this
-- ships and the rollback window has closed): drop `sync_izin_pemakaian_compat`,
-- the parent asset columns + their indexes + CHECKs, the view; set NOT NULL on
-- the item columns that are nullable here. Shipping it with this change would
-- recreate the problem this phase exists to avoid.
--
-- Period: the item's dates are the authority for that asset. The header keeps
-- `tanggal_mulai`/`tanggal_selesai` as the permit's overall window, DERIVED as
-- MIN/MAX over its items and rewritten by the repository on every item write.
-- That is a denormalisation, deliberately: two dozen indexes, the expiry
-- scheduler and every monitoring query key on the header dates, and a derived
-- column with exactly one writer plus a test pinning the derivation is a
-- smaller risk than rewriting all of them. The single writer is
-- `repository::items::sync_permit_window`; the test is
-- `pemakaian_header_detail_test::the_permit_window_tracks_its_items`.
--
-- One dead constraint is fixed on the way past. `chk_housing_fields` read
--     jenis_bmn <> 'RUMAH_DINAS' OR alamat IS NOT NULL
-- while the sibling CHECK allows only KENDARAAN_BERMOTOR / RUMAH_NEGARA /
-- LAPTOP / LAINNYA. `RUMAH_DINAS` appears nowhere else in the repository, so
-- the left side was always true and the rule could never fire: a rumah negara
-- permit with no address was accepted. Measured, not inferred — inserting one
-- against the old constraint pair returns INSERT 0 1. The item-level
-- replacement names RUMAH_NEGARA and stays NOT VALID, because rows admitted
-- under the dead rule are real history; every write from here on is checked.
-- The other three item CHECKs are VALIDATED below: leaving everything NOT
-- VALID would re-create the defect this migration fixes — a rule that cannot
-- fire. (The parent's dead copy is dropped in the contract migration.)
--
-- Re-applying this file is safe, including after the previous release has
-- written rows in between (a rollback and a roll-forward): every statement is
-- guarded, and the backfill only touches rows still missing their new fields.
-- ============================================================================

-- ---------------------------------------------------------------------------
-- 1. The item table learns every field an asset can carry
-- ---------------------------------------------------------------------------
ALTER TABLE perlengkapan.pemakaian_bmn_items
    ADD COLUMN IF NOT EXISTS urutan            integer,
    ADD COLUMN IF NOT EXISTS jenis_bmn         character varying(50),
    ADD COLUMN IF NOT EXISTS bmn_tipe          character varying(100),
    ADD COLUMN IF NOT EXISTS no_polisi         character varying(20),
    ADD COLUMN IF NOT EXISTS no_bpkb           character varying(50),
    ADD COLUMN IF NOT EXISTS no_rangka         character varying(50),
    ADD COLUMN IF NOT EXISTS no_mesin          character varying(50),
    ADD COLUMN IF NOT EXISTS no_stnk           character varying(50),
    ADD COLUMN IF NOT EXISTS alamat            text,
    ADD COLUMN IF NOT EXISTS lokasi_pemakaian  text,
    ADD COLUMN IF NOT EXISTS luas_tanah        double precision,
    ADD COLUMN IF NOT EXISTS luas_bangunan     double precision,
    ADD COLUMN IF NOT EXISTS serial_number     character varying(100),
    ADD COLUMN IF NOT EXISTS spesifikasi       jsonb,
    ADD COLUMN IF NOT EXISTS tanggal_mulai     date,
    ADD COLUMN IF NOT EXISTS tanggal_selesai   date;

-- ---------------------------------------------------------------------------
-- 2. Move the parent's own asset down, and give the existing items the fields
--    they never had.
--
-- Slot 1 is the asset that was sitting in the parent, which was always the
-- permit's first. It is inserted FIRST, so the items that already exist are
-- numbered after it. A permit that already has a slot 1 (written by this
-- release) is left alone, which is also what makes a second run a no-op.
-- ---------------------------------------------------------------------------
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

-- ---------------------------------------------------------------------------
-- 3. Lock the shape in — except NOT NULL, which waits for the contract phase:
--    the previous release inserts items without these columns.
-- ---------------------------------------------------------------------------
DO $c$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'pemakaian_bmn_items_urutan_key') THEN
        ALTER TABLE perlengkapan.pemakaian_bmn_items
            ADD CONSTRAINT pemakaian_bmn_items_urutan_key UNIQUE (izin_pemakaian_id, urutan);
    END IF;

    IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'chk_item_jenis_bmn') THEN
        ALTER TABLE perlengkapan.pemakaian_bmn_items
            ADD CONSTRAINT chk_item_jenis_bmn CHECK (
                jenis_bmn::text = ANY (ARRAY[
                    'KENDARAAN_BERMOTOR', 'RUMAH_NEGARA', 'LAPTOP', 'LAINNYA'
                ]::text[])
            ) NOT VALID;
    END IF;

    IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'chk_item_tanggal_valid') THEN
        ALTER TABLE perlengkapan.pemakaian_bmn_items
            ADD CONSTRAINT chk_item_tanggal_valid
            CHECK (tanggal_selesai >= tanggal_mulai) NOT VALID;
    END IF;

    IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'chk_item_vehicle_fields') THEN
        ALTER TABLE perlengkapan.pemakaian_bmn_items
            ADD CONSTRAINT chk_item_vehicle_fields CHECK (
                jenis_bmn::text <> 'KENDARAAN_BERMOTOR'
                OR (no_polisi IS NOT NULL AND no_bpkb IS NOT NULL)
            ) NOT VALID;
    END IF;

    -- The rule the parent's `chk_housing_fields` was meant to be.
    IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'chk_item_housing_fields') THEN
        ALTER TABLE perlengkapan.pemakaian_bmn_items
            ADD CONSTRAINT chk_item_housing_fields CHECK (
                jenis_bmn::text <> 'RUMAH_NEGARA' OR alamat IS NOT NULL
            ) NOT VALID;
    END IF;
END
$c$;

-- Validate the three that hold over every migrated row. Only the housing rule
-- stays NOT VALID (see the header).
ALTER TABLE perlengkapan.pemakaian_bmn_items VALIDATE CONSTRAINT chk_item_jenis_bmn;
ALTER TABLE perlengkapan.pemakaian_bmn_items VALIDATE CONSTRAINT chk_item_tanggal_valid;
ALTER TABLE perlengkapan.pemakaian_bmn_items VALIDATE CONSTRAINT chk_item_vehicle_fields;

CREATE INDEX IF NOT EXISTS idx_pemakaian_items_jenis
    ON perlengkapan.pemakaian_bmn_items (jenis_bmn);
CREATE INDEX IF NOT EXISTS idx_pemakaian_items_no_polisi
    ON perlengkapan.pemakaian_bmn_items (no_polisi) WHERE no_polisi IS NOT NULL;
-- The asset identity is (kode satker, kode barang, NUP): 44 017 assets carry
-- NUP `1`. The satker half lives on the parent, so the conflict query joins;
-- this index serves the two columns the item side contributes.
CREATE INDEX IF NOT EXISTS idx_pemakaian_items_identitas
    ON perlengkapan.pemakaian_bmn_items (bmn_kode_barang, bmn_nup);

-- ---------------------------------------------------------------------------
-- 4. The parent keeps an asset only as a MIRROR of item #1
-- ---------------------------------------------------------------------------
-- Relaxed so the header can be inserted before its items (see the header).
ALTER TABLE perlengkapan.izin_pemakaian_bmn
    ALTER COLUMN jenis_bmn       DROP NOT NULL,
    ALTER COLUMN bmn_nup         DROP NOT NULL,
    ALTER COLUMN bmn_kode_barang DROP NOT NULL,
    ALTER COLUMN bmn_nama_barang DROP NOT NULL;

-- One definition of the mirror, in SQL, because two kinds of caller need it:
-- the repository (after every item write) and seed files, which insert a
-- header and its items directly. A copy of this column list in each would
-- drift the first time an asset column is added.
CREATE OR REPLACE FUNCTION perlengkapan.sync_izin_pemakaian_compat(p_izin_id uuid)
RETURNS void
LANGUAGE sql
AS $f$
    UPDATE perlengkapan.izin_pemakaian_bmn p
    SET jenis_bmn           = i.jenis_bmn,
        bmn_nup             = i.bmn_nup,
        bmn_kode_barang     = i.bmn_kode_barang,
        bmn_nama_barang     = i.bmn_nama_barang,
        bmn_merk            = i.bmn_merk,
        bmn_tipe            = i.bmn_tipe,
        bmn_tahun_perolehan = i.bmn_tahun_perolehan,
        no_polisi           = i.no_polisi,
        no_bpkb             = i.no_bpkb,
        no_rangka           = i.no_rangka,
        no_mesin            = i.no_mesin,
        no_stnk             = i.no_stnk,
        alamat              = i.alamat,
        luas_tanah          = i.luas_tanah,
        luas_bangunan       = i.luas_bangunan,
        serial_number       = i.serial_number,
        spesifikasi         = i.spesifikasi,
        lokasi_pemakaian    = i.lokasi_pemakaian
    FROM perlengkapan.pemakaian_bmn_items i
    WHERE p.id = p_izin_id
      AND i.izin_pemakaian_id = p.id
      AND i.urutan = 1
$f$;

COMMENT ON FUNCTION perlengkapan.sync_izin_pemakaian_compat(uuid) IS
    'Menyalin BMN urutan 1 ke kolom aset lama di izin_pemakaian_bmn agar rilis '
    'sebelumnya tetap bisa membacanya (V014, fase expand). Dibuang bersama '
    'kolomnya pada migrasi contract.';

COMMENT ON COLUMN perlengkapan.izin_pemakaian_bmn.bmn_nup IS
    'DEPRECATED (V014): cermin dari pemakaian_bmn_items urutan 1, dijaga oleh '
    'sync_izin_pemakaian_compat(). Jangan dibaca — sumber kebenaran ada di '
    'pemakaian_bmn_items. Berlaku sama untuk kolom aset lain di tabel ini '
    '(jenis_bmn … lokasi_pemakaian). Dibuang pada migrasi contract.';

COMMENT ON COLUMN perlengkapan.izin_pemakaian_bmn.tanggal_mulai IS
    'Jendela izin secara keseluruhan = MIN(item.tanggal_mulai). DITURUNKAN, '
    'bukan sumber kebenaran: jangka waktu tiap BMN ada di pemakaian_bmn_items. '
    'Satu-satunya penulis = repository::items::sync_permit_window (V014).';
COMMENT ON COLUMN perlengkapan.izin_pemakaian_bmn.tanggal_selesai IS
    'Jendela izin secara keseluruhan = MAX(item.tanggal_selesai). DITURUNKAN — '
    'lihat komentar pada tanggal_mulai. Dipakai penjadwal kedaluwarsa dan '
    'seluruh kueri monitoring (V014).';

-- ---------------------------------------------------------------------------
-- 5. Supersede: a renewal issues a NEW SK that replaces the old one
-- ---------------------------------------------------------------------------
-- The signing date of a permit differs from its renewal's by definition, so a
-- signed SK is never edited — the newer one takes effect and the older is
-- revoked as replaced. `previous_permit_id` already pointed backwards; this is
-- the forward link, so "which permit is in force for this asset today" is one
-- hop rather than a search.
ALTER TABLE perlengkapan.izin_pemakaian_bmn
    ADD COLUMN IF NOT EXISTS superseded_by_id uuid,
    ADD COLUMN IF NOT EXISTS superseded_at    timestamp with time zone;

DO $s$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'izin_pemakaian_superseded_by_fkey') THEN
        ALTER TABLE perlengkapan.izin_pemakaian_bmn
            ADD CONSTRAINT izin_pemakaian_superseded_by_fkey
            FOREIGN KEY (superseded_by_id)
            REFERENCES perlengkapan.izin_pemakaian_bmn(id) ON DELETE SET NULL;
    END IF;
END
$s$;

CREATE INDEX IF NOT EXISTS idx_izin_superseded_by
    ON perlengkapan.izin_pemakaian_bmn (superseded_by_id)
    WHERE superseded_by_id IS NOT NULL;

COMMENT ON COLUMN perlengkapan.izin_pemakaian_bmn.superseded_by_id IS
    'Izin yang menggantikan baris ini (perpanjangan). Terisi saat izin baru '
    'menjadi ACTIVE; baris ini serentak menjadi REVOKED dengan catatan '
    '"Digantikan oleh izin …". Kebalikan arah dari previous_permit_id (V014).';

-- The withdrawal happens IN THE SAME STATEMENT as the activation, as a trigger.
--
-- The workflow engine opens its own connection, writes `status`, and commits
-- before the caller regains control, so a caller cannot put "activate the
-- renewal" and "revoke the permit it replaces" into one transaction. The
-- first version did them as two commits; if the second failed, both permits
-- stayed ACTIVE over the same assets while the client was told it was an
-- error. Here the two row changes succeed or fail together, and every path to
-- ACTIVE gets it — the normal activation, the audited break-glass override,
-- and anything that updates the column directly.
--
-- Two renewals of one permit cannot both win. The first locks and revokes the
-- predecessor; the second's UPDATE waits for that lock, finds the predecessor
-- no longer ACTIVE/EXPIRED, sees it was superseded by someone else, and
-- raises — which aborts the second activation. A predecessor that was revoked
-- BY HAND (superseded_by_id is null) is not a conflict: the renewal simply
-- activates.
--
-- `status_kode` 3006 is PemakaianBmnStatus::Revoked; the denormalised code is
-- set here because the application only syncs it for the row it transitions.
CREATE OR REPLACE FUNCTION perlengkapan.supersede_previous_permit()
RETURNS trigger
LANGUAGE plpgsql
AS $f$
DECLARE
    v_diganti   integer;
    v_pengganti uuid;
BEGIN
    UPDATE perlengkapan.izin_pemakaian_bmn
       SET status             = 'REVOKED',
           status_kode        = 3006,
           superseded_by_id   = NEW.id,
           superseded_at      = now(),
           revoked_by_nama    = 'system',
           revoked_at         = now(),
           catatan_revocation = 'Digantikan oleh izin ' || COALESCE(NEW.nomor_izin, NEW.id::text),
           updated_at         = now()
     WHERE id = NEW.previous_permit_id
       AND status IN ('ACTIVE', 'EXPIRED');
    GET DIAGNOSTICS v_diganti = ROW_COUNT;

    IF v_diganti = 0 THEN
        SELECT superseded_by_id INTO v_pengganti
          FROM perlengkapan.izin_pemakaian_bmn
         WHERE id = NEW.previous_permit_id;
        IF v_pengganti IS NOT NULL AND v_pengganti <> NEW.id THEN
            RAISE EXCEPTION 'Izin % sudah digantikan oleh izin % dan tidak dapat digantikan oleh izin %',
                NEW.previous_permit_id, v_pengganti, NEW.id;
        END IF;
    END IF;

    RETURN NULL;
END
$f$;

DROP TRIGGER IF EXISTS trg_izin_pemakaian_supersede ON perlengkapan.izin_pemakaian_bmn;
CREATE TRIGGER trg_izin_pemakaian_supersede
    AFTER UPDATE OF status ON perlengkapan.izin_pemakaian_bmn
    FOR EACH ROW
    WHEN (NEW.status = 'ACTIVE'
          AND OLD.status IS DISTINCT FROM 'ACTIVE'
          AND NEW.is_renewal IS TRUE
          AND NEW.previous_permit_id IS NOT NULL)
    EXECUTE FUNCTION perlengkapan.supersede_previous_permit();
