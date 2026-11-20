use super::PakaianDinasRepository;
use crate::pakaian_dinas::models::*;
use crate::pakaian_dinas::validasi_ukuran::validasi_ukuran;
use crate::shared::error::{AppError, AppResult, bad_request};
use crate::shared::pegawai_ref::{PEGAWAI_KODE_SATKER_SQL, PEGAWAI_SATKER_JOIN_SQL};
use crate::shared::satker_scope::{BoxedParam, SatkerScope, as_refs};
use std::collections::HashMap;
use uuid::Uuid;

/// `scope` as an extra `AND` over the SoT row's satker column.
///
/// The upsert binds $1..$13 first, so this predicate must come out as $14 —
/// see the shared helper's note on why numbering is the part no type can
/// check. `the_upsert_takes_exactly_thirteen_value_binds` below pins it.
fn scope_and(scope: &SatkerScope, params: &mut Vec<BoxedParam>) -> String {
    // NOT `p.satker_id`: that column holds the upstream API's UUID, not the
    // `kode_satker` a scope carries, so the predicate matched zero rows for
    // every satker-scoped caller and the upsert silently affected nothing.
    // See [`crate::shared::pegawai_ref`] for the measurement.
    crate::shared::satker_scope::scope_and(scope, PEGAWAI_KODE_SATKER_SQL, params)
}

/// The 13 binds the upsert takes, in order. `kode_satker` is deliberately NOT
/// among them: the request used to carry it, which let a caller both label an
/// employee with any satker they liked and reach any employee at all. It now
/// comes from `integrasi.mysimkari_pegawai`, the SoT for employee identity.
fn profile_binds(
    request: &crate::pakaian_dinas::models::UpsertPegawaiProfileRequest,
) -> Vec<BoxedParam> {
    vec![
        Box::new(request.nip.clone()),
        Box::new(request.nama.clone()),
        Box::new(request.pangkat.clone()),
        Box::new(request.jabatan.clone()),
        Box::new(request.eselon.clone()),
        Box::new(request.jenis_kelamin.clone()),
        Box::new(request.jenis_pegawai.clone()),
        Box::new(request.with_hijab),
        Box::new(request.mapped_unit_kerja.clone()),
        Box::new(request.ukuran_baju.clone()),
        Box::new(request.ukuran_celana.clone()),
        Box::new(request.ukuran_sepatu.clone()),
        Box::new(chrono::Utc::now()),
    ]
}

