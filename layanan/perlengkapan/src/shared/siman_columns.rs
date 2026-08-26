//! Canonical SQL for reading `integrasi.siman_aset`.
//!
//! # Why this module exists
//!
//! `integrasi.siman_aset` has two parallel sets of columns for the same facts,
//! and only one of them is ever written. The SIMAN ingest builds its `INSERT`
//! column list **directly from the keys of the API payload**
//! (`layanan/integrasi/src/db.rs` — `first_obj.keys()`, lowercased), so every
//! schema column is silently optional: it stays empty forever unless SIMAN
//! happens to send a field of that exact name. There is no `NOT NULL`, no
//! error, and no log line.
//!
//! Census of the 624 533-row staging snapshot:
//!
//! | populated (624 533 rows) | populated in 5 rows (our own e2e seed) | populated in 0 rows |
//! |---|---|---|
//! | `no_aset`, `kd_brg`, `nama`, `ur_kondisi`, `ur_sskel`, `rph_aset`, `jenis_aset` | `kategori_aset`, `nup` | `kode_barang`, `nama_barang`, `kondisi`, `satker_id` |
//!
//! The `5 / 624533` ratio is the signature of "only our own seed" — which is
//! why a `LIMIT 10` spot-check is worthless here and every claim below was
//! measured with
//! `count(*) FILTER (WHERE c IS NOT NULL AND c <> ''), count(*)`.
//!
//! The deepest cause is that the normalisation layer meant to populate the
//! canonical half was written and never wired in: `SimanTransformer`
//! (`layanan/integrasi/src/siman/transform.rs`) maps `NUP`/`KODE_BARANG`/
//! `NAMA_BARANG`/`KONDISI` onto `nup`/`kode_barang`/`nama_barang`/`kondisi`
//! and has **zero callers**. Wiring it in means re-ingesting 624 533 rows, so
//! it is tracked separately; the read side is correct either way.
//!
//! # Why the SQL lives in constants rather than being spelled out per query
//!
//! Four separate sites had already been fixed one at a time — the dashboard
//! breakdown (#829), the bank-aset list and lookup (#832), and the integrasi
//! gRPC projection — each time by someone rediscovering the census. A fifth
//! (this crate's BMN utilisation report) was still reading the dead half.
//! Constants make the mapping one edit instead of five, and make the next
//! consumer inherit the fix rather than re-derive it.
//!
//! A projection and the predicate that filters it MUST use the same expression:
//! if a dropdown offers a value the `WHERE` clause cannot match, filtering
//! silently returns nothing and looks like "no data" rather than a bug.

/// An asset's type ("kategori"/"jenis") — SIMAN's `jenis_aset`.
///
/// Populated for all 624 533 rows and carrying the real BMN taxonomy (Tanah,
/// Gedung dan Bangunan, Alat Angkutan Bermotor, Peralatan Mesin Khusus TIK,
/// Peralatan Mesin Non TIK, Alat Persenjataan, Alat Besar, Aset Tetap Lainnya,
/// Aset Tak Berwujud, Konstruksi Dalam Pengerjaan, Rumah Negara, Instalasi dan
/// Jaringan, Jalan dan Jembatan, Bangunan Air, Aset Tetap Renovasi). It is the
/// discriminator the owning service itself filters on ("unified siman_aset
/// table filtered by jenis_aset", `layanan/integrasi/src/grpc/service.rs`).
///
/// Its dead counterpart `kategori_aset` is NULL in 624 528 of 624 533 rows;
/// reading it panicked the dashboard on the NULL group, and with
/// `panic = "abort"` that killed the process.
pub const ASSET_KATEGORI_SQL: &str = "COALESCE(NULLIF(jenis_aset, ''), 'TIDAK DIKETAHUI')";

/// An asset's NUP (Nomor Urut Perolehan) — SIMAN's `no_aset`.
///
/// SIMAN sends `no_aset`; it does not send `nup`. `no_aset` is numeric in
/// 624 528 of 624 533 rows, ranges 0–21 727, and runs consecutively within one
/// `kd_brg`, which is exactly "sequence number of this item within its barang
/// code".
///
/// `NULLIF` yields NULL rather than `''`, which the FE renders as "-" — the
/// correct display for an asset SIMAN gave no NUP, instead of a misleading "0".
///
/// # NUP alone is not an identity; the identity is a TRIPLE
///
/// A BMN asset is identified by **kode satker + kode barang + NUP**
/// (`kdsatker_keu`, `kd_brg`, `no_aset`). NUP is unique only WITHIN one barang
/// code at one satker — "Kejari Mamuju / kendaraan unit tahanan / NUP 2" names
/// exactly one physical asset.
///
/// So NUP on its own identifies nothing: the 548 042 assets in "Baik" condition
/// carry only **14 141 distinct `no_aset`** values between them. Anything keyed
/// on NUP alone — `COUNT(DISTINCT nup)` as an asset population, or a join
/// `WHERE bmn_nup = …` — is wrong by roughly a factor of 39, and matches assets
/// belonging to other satkers.
///
/// The triple IS meant to be unique, and very nearly is: 620 676 distinct out
/// of 624 533 rows. The 3 503 groups that repeat (7 360 rows, 0.62%) are a
/// **source-data fault, not a modelling nuance** — inspection shows genuinely
/// different assets sharing one NUP, e.g. `006010199005016000KP / 3050201004 /
/// 677` holds one item acquired 2015-05-06 for Rp 1 650 000 and another
/// acquired 2020-12-30 for Rp 9 801 000. None of the 3 503 groups is a
/// re-ingest of an identical row (every one differs in name, value, date or
/// condition), so deduplicating them would destroy data. They are worth raising
/// with the SIMAN owner; they are NOT a licence to treat the triple as
/// non-identifying.
pub const ASSET_NUP_SQL: &str = "NULLIF(no_aset, '')";

