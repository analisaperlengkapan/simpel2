use super::PakaianDinasRepository;
use crate::pakaian_dinas::models::*;
use crate::shared::error::{AppResult, bad_request};
use crate::shared::pegawai_ref::{PEGAWAI_KODE_SATKER_SQL, PEGAWAI_SATKER_JOIN_SQL};
use crate::shared::satker_scope::{BoxedParam, SatkerScope, as_refs};

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

    /// The failure mode no type can catch: bound at `$1` the predicate would
    /// compare `p.satker_id` against the NIP, match nothing, and look like a
    /// working deny to anyone who only tested the negative case.
    #[test]
    fn scope_predicate_binds_after_the_value_binds() {
        let mut params = profile_binds(&req());
        let clause = scope_and(&SatkerScope::Satker("02.28".to_string()), &mut params);
        assert_eq!(clause, " AND p.satker_id = $14");
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
        let sql = upsert_profile_sql(" AND p.satker_id = $14");
        assert!(
            sql.contains("FROM integrasi.mysimkari_pegawai p"),
            "the employee's satker must be read from the SoT"
        );
        assert!(sql.contains("$8::boolean,$9::text,p.satker_id,"));
        // On conflict it is overwritten, not COALESCEd: a COALESCE would let a
        // stale row keep a satker the SoT no longer agrees with.
        assert!(sql.contains("kode_satker = EXCLUDED.kode_satker"));
        assert!(!sql.contains("kode_satker = COALESCE("));
    }

    /// The scope rides inside the statement that writes. Placed in a preceding
    /// SELECT it would be a check the next refactor can drop.
    #[test]
    fn the_scope_is_part_of_the_write() {
        let sql = upsert_profile_sql(" AND p.satker_id = $14");
        let where_at = sql.find("WHERE p.nip = $1").expect("row selector");
        let conflict_at = sql.find("ON CONFLICT (nip)").expect("upsert");
        assert!(where_at < conflict_at);
        assert!(sql[where_at..conflict_at].contains("AND p.satker_id = $14"));
    }
}
