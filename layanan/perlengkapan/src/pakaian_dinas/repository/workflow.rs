use super::PakaianDinasRepository;
use crate::pakaian_dinas::models::*;
use crate::shared::error::{AppResult, bad_request};
use uuid::Uuid;

impl PakaianDinasRepository {
    /// Update pengajuan status and log activity
    pub async fn update_pengajuan_status(
        &self,
        pengajuan_id: Uuid,
        new_aktivitas_id: i32,
        user_id: Uuid,
        catatan: Option<String>,
    ) -> AppResult<()> {
        let mut client = self
            .pool
            .get()
            .await
            .map_err(|e| bad_request(&e.to_string()))?;

        let tx = client
            .transaction()
            .await
            .map_err(|e| bad_request(&e.to_string()))?;

        // Update pengajuan status
        let update_query = r#"
            UPDATE perlengkapan.pengajuan_pakaian_dinas
            SET aktivitas_id = $1, updated_at = NOW()
            WHERE id = $2
        "#;

        tx.execute(update_query, &[&new_aktivitas_id, &pengajuan_id])
            .await
            .map_err(|e| bad_request(&e.to_string()))?;

        // Log activity
        let activity_id = Uuid::new_v4();
        let insert_activity_query = r#"
            INSERT INTO perlengkapan.pengajuan_pakaian_dinas_aktivitas
            (id, pengajuan_id, aktivitas_id, user_id, catatan, created_at)
            VALUES ($1, $2, $3, $4, $5, NOW())
        "#;

        tx.execute(
            insert_activity_query,
            &[
                &activity_id,
                &pengajuan_id,
                &new_aktivitas_id,
                &user_id,
                &catatan,
            ],
        )
        .await
        .map_err(|e| bad_request(&e.to_string()))?;

        tx.commit().await.map_err(|e| bad_request(&e.to_string()))?;

        Ok(())
    }

    /// Persist a per-satker workflow transition and record an activity row,
    /// atomically (#40). Replaces the previous no-op validator action: the new
    /// `aktivitas_id` lands on `pengajuan_pakaian_dinas_satker` and an audit-
    /// friendly row is appended to `pengajuan_pakaian_dinas_satker_aktivitas`
    /// so the per-satker timeline has real history.
    pub async fn transition_satker_with_activity(
        &self,
        satker_id: Uuid,
        new_aktivitas_id: i32,
        komentar: Option<String>,
        nip: Option<&str>,
        nama: Option<&str>,
        jabatan: Option<&str>,
        role: Option<&str>,
    ) -> AppResult<()> {
        let mut client = self
            .pool
            .get()
            .await
            .map_err(|e| bad_request(&e.to_string()))?;

        let tx = client
            .transaction()
            .await
            .map_err(|e| bad_request(&e.to_string()))?;

        tx.execute(
            "UPDATE pengajuan_pakaian_dinas_satker \
             SET aktivitas_id = $1, updated_at = NOW() WHERE id = $2",
            &[&new_aktivitas_id, &satker_id],
        )
        .await
        .map_err(|e| bad_request(&e.to_string()))?;

        let activity_id = Uuid::new_v4();
        tx.execute(
            "INSERT INTO pengajuan_pakaian_dinas_satker_aktivitas \
             (id, pengajuan_satker_id, aktivitas_id, komentar, nip, nama, jabatan, role, created_at) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, NOW())",
            &[
                &activity_id,
                &satker_id,
                &new_aktivitas_id,
                &komentar,
                &nip,
                &nama,
                &jabatan,
                &role,
            ],
        )
        .await
        .map_err(|e| bad_request(&e.to_string()))?;

        tx.commit().await.map_err(|e| bad_request(&e.to_string()))?;
        Ok(())
    }

    /// Per-satker activity history, oldest first — backs the workflow timeline
    /// in the FE satker detail view (#40).
    pub async fn list_satker_aktivitas(
        &self,
        satker_id: Uuid,
    ) -> AppResult<Vec<PengajuanSatkerAktivitas>> {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| bad_request(&e.to_string()))?;

        let rows = client
            .query(
                "SELECT id, pengajuan_satker_id, aktivitas_id, komentar, nip, nama, \
                        pangkat, jabatan, role, created_at \
                 FROM pengajuan_pakaian_dinas_satker_aktivitas \
                 WHERE pengajuan_satker_id = $1 \
                 ORDER BY created_at ASC",
                &[&satker_id],
            )
            .await
            .map_err(|e| bad_request(&e.to_string()))?;

        Ok(rows
            .iter()
            .map(PengajuanSatkerAktivitas::from_row)
            .collect())
    }

    /// Update pengajuan document metadata
    pub async fn update_pengajuan_document(
        &self,
        pengajuan_id: Uuid,
        document_id: Uuid,
        document_url: String,
    ) -> AppResult<()> {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| bad_request(&e.to_string()))?;

        let update_query = r#"
            UPDATE perlengkapan.pengajuan_pakaian_dinas
            SET document_id = $1, document_url = $2, updated_at = NOW()
            WHERE id = $3
        "#;

        client
            .execute(update_query, &[&document_id, &document_url, &pengajuan_id])
            .await
            .map_err(|e| bad_request(&e.to_string()))?;

        Ok(())
    }
}