/// One upsert, with the employee's satker taken from the SoT and the caller's
/// scope applied INSIDE the write.
///
/// `{scope_sql}` is spliced in as an extra `AND` on the SoT row, so an
/// out-of-scope caller selects nothing, inserts nothing, and never reaches the
/// `ON CONFLICT` path. A check in a preceding statement is a check the next
/// refactor can leave behind.
fn upsert_profile_sql(scope_sql: &str) -> String {
    format!(
        r#"
        INSERT INTO perlengkapan.pegawai_pakaian_dinas
            (nip, nama, pangkat, jabatan, eselon, jenis_kelamin,
             jenis_pegawai, with_hijab, mapped_unit_kerja, kode_satker,
             ukuran_baju, ukuran_celana, ukuran_sepatu, updated_at)
        -- Explicit casts are load-bearing: in `INSERT ... SELECT` Postgres
        -- does NOT infer a parameter's type from the target column the way
        -- it does for `VALUES`, so an uncast $n fails to prepare with
        -- "could not determine data type".
        SELECT $1::varchar,$2::varchar,$3::varchar,$4::varchar,$5::varchar,
               $6::varchar,$7::varchar,$8::boolean,$9::text,{kode_satker},
               $10::varchar,$11::varchar,$12::varchar,$13::timestamptz
        FROM integrasi.mysimkari_pegawai p
        {satker_join}
        WHERE p.nip = $1{scope_sql}
        ON CONFLICT (nip) DO UPDATE SET
            nama = COALESCE(EXCLUDED.nama, pegawai_pakaian_dinas.nama),
            pangkat = COALESCE(EXCLUDED.pangkat, pegawai_pakaian_dinas.pangkat),
            jabatan = COALESCE(EXCLUDED.jabatan, pegawai_pakaian_dinas.jabatan),
            eselon = COALESCE(EXCLUDED.eselon, pegawai_pakaian_dinas.eselon),
            jenis_kelamin = COALESCE(EXCLUDED.jenis_kelamin, pegawai_pakaian_dinas.jenis_kelamin),
            jenis_pegawai = COALESCE(EXCLUDED.jenis_pegawai, pegawai_pakaian_dinas.jenis_pegawai),
            with_hijab = EXCLUDED.with_hijab,
            mapped_unit_kerja = COALESCE(EXCLUDED.mapped_unit_kerja, pegawai_pakaian_dinas.mapped_unit_kerja),
            kode_satker = EXCLUDED.kode_satker,
            ukuran_baju = COALESCE(EXCLUDED.ukuran_baju, pegawai_pakaian_dinas.ukuran_baju),
            ukuran_celana = COALESCE(EXCLUDED.ukuran_celana, pegawai_pakaian_dinas.ukuran_celana),
            ukuran_sepatu = COALESCE(EXCLUDED.ukuran_sepatu, pegawai_pakaian_dinas.ukuran_sepatu),
            updated_at = EXCLUDED.updated_at
        "#,
        satker_join = PEGAWAI_SATKER_JOIN_SQL,
        kode_satker = PEGAWAI_KODE_SATKER_SQL,
    )
}

/// Nama kunci advisory untuk sebuah grup ukuran: satu ruang kunci bagi penulis ukuran dan
/// penghapusan ukuran.
const KUNCI_GRUP: &str = "ukuran_grup:";

/// Kunci BERSAMA per grup, tertahan sampai transaksi selesai. Dipanggil jalur yang membaca
/// daftar ukuran lalu menyimpan nilai terhadapnya; diambil berurut supaya tak ada siklus.
pub(super) async fn kunci_grup_bersama<C: deadpool_postgres::GenericClient>(
    client: &C,
    kode: &[String],
) -> AppResult<()> {
    let mut urut: Vec<&String> = kode.iter().collect();
    urut.sort();
    urut.dedup();
    for g in urut {
        client
            .execute(
                "SELECT pg_advisory_xact_lock_shared(hashtext($1::text)::bigint)",
                &[&format!("{KUNCI_GRUP}{g}")],
            )
            .await
            .map_err(|e| bad_request(&e.to_string()))?;
    }
    Ok(())
}

/// Aturan ukuran untuk sekumpulan kode grup: grupnya dan, untuk grup `PILIHAN`, daftar ukuran
/// yang sah. Satu kueri untuk grup dan satu untuk daftar, berapa pun banyaknya kode.
///
/// Dipakai SEMUA jalur yang menyimpan ukuran (profil pegawai, pengisian kampanye), supaya
/// aturannya tidak bisa menyimpang antar jalur.
pub(super) async fn aturan_grup<C: deadpool_postgres::GenericClient>(
    client: &C,
    kode: &[String],
) -> AppResult<HashMap<String, (UkuranGrup, Vec<String>)>> {
    if kode.is_empty() {
        return Ok(HashMap::new());
    }
    let grup = client
        .query(
            "SELECT kode, nama, tipe, urutan, is_active
             FROM perlengkapan.ms_ukuran_group WHERE kode = ANY($1)",
            &[&kode],
        )
        .await
        .map_err(|e| bad_request(&e.to_string()))?;
    let daftar = client
        .query(
            "SELECT \"group\", ukuran FROM perlengkapan.ms_ukuran
             WHERE \"group\" = ANY($1) ORDER BY urutan, ukuran",
            &[&kode],
        )
        .await
        .map_err(|e| bad_request(&e.to_string()))?;
    let mut per_grup: HashMap<String, Vec<String>> = HashMap::new();
    for r in &daftar {
        per_grup
            .entry(r.get("group"))
            .or_default()
            .push(r.get("ukuran"));
    }
    Ok(grup
        .iter()
        .map(|r| {
            let g = UkuranGrup::from_row(r);
            let l = per_grup.remove(&g.kode).unwrap_or_default();
            (g.kode.clone(), (g, l))
        })
        .collect())
}

