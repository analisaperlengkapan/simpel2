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
-- Re-applying this file is safe (every statement is guarded), which matters
-- if an operator deletes its history row to roll back onto a binary that
-- predates it. Rows the previous release writes are NOT repaired by re-running
-- this file — refinery would not run it again. Two things cover them instead:
-- triggers (section 6) normalise each legacy write AT INSERT, so a rolling
-- upgrade, where old pods still serve while new ones start, leaves no window;
-- and the startup normaliser (sql/normalize_pemakaian_items.sql, run by
-- `migrations::run` on every start) repairs what already existed before this
-- file ran — the first-apply backfill is that same file.
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
-- 2. (Backfill) lives in sql/normalize_pemakaian_items.sql, not here
-- ---------------------------------------------------------------------------
-- The parent's own asset becomes item slot 1 and the items that already exist
-- are numbered after it. That is not done in this file because refinery runs a
-- version once: the previous release may write more legacy items after a
-- rollback, and a one-shot backfill would never see them. `migrations::run`
-- executes the same statements on every startup, after refinery and before the
-- service serves, so the first apply and every later roll-forward take the
-- same path.


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
--
-- Who is recorded as revoking: the SYSTEM (`revoked_by_nama = 'system'`), since
-- the withdrawal is the automatic consequence of the renewal and not a
-- decision anyone took about the old permit. `revoked_by` (the user id) is left
-- NULL here because the trigger sees only the row; the service fills it with
-- the human who activated the renewal immediately afterwards
-- (`attribute_supersession`), as the earlier two-step version did. An
-- activation that bypasses the service leaves it NULL, which reads "unknown"
-- rather than a guess taken from `updated_by`. Either way `superseded_by_id`
-- leads to the renewal, whose approval and transition audit name the people.
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

-- ---------------------------------------------------------------------------
-- 6. The previous release keeps writing while this one starts (rolling upgrade)
-- ---------------------------------------------------------------------------
-- Old and new pods serve side by side during a rollout, and the old release
-- writes in ITS shapes: a permit with the first asset on the header, and extra
-- items that know nothing of `urutan`, `jenis_bmn` or the period. A repair that
-- runs once at startup (sql/normalize_pemakaian_items.sql) covers what exists
-- BEFORE a new pod serves, not what an old pod writes afterwards — and the new
-- release reads those fields as required, so one such row makes every request
-- for its permit fail until some pod happens to restart.
--
-- So the repair also happens AT INSERT, in the same transaction as the write,
-- which leaves no window. These two triggers fire only for the old release's
-- shapes and are inert for this release's own writes:
--
--   * a new-release item always supplies `urutan`      -> item trigger skipped;
--   * a new-release header always has NULL asset cols  -> header trigger skipped.
--
-- Everything the old release inserts into `pemakaian_bmn_items` is covered
-- because INSERT is the only statement it runs against that table (it never
-- updates or deletes an item).
--
-- Both go with the contract migration, together with the columns they exist for.

-- 6a. A permit written with its first asset on the header gets that asset as
--     item #1 straight away, so the new release sees it (list, SK, booking
--     check) without waiting for a restart.
CREATE OR REPLACE FUNCTION perlengkapan.legacy_header_to_item_one()
RETURNS trigger
LANGUAGE plpgsql
AS $f$
BEGIN
    INSERT INTO perlengkapan.pemakaian_bmn_items (
        izin_pemakaian_id, urutan, jenis_bmn,
        bmn_nup, bmn_kode_barang, bmn_nama_barang, bmn_merk, bmn_tipe,
        bmn_tahun_perolehan,
        no_polisi, no_bpkb, no_rangka, no_mesin, no_stnk,
        alamat, luas_tanah, luas_bangunan,
        serial_number, spesifikasi, lokasi_pemakaian,
        tanggal_mulai, tanggal_selesai, created_at, updated_at
    ) VALUES (
        NEW.id, 1, NEW.jenis_bmn,
        NEW.bmn_nup, NEW.bmn_kode_barang, NEW.bmn_nama_barang, NEW.bmn_merk, NEW.bmn_tipe,
        NEW.bmn_tahun_perolehan,
        NEW.no_polisi, NEW.no_bpkb, NEW.no_rangka, NEW.no_mesin, NEW.no_stnk,
        NEW.alamat, NEW.luas_tanah, NEW.luas_bangunan,
        NEW.serial_number, NEW.spesifikasi, NEW.lokasi_pemakaian,
        NEW.tanggal_mulai, NEW.tanggal_selesai, now(), now()
    );
    RETURN NULL;
END
$f$;

DROP TRIGGER IF EXISTS trg_izin_pemakaian_legacy_item_one ON perlengkapan.izin_pemakaian_bmn;
CREATE TRIGGER trg_izin_pemakaian_legacy_item_one
    AFTER INSERT ON perlengkapan.izin_pemakaian_bmn
    FOR EACH ROW
    WHEN (NEW.bmn_nup IS NOT NULL)
    EXECUTE FUNCTION perlengkapan.legacy_header_to_item_one();

-- 6b. An item written without a position gets the next one, a kind and a
--     period. The permit row is locked first, so concurrent legacy inserts for
--     one permit number themselves one after another instead of both taking
--     the same `max + 1` and one dying on the unique constraint.
--
--     `LAINNYA`, not the permit's kind, for the reason in the normaliser: the
--     old item table never recorded a kind, so copying the header's would
--     fabricate one.
CREATE OR REPLACE FUNCTION perlengkapan.legacy_item_defaults()
RETURNS trigger
LANGUAGE plpgsql
AS $f$
DECLARE
    v_izin perlengkapan.izin_pemakaian_bmn%ROWTYPE;
BEGIN
    SELECT * INTO v_izin
      FROM perlengkapan.izin_pemakaian_bmn
     WHERE id = NEW.izin_pemakaian_id
       FOR UPDATE;
    IF NOT FOUND THEN
        RETURN NEW;   -- the foreign key refuses it with a proper message
    END IF;

    NEW.urutan := (
        SELECT COALESCE(MAX(urutan), 0) + 1
          FROM perlengkapan.pemakaian_bmn_items
         WHERE izin_pemakaian_id = NEW.izin_pemakaian_id
    );
    NEW.jenis_bmn        := COALESCE(NEW.jenis_bmn, 'LAINNYA');
    NEW.tanggal_mulai    := COALESCE(NEW.tanggal_mulai, v_izin.tanggal_mulai);
    NEW.tanggal_selesai  := COALESCE(NEW.tanggal_selesai, v_izin.tanggal_selesai);
    NEW.lokasi_pemakaian := COALESCE(NEW.lokasi_pemakaian, v_izin.lokasi_pemakaian);
    RETURN NEW;
END
$f$;

DROP TRIGGER IF EXISTS trg_pemakaian_items_legacy_defaults ON perlengkapan.pemakaian_bmn_items;
CREATE TRIGGER trg_pemakaian_items_legacy_defaults
    BEFORE INSERT ON perlengkapan.pemakaian_bmn_items
    FOR EACH ROW
    WHEN (NEW.urutan IS NULL)
    EXECUTE FUNCTION perlengkapan.legacy_item_defaults();
