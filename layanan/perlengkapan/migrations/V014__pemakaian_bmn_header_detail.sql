-- ============================================================================
-- Migration V014: pemakaian BMN becomes header/detail — every asset is equal
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
-- After this migration `pemakaian_bmn_items` is the ONLY place an asset lives.
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
-- fire.
--
-- Pre-production, and `izin_pemakaian_bmn` in staging holds seeded mock data
-- only, so the parent columns are dropped in this same migration rather than
-- left behind for an expand/contract window. Production is a pre-Helm relic
-- scheduled for wholesale replacement at F6 and is not migrated from.
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
--    they never had. Guarded on the parent column still existing so a replay
--    is a no-op rather than an error.
-- ---------------------------------------------------------------------------
DO $migrasi$
BEGIN
    IF EXISTS (
        SELECT 1 FROM information_schema.columns
        WHERE table_schema = 'perlengkapan'
          AND table_name   = 'izin_pemakaian_bmn'
          AND column_name  = 'bmn_nup'
    ) THEN
        -- Existing item rows keep their order and start at 2: slot 1 is
        -- reserved for the asset that was sitting in the parent, which was
        -- always the permit's first.
        --
        -- `LAINNYA`, NOT the parent's jenis. The old item table had no jenis
        -- column at all, so there is no per-item value to recover and copying
        -- the parent's would label a laptop KENDARAAN_BERMOTOR — a fabrication
        -- the vehicle CHECK would then reject for having no plate. LAINNYA is
        -- the enum's own "unclassified" member: it records that the old schema
        -- never knew, which is the truth. (The first draft of this migration
        -- did copy the parent's jenis; backfilling real rows is what caught it.)
        UPDATE perlengkapan.pemakaian_bmn_items it
        SET urutan           = s.urutan,
            jenis_bmn        = COALESCE(it.jenis_bmn, 'LAINNYA'),
            lokasi_pemakaian = COALESCE(it.lokasi_pemakaian, p.lokasi_pemakaian),
            tanggal_mulai    = COALESCE(it.tanggal_mulai, p.tanggal_mulai),
            tanggal_selesai  = COALESCE(it.tanggal_selesai, p.tanggal_selesai)
        FROM (
            SELECT id, izin_pemakaian_id,
                   1 + row_number() OVER (
                       PARTITION BY izin_pemakaian_id ORDER BY created_at, id
                   ) AS urutan
            FROM perlengkapan.pemakaian_bmn_items
        ) s
        JOIN perlengkapan.izin_pemakaian_bmn p ON p.id = s.izin_pemakaian_id
        WHERE it.id = s.id;

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
        WHERE NOT EXISTS (
            SELECT 1 FROM perlengkapan.pemakaian_bmn_items it
            WHERE it.izin_pemakaian_id = p.id AND it.urutan = 1
        );
    END IF;
END
$migrasi$;

-- ---------------------------------------------------------------------------
-- 3. Lock the shape in
-- ---------------------------------------------------------------------------
ALTER TABLE perlengkapan.pemakaian_bmn_items
    ALTER COLUMN urutan          SET NOT NULL,
    ALTER COLUMN jenis_bmn       SET NOT NULL,
    ALTER COLUMN tanggal_mulai   SET NOT NULL,
    ALTER COLUMN tanggal_selesai SET NOT NULL;

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
-- 4. The parent stops holding an asset
-- ---------------------------------------------------------------------------
-- Zero consumers: nothing in the repository selects from this view. Dropped
-- rather than rebuilt over the items, because rebuilding it would create a
-- second way to ask a question that already has repository methods.
DROP VIEW IF EXISTS perlengkapan.v_izin_pemakaian_aktif;

ALTER TABLE perlengkapan.izin_pemakaian_bmn
    DROP CONSTRAINT IF EXISTS chk_housing_fields,
    DROP CONSTRAINT IF EXISTS chk_vehicle_fields,
    DROP CONSTRAINT IF EXISTS izin_pemakaian_bmn_jenis_bmn_check;

ALTER TABLE perlengkapan.izin_pemakaian_bmn
    DROP COLUMN IF EXISTS jenis_bmn,
    DROP COLUMN IF EXISTS bmn_nup,
    DROP COLUMN IF EXISTS bmn_kode_barang,
    DROP COLUMN IF EXISTS bmn_nama_barang,
    DROP COLUMN IF EXISTS bmn_merk,
    DROP COLUMN IF EXISTS bmn_tipe,
    DROP COLUMN IF EXISTS bmn_tahun_perolehan,
    DROP COLUMN IF EXISTS no_polisi,
    DROP COLUMN IF EXISTS no_bpkb,
    DROP COLUMN IF EXISTS no_rangka,
    DROP COLUMN IF EXISTS no_mesin,
    DROP COLUMN IF EXISTS no_stnk,
    DROP COLUMN IF EXISTS alamat,
    DROP COLUMN IF EXISTS luas_tanah,
    DROP COLUMN IF EXISTS luas_bangunan,
    DROP COLUMN IF EXISTS serial_number,
    DROP COLUMN IF EXISTS spesifikasi,
    DROP COLUMN IF EXISTS lokasi_pemakaian;

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