impl PakaianDinasRepository {
    pub async fn get_pegawai_pakaian_dinas(
        &self,
        nip: &str,
    ) -> AppResult<Option<PegawaiPakaianDinas>> {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| bad_request(&e.to_string()))?;

        let row = client
            .query_opt(
                "SELECT * FROM perlengkapan.pegawai_pakaian_dinas WHERE nip = $1",
                &[&nip],
            )
            .await
            .map_err(|e| bad_request(&e.to_string()))?;

        Ok(row.map(|r| PegawaiPakaianDinas::from_row(&r)))
    }

    pub async fn upsert_pegawai_pakaian_dinas(
        &self,
        request: &UpdatePersonalUkuranRequest,
        nip: &str,
        nama: Option<&str>,
    ) -> AppResult<PegawaiPakaianDinas> {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| bad_request(&e.to_string()))?;
        let now = chrono::Utc::now();

        client
            .execute(
                r#"
                INSERT INTO perlengkapan.pegawai_pakaian_dinas
                    (nip, nama, ukuran_baju, ukuran_celana, ukuran_sepatu, with_hijab, updated_at)
                VALUES ($1, $2, $3, $4, $5, COALESCE($6, false), $7)
                ON CONFLICT (nip) DO UPDATE SET
                    ukuran_baju = EXCLUDED.ukuran_baju,
                    ukuran_celana = EXCLUDED.ukuran_celana,
                    ukuran_sepatu = EXCLUDED.ukuran_sepatu,
                    -- $6, not EXCLUDED: the INSERT list already collapsed a
                    -- missing flag to `false`, so EXCLUDED cannot tell "the
                    -- caller said false" from "the caller said nothing" and
                    -- would clear a flag this endpoint does not own.
                    with_hijab = COALESCE($6, pegawai_pakaian_dinas.with_hijab),
                    updated_at = EXCLUDED.updated_at
                "#,
                &[
                    &nip,
                    &nama,
                    &request.ukuran_baju,
                    &request.ukuran_celana,
                    &request.ukuran_sepatu,
                    &request.with_hijab,
                    &now,
                ],
            )
            .await
            .map_err(|e| bad_request(&e.to_string()))?;

        self.get_pegawai_pakaian_dinas(nip)
            .await?
            .ok_or_else(|| bad_request("Gagal menyimpan data ukuran"))
    }

    /// Full profile upsert — writes sizes + reporting fields in one shot.
    /// Used by the wizard and admin bulk-import.
    pub async fn upsert_pegawai_profile(
        &self,
        request: &crate::pakaian_dinas::models::UpsertPegawaiProfileRequest,
        scope: &SatkerScope,
    ) -> AppResult<PegawaiPakaianDinas> {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| bad_request(&e.to_string()))?;

        let mut params = profile_binds(request);
        let scope_sql = scope_and(scope, &mut params);
        let affected = client
            .execute(&upsert_profile_sql(&scope_sql), &as_refs(&params))
            .await
            .map_err(|e| bad_request(&e.to_string()))?;

        if affected == 0 {
            // Either the NIP is absent from the SoT or it belongs to a satker
            // outside the caller's scope. Both answer NotFound: telling the two
            // apart would let a caller enumerate other satkers' employees one
            // NIP at a time.
            return Err(crate::shared::error::AppError::NotFound(format!(
                "Pegawai tidak ditemukan: {}",
                request.nip
            )));
        }

        self.get_pegawai_pakaian_dinas(&request.nip)
            .await?
            .ok_or_else(|| bad_request("Gagal menyimpan profil pegawai"))
    }

    /// Bulk upsert profiles — used by the pakaian dinas wizard when
    /// submitting a full satker roster at once.
    /// Bulk upsert profiles — used by the pakaian dinas wizard when
    /// submitting a full satker roster at once.
    ///
    /// Same rule per row as the single upsert, and the same reason it is one
    /// statement per row rather than a preceding bulk permission check: the
    /// wizard posts a roster the caller chose, so a check that ran once over
    /// the batch would be a check the caller can widen after it passed.
    ///
    /// Returns how many rows the SoT + scope actually admitted. A row the
    /// caller may not touch is skipped, not an error for the whole batch —
    /// but the count comes back so the caller can see the batch was trimmed.
    pub async fn bulk_upsert_pegawai_profiles(
        &self,
        requests: &[crate::pakaian_dinas::models::UpsertPegawaiProfileRequest],
        scope: &SatkerScope,
    ) -> AppResult<usize> {
        if requests.is_empty() {
            return Ok(0);
        }
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| bad_request(&e.to_string()))?;

        let mut count = 0usize;
        for req in requests {
            let mut params = profile_binds(req);
            let scope_sql = scope_and(scope, &mut params);
            let affected = client
                .execute(&upsert_profile_sql(&scope_sql), &as_refs(&params))
                .await
                .map_err(|e| bad_request(&e.to_string()))?;
            count += affected as usize;
        }

        Ok(count)
    }

    // ============ MySIMKARI Integration ============

    /// This comment used to claim `integrasi.mysimkari_pegawai.satker_id`
    /// held the MySIMKARI `kode_satker`. It holds the upstream API's UUID
    /// (`mysimkari_satker.api_id`), so `WHERE satker_id = $1` with a code
    /// returned an EMPTY roster for every one of the 191 satkers that have
    /// people in them — see [`crate::shared::pegawai_ref`]. The satker is now
    /// resolved through a join instead of assumed from the column name.
    /// An employee roster is named people — NIP, name, phone, gender, rank —
    /// so it is satker data, not reference data. `scope` is in the signature
    /// rather than left to the three handlers that call this, because a rule
    /// the compiler enforces cannot be forgotten by the fourth caller.
    pub async fn get_mysimkari_pegawai_by_satker(
        &self,
        satker_code: &str,
        scope: &SatkerScope,
    ) -> AppResult<Vec<MysimkariPegawai>> {
        if !self.satker_code_in_scope(scope, satker_code).await? {
            // NotFound, not Forbidden: 403 would confirm the satker exists and
            // holds people, which is the cross-tenant existence oracle #93
            // closed for satker detail.
            return Err(crate::shared::error::AppError::NotFound(format!(
                "Satker tidak ditemukan: {satker_code}"
            )));
        }

        let client = self
            .pool
            .get()
            .await
            .map_err(|e| bad_request(&e.to_string()))?;

        let rows = client
            .query(
                &format!(
                    r#"
                    SELECT p.*
                    FROM integrasi.mysimkari_pegawai p
                    {join}
                    WHERE {kode_satker} = $1
                    ORDER BY p.nama ASC
                    "#,
                    join = PEGAWAI_SATKER_JOIN_SQL,
                    kode_satker = PEGAWAI_KODE_SATKER_SQL,
                ),
                &[&satker_code],
            )
            .await
            .map_err(|e| bad_request(&e.to_string()))?;

        Ok(rows.iter().map(MysimkariPegawai::from_row).collect())
    }

    /// Is `code` visible to a caller holding `scope`?
    ///
    /// Third of its name — see `SatkerScope::WILAYAH_MEMBERSHIP_SQL`, which is
    /// where the query itself lives so that these three cannot drift apart.
    /// The pure tiers (All/Denied/Satker) are decided without touching the DB,
    /// by `SatkerScope` itself; only Wilayah needs `integrasi.v_satker_wilayah`.
    pub async fn satker_code_in_scope(&self, scope: &SatkerScope, code: &str) -> AppResult<bool> {
        let SatkerScope::Wilayah(caller) = scope else {
            // `unwrap_or(false)` = fail closed; unreachable in practice because
            // Wilayah is the only variant that answers None.
            return Ok(scope.contains_code_local(code).unwrap_or(false));
        };

        let client = self
            .pool
            .get()
            .await
            .map_err(|e| bad_request(&e.to_string()))?;
        let row = client
            .query_one(SatkerScope::WILAYAH_MEMBERSHIP_SQL, &[&code, &caller])
            .await
            .map_err(|e| bad_request(&e.to_string()))?;
        Ok(row.get::<_, bool>("in_scope"))
    }

    // ============ Reports ============
}