/// An asset's barang code — SIMAN's `kd_brg`, with the canonical `kode_barang`
/// preferred if a future ingest ever populates it.
///
/// Format matters as much as which column is read: `kd_brg` is **ten digits
/// with no dots** (`3050201002`) in 624 528 of 624 533 rows. Any predicate
/// written against the dotted presentation form — `LIKE '03.01%'` — matches
/// nothing, which is a silent empty result rather than an error.
pub const ASSET_KODE_BARANG_SQL: &str = "COALESCE(NULLIF(kode_barang, ''), NULLIF(kd_brg, ''), '')";

/// An asset's name — SIMAN's `nama`. `nama_barang` is populated in 0 rows.
pub const ASSET_NAMA_SQL: &str = "COALESCE(NULLIF(nama_barang, ''), NULLIF(nama, ''), '')";

/// An asset's condition — SIMAN's `ur_kondisi`. `kondisi` is populated in 0 rows.
///
/// Values are **Title Case**, not upper: `Baik` 548 038, `Rusak Berat` 64 522,
/// `Rusak Ringan` 11 936. A `WHERE kondisi = 'BAIK'` therefore fails twice over
/// — wrong column *and* wrong case — and returns 0 rows without erroring.
pub const ASSET_KONDISI_SQL: &str = "COALESCE(NULLIF(kondisi, ''), NULLIF(ur_kondisi, ''), '')";

/// Predicate for "this asset is in serviceable condition".
///
/// Case-insensitive deliberately: the populated column is Title Case today, but
/// the dead canonical column was written against `'BAIK'`, and a report whose
/// denominator silently collapses to zero is worse than a slightly slower scan.
pub const ASSET_KONDISI_BAIK_PREDICATE: &str =
    "upper(COALESCE(NULLIF(kondisi, ''), NULLIF(ur_kondisi, ''), '')) = 'BAIK'";

/// Map a SIMAN `jenis_aset` onto the four-value `jenis_bmn` domain that
/// `perlengkapan.izin_pemakaian_bmn` is constrained to
/// (`KENDARAAN_BERMOTOR`, `RUMAH_NEGARA`, `LAPTOP`, `LAINNYA` — see the CHECK
/// in `V001__baseline.sql`). The two sides must agree or the report's
/// `LEFT JOIN … ON bc.jenis_bmn = uc.jenis_bmn` matches nothing.
///
/// This replaces a `kode_barang LIKE '03.01%'` ladder that could not match a
/// single row twice over: `kode_barang` is populated in 0 of 624 533 rows, and
/// the column that is populated (`kd_brg`) holds ten undotted digits, so the
/// dotted prefix never matched even after the column was corrected.
///
/// Counts over the 548 042 assets in serviceable condition on staging:
/// `LAINNYA` 469 710, `LAPTOP` 66 732, `KENDARAAN_BERMOTOR` 8 344,
/// `RUMAH_NEGARA` 3 256.
///
/// Caveat kept explicit rather than smoothed over: `LAPTOP` is the permit
/// domain's only IT bucket, so all "Peralatan Mesin Khusus TIK" lands there —
/// it is broader than the label. Folding it into `LAINNYA` instead would hide
/// the second-largest category entirely, so the label is wrong in a way that is
/// visible rather than wrong in a way that is not. Widening the domain needs a
/// migration plus FE work and is out of scope here.
///
/// `alias` is the table alias used in the query (`""` when the table is
/// unaliased, `"s."` for `FROM integrasi.siman_aset s`).
pub fn jenis_bmn_sql(alias: &str) -> String {
    format!(
        "CASE \
           WHEN {a}jenis_aset = 'Alat Angkutan Bermotor' THEN 'KENDARAAN_BERMOTOR' \
           WHEN {a}jenis_aset = 'Rumah Negara'           THEN 'RUMAH_NEGARA' \
           WHEN {a}jenis_aset = 'Peralatan Mesin Khusus TIK' THEN 'LAPTOP' \
           ELSE 'LAINNYA' \
         END",
        a = alias
    )
}