// ============ Profil ukuran per spesifikasi (V019) ============
impl PakaianDinasRepository {
    /// Semua kolom pakaian yang aktif beserta ukuran yang tercatat untuk `nip`, kosong bila
    /// belum pernah diisi. Spesifikasi yang grupnya dinonaktifkan tidak ditawarkan lagi.
    pub async fn get_ukuran_profil(&self, nip: &str) -> AppResult<Vec<UkuranProfilItem>> {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| bad_request(&e.to_string()))?;
        let rows = client
            .query(
                "SELECT s.id AS spesifikasi_id, j.nama AS jenis_pakaian_nama, s.nama AS spesifikasi_nama,
                        s.ukuran_group, g.tipe, s.gender, u.ukuran
                 FROM perlengkapan.ms_spesifikasi_pakaian_dinas s
                 JOIN perlengkapan.ms_jenis_pakaian_dinas j ON j.id = s.jenis_pakaian_dinas_id
                 JOIN perlengkapan.ms_ukuran_group g ON g.kode = s.ukuran_group
                 LEFT JOIN perlengkapan.pegawai_pakaian_dinas_ukuran u
                        ON u.spesifikasi_id = s.id AND u.nip = $1
                 WHERE COALESCE(s.is_active, true) AND g.is_active AND COALESCE(j.is_active, true)
                 ORDER BY j.nama, g.urutan, s.nama",
                &[&nip],
            )
            .await
            .map_err(|e| bad_request(&e.to_string()))?;
        Ok(rows.iter().map(UkuranProfilItem::from_row).collect())
    }

    /// Simpan ukuran profil per spesifikasi. Setiap nilai diperiksa terhadap aturan grup
    /// spesifikasinya (pilihan dari daftar, atau teks bebas); nilai kosong menghapus isian.
    /// Satu transaksi: atau semuanya tersimpan, atau tak satu pun.
    pub async fn simpan_ukuran_profil(
        &self,
        nip: &str,
        nama: Option<&str>,
        items: &[UkuranProfilInput],
    ) -> AppResult<Vec<UkuranProfilItem>> {
        let mut client = self
            .pool
            .get()
            .await
            .map_err(|e| bad_request(&e.to_string()))?;
        let tx = client
            .transaction()
            .await
            .map_err(|e| bad_request(&e.to_string()))?;

        let spek_ids: Vec<Uuid> = items.iter().map(|i| i.spesifikasi_id).collect();
        let spek = tx
            .query(
                "SELECT s.id, s.nama, s.ukuran_group
                 FROM perlengkapan.ms_spesifikasi_pakaian_dinas s WHERE s.id = ANY($1)",
                &[&spek_ids],
            )
            .await
            .map_err(|e| bad_request(&e.to_string()))?;
        let per_spek: HashMap<Uuid, (String, String)> = spek
            .iter()
            .map(|r| (r.get("id"), (r.get("nama"), r.get("ukuran_group"))))
            .collect();
        let kode: Vec<String> = per_spek.values().map(|(_, g)| g.clone()).collect();
        // Kunci bersama per grup SEBELUM aturannya dibaca: penghapusan ukuran mengambil kunci
        // eksklusif yang sama, jadi sebuah ukuran tak bisa lenyap dari daftar di antara validasi
        // dan penyimpanan. Penulis tak saling menunggu (kunci bersama).
        kunci_grup_bersama(&tx, &kode).await?;
        let aturan = aturan_grup(&tx, &kode).await?;

        // Baris profil bagi pemiliknya sendiri (FK dari tabel per spesifikasi). Tak membuat
        // apa pun untuk NIP orang lain: NIP berasal dari klaim pemanggil, bukan dari isi permintaan.
        tx.execute(
            "INSERT INTO perlengkapan.pegawai_pakaian_dinas (nip, nama, updated_at)
             VALUES ($1, $2, now()) ON CONFLICT (nip) DO NOTHING",
            &[&nip, &nama],
        )
        .await
        .map_err(|e| bad_request(&e.to_string()))?;

        for item in items {
            let (nama_spek, grup_kode) = per_spek.get(&item.spesifikasi_id).ok_or_else(|| {
                AppError::NotFound(format!(
                    "Spesifikasi {} tidak ditemukan",
                    item.spesifikasi_id
                ))
            })?;
            if item.ukuran.trim().is_empty() {
                tx.execute(
                    "DELETE FROM perlengkapan.pegawai_pakaian_dinas_ukuran
                     WHERE nip = $1 AND spesifikasi_id = $2",
                    &[&nip, &item.spesifikasi_id],
                )
                .await
                .map_err(|e| bad_request(&e.to_string()))?;
                continue;
            }
            let (grup, daftar) = aturan
                .get(grup_kode)
                .ok_or_else(|| bad_request(&format!("Grup ukuran \"{grup_kode}\" tidak ada")))?;
            let nilai = validasi_ukuran(grup, daftar, &item.ukuran)
                .map_err(|e| bad_request(&format!("{nama_spek}: {e}")))?;
            tx.execute(
                "INSERT INTO perlengkapan.pegawai_pakaian_dinas_ukuran (nip, spesifikasi_id, ukuran, sumber)
                 VALUES ($1, $2, $3, 'APLIKASI')
                 ON CONFLICT (nip, spesifikasi_id) DO UPDATE
                    SET ukuran = EXCLUDED.ukuran, sumber = EXCLUDED.sumber, updated_at = now()",
                &[&nip, &item.spesifikasi_id, &nilai],
            )
            .await
            .map_err(|e| bad_request(&e.to_string()))?;
        }
        tx.commit().await.map_err(|e| bad_request(&e.to_string()))?;
        self.get_ukuran_profil(nip).await
    }
}

#[cfg(test)]
mod scope_sql_tests {
    use super::*;
    use crate::pakaian_dinas::models::UpsertPegawaiProfileRequest;

    fn req() -> UpsertPegawaiProfileRequest {
        UpsertPegawaiProfileRequest {
            nip: "199203142014031001".to_string(),
            nama: None,
            pangkat: None,
            jabatan: None,
            eselon: None,
            jenis_kelamin: None,
            jenis_pegawai: None,
            with_hijab: false,
            mapped_unit_kerja: None,
            kode_satker: Some("02.28".to_string()),
            ukuran_baju: None,
            ukuran_celana: None,
            ukuran_sepatu: None,
        }
    }

    /// The count the `$14` below depends on. If a column is ever added to the
    /// upsert without extending this, the scope predicate silently lands on the
    /// wrong placeholder — so pin it here rather than trusting the reader.
    #[test]
    fn the_upsert_takes_exactly_thirteen_value_binds() {
        assert_eq!(profile_binds(&req()).len(), 13);
    }

    /// Two failure modes no type can catch. Bound at `$1` the predicate would
    /// compare the satker column against the NIP; keyed on `p.satker_id` it
    /// would compare a `kode_satker` against a UUID. Both match nothing, and
    /// both look like a working deny to anyone who only tested the negative
    /// case — which is how the second one survived until #882.
    #[test]
    fn scope_predicate_binds_after_the_value_binds() {
        let mut params = profile_binds(&req());
        let clause = scope_and(&SatkerScope::Satker("02.28".to_string()), &mut params);
        assert_eq!(clause, format!(" AND {PEGAWAI_KODE_SATKER_SQL} = $14"));
        assert!(
            !clause.contains("p.satker_id"),
            "the scope must key on the resolved code, not the upstream UUID"
        );
        assert_eq!(params.len(), 14);
    }

    #[test]
    fn unrestricted_scope_adds_no_clause_and_no_bind() {
        let mut params = profile_binds(&req());
        assert_eq!(scope_and(&SatkerScope::All, &mut params), "");
        assert_eq!(params.len(), 13);
    }

    #[test]
    fn denied_scope_matches_no_row() {
        let mut params = profile_binds(&req());
        assert_eq!(scope_and(&SatkerScope::Denied, &mut params), " AND FALSE");
        assert_eq!(params.len(), 13);
    }

    #[test]
    fn wilayah_scope_resolves_through_the_shared_view() {
        let mut params = profile_binds(&req());
        let clause = scope_and(&SatkerScope::Wilayah("02.28".to_string()), &mut params);
        assert!(clause.contains("integrasi.v_satker_wilayah"));
        assert!(clause.contains("$14"));
        // The column that groups 15 Kejati under one value must not come back.
        assert!(!clause.contains("mysimkari_satker"));
        assert_eq!(params.len(), 14);
    }

    /// The request still *has* a `kode_satker` field (the frontend sends it),
    /// but the write must ignore it: trusting it is what let one satker label
    /// and rewrite another satker's employee.
    #[test]
    fn the_satker_written_comes_from_the_sot_not_the_request() {
        let sql = upsert_profile_sql(&real_scope_sql());
        assert!(
            sql.contains("FROM integrasi.mysimkari_pegawai p"),
            "the employee's satker must be read from the SoT"
        );
        // The stored value is the RESOLVED code, reached through the join —
        // storing `p.satker_id` here is what put UUIDs in a `kode_satker`
        // column and made every satker-scoped read of the roster empty.
        assert!(sql.contains(PEGAWAI_SATKER_JOIN_SQL));
        assert!(sql.contains(&format!("$8::boolean,$9::text,{PEGAWAI_KODE_SATKER_SQL},")));
        assert!(!sql.contains("$9::text,p.satker_id,"));
        // On conflict it is overwritten, not COALESCEd: a COALESCE would let a
        // stale row keep a satker the SoT no longer agrees with.
        assert!(sql.contains("kode_satker = EXCLUDED.kode_satker"));
        assert!(!sql.contains("kode_satker = COALESCE("));
    }

    /// The scope rides inside the statement that writes. Placed in a preceding
    /// SELECT it would be a check the next refactor can drop.
    #[test]
    fn the_scope_is_part_of_the_write() {
        let scope_sql = real_scope_sql();
        let sql = upsert_profile_sql(&scope_sql);
        let where_at = sql.find("WHERE p.nip = $1").expect("row selector");
        let conflict_at = sql.find("ON CONFLICT (nip)").expect("upsert");
        assert!(where_at < conflict_at);
        assert!(sql[where_at..conflict_at].contains(scope_sql.trim()));
    }

    /// The scope clause these assertions splice in is the one `scope_and`
    /// actually produces — a hand-written literal would keep passing after the
    /// column underneath it changed, which is exactly what happened here.
    fn real_scope_sql() -> String {
        let mut params = profile_binds(&req());
        scope_and(&SatkerScope::Satker("02.28".to_string()), &mut params)
    }
}
