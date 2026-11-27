use super::*;
use crate::workflow::giliran::{peran_giliran, peran_terikat_satker};
use std::collections::HashSet;
use uuid::Uuid;

/// What [`WorkflowEngine::beri_tahu_giliran`] needs to know about a move.
pub(crate) struct Giliran<'a> {
    pub entity_type: &'a str,
    /// The id the notification links to (for Kebutuhan BMN, the campaign's).
    pub entity_id: Uuid,
    pub to_state: &'a str,
    /// The satker(s) the entity belongs to: who a satker-bound role is resolved through.
    pub satker_codes: &'a [String],
    /// Names the satker when one record spans several.
    pub satker_nama: Option<&'a str>,
    /// The mover's note, quoted in the message.
    pub catatan: Option<&'a str>,
    /// Whoever asked for the entity: never asked to validate or approve it (maker-checker).
    pub pemohon: Option<Uuid>,
    /// Whoever just moved it: never told about their own move.
    pub aktor: Uuid,
}

/// Who asked for an entity, and which satker(s) it belongs to.
struct Identitas {
    /// `created_by`; nullable in the schema.
    pemohon: Option<Uuid>,
    /// MySIMKARI `kode_satker`. Empty when the row predates the column (V003), which no
    /// satker-bound role can then be resolved from. A campaign holds one per participating satker.
    satker_codes: Vec<String>,
}

impl WorkflowEngine {
    /// Send workflow notifications after a state transition.
    ///
    /// Two kinds, and they are not the same message:
    ///
    /// 1. to **the person who asked**: what became of their request (approved, rejected, sent back,
    ///    or simply moved on);
    /// 2. to **the role whose move comes next**: that work is waiting for them
    ///    ([`Self::beri_tahu_giliran`]).
    ///
    /// The second used to be a placeholder that returned nobody, matched on state names that no
    /// workflow has (`SUBMITTED`, `REVISION_REQUIRED`), and so told no approver about anything: work
    /// sat in a list until somebody happened to open it.
    ///
    /// Requirements: REQ-N001, REQ-N003, REQ-N005, REQ-W011
    pub(crate) async fn send_workflow_notifications(
        &self,
        request: &TransitionRequest,
        entity_type: &str,
        document_url: Option<&str>,
    ) -> Result<usize> {
        let notifier = self.notifier.as_ref().ok_or_else(|| {
            WorkflowError::InvalidState("Notification sender not configured".to_string())
        })?;

        let identitas = self
            .identitas_entitas(request.entity_id, entity_type)
            .await?;

        // 1. The person who asked.
        let (jenis, prioritas) = Self::pesan_pemohon(request, entity_type, document_url);
        let mut sent = 0usize;
        match identitas.pemohon {
            Some(pemohon) => {
                let msg = crate::workflow::to_notification_message(pemohon, &jenis, prioritas);
                match notifier.send(msg).await {
                    Ok(_) => sent += 1,
                    Err(e) => tracing::error!(
                        user_id = %pemohon,
                        error = %e,
                        "Failed to send notification to user"
                    ),
                }
            }
            None => tracing::warn!(
                entity_id = %request.entity_id,
                from_state = %request.from_state,
                to_state = %request.to_state,
                "No recipients found for notification"
            ),
        }

        // 2. The role whose turn it is.
        sent += self
            .beri_tahu_giliran(Giliran {
                entity_type,
                entity_id: request.entity_id,
                to_state: &request.to_state,
                satker_codes: &identitas.satker_codes,
                satker_nama: None,
                catatan: request.catatan.as_deref(),
                pemohon: identitas.pemohon,
                aktor: request.user_id,
            })
            .await?;
        Ok(sent)
    }

    /// Tell the role whose move comes next that `entity_id` is waiting for them in `to_state`.
    ///
    /// `satker_codes` are the satker(s) the entity belongs to; the people told are the holders of
    /// the role there (an operator or satker validator of that satker, the wilayah validators of the
    /// Kejati above it, every pusat validator). `aktor` is never told about their own move.
    ///
    /// A role that is bound to a satker while the entity has none on record is logged and skipped,
    /// not silently dropped: that is a row to repair, and the log is the only place it shows.
    ///
    /// Public to the crate because a module that moves a part of an entity itself (Kebutuhan BMN's
    /// per-satker rows do not go through [`Self::transition`]) must tell the next role the same way.
    pub(crate) async fn beri_tahu_giliran(&self, giliran: Giliran<'_>) -> Result<usize> {
        let Giliran {
            entity_type,
            entity_id,
            to_state,
            satker_codes,
            satker_nama,
            catatan,
            pemohon,
            aktor,
        } = giliran;
        let Some(notifier) = self.notifier.as_ref() else {
            return Ok(0);
        };
        let peran_berikut = peran_giliran(entity_type, to_state);
        if peran_berikut.is_empty() {
            return Ok(0);
        }

        let mut sudah: HashSet<Uuid> = HashSet::new();
        sudah.insert(aktor);
        let mut sent = 0usize;
        for peran in peran_berikut {
            if peran_terikat_satker(peran) && satker_codes.is_empty() {
                tracing::warn!(
                    entity_type,
                    %entity_id,
                    state = to_state,
                    role = peran,
                    "work is waiting for a satker-bound role but the entity has no satker on record; nobody can be told"
                );
                continue;
            }
            let penerima =
                crate::shared::pengguna::penerima_peran_untuk(&self.db_pool, peran, satker_codes)
                    .await
                    .map_err(|e| WorkflowError::NotificationError(e.to_string()))?;
            // Maker-checker: whoever asked for it may not validate or approve it, so being asked to
            // would be a task the API then refuses. (The operator role, which fixes a return, is
            // exactly who the requester is.)
            let penerima: Vec<Uuid> = penerima
                .into_iter()
                .filter(|u| *peran == "operator_satker" || Some(*u) != pemohon)
                .filter(|u| sudah.insert(*u))
                .collect();
            if penerima.is_empty() {
                tracing::warn!(
                    entity_type,
                    %entity_id,
                    state = to_state,
                    role = peran,
                    "work is waiting for a role but no known user holds it here; nobody was told"
                );
                continue;
            }

            let jenis = crate::workflow::WorkflowNotificationType::ApprovalRequired {
                entity_type: entity_type.to_string(),
                entity_id: entity_id.to_string(),
                current_state: to_state.to_string(),
                required_role: (*peran).to_string(),
                deadline: self.compute_sla_deadline(to_state),
                satker_nama: satker_nama.map(str::to_string),
                catatan: catatan.map(str::to_string),
            };
            for user in penerima {
                let msg = crate::workflow::to_notification_message(
                    user,
                    &jenis,
                    crate::workflow::NotificationPriority::High,
                );
                match notifier.send(msg).await {
                    Ok(_) => sent += 1,
                    Err(e) => tracing::error!(
                        user_id = %user,
                        error = %e,
                        "Failed to send notification to user"
                    ),
                }
            }
        }
        Ok(sent)
    }

    /// What the person who asked is told about this move.
    fn pesan_pemohon(
        request: &TransitionRequest,
        entity_type: &str,
        document_url: Option<&str>,
    ) -> (
        crate::workflow::WorkflowNotificationType,
        crate::workflow::NotificationPriority,
    ) {
        use crate::workflow::{NotificationPriority, WorkflowNotificationType};
        match request.to_state.as_str() {
            "APPROVED" => (
                WorkflowNotificationType::ApprovalCompleted {
                    entity_type: entity_type.to_string(),
                    entity_id: request.entity_id.to_string(),
                    approved_by: request.user_id.to_string(),
                    document_url: document_url.map(|s| s.to_string()),
                },
                NotificationPriority::Normal,
            ),
            "REJECTED" => (
                WorkflowNotificationType::Rejected {
                    entity_type: entity_type.to_string(),
                    entity_id: request.entity_id.to_string(),
                    rejected_by: request.user_id.to_string(),
                    reason: request.catatan.clone(),
                },
                NotificationPriority::High,
            ),
            // The states a workflow sends back for correction. (This used to compare against
            // `REVISION_REQUIRED`, a name no workflow has, so a return was announced as a
            // plain move.)
            s if s.starts_with("REVISI_") || s == "RETURNED_TO_OPERATOR" => (
                WorkflowNotificationType::RevisionRequired {
                    entity_type: entity_type.to_string(),
                    entity_id: request.entity_id.to_string(),
                    requested_by: request.user_id.to_string(),
                    notes: request.catatan.clone(),
                },
                NotificationPriority::High,
            ),
            _ => (
                WorkflowNotificationType::WorkflowTransition {
                    entity_type: entity_type.to_string(),
                    entity_id: request.entity_id.to_string(),
                    from_state: request.from_state.clone(),
                    to_state: request.to_state.clone(),
                    transition_by: request.user_id.to_string(),
                    catatan: request.catatan.clone(),
                },
                NotificationPriority::Normal,
            ),
        }
    }

    /// Who asked for the entity and which satker(s) it belongs to.
    async fn identitas_entitas(&self, entity_id: Uuid, entity_type: &str) -> Result<Identitas> {
        let client = self.db_pool.get().await?;
        match entity_type {
            "kebutuhan_bmn" => {
                // `perlengkapan.kebutuhan_bmn` does not exist and never did: the real entity is the
                // pusat-driven campaign `pengajuan_kebutuhan_bmn` (the same table `status_table()`
                // transitions). `created_by` is nullable, hence `Option`; reading it as a bare
                // `Uuid` would panic.
                //
                // A campaign is national by construction and fans out to N rows in
                // `pengajuan_kebutuhan_bmn_satker`, so it has no single satker: it has the ones
                // taking part, and those are who a role bound to a satker is resolved through.
                let row = client
                    .query_one(
                        "SELECT created_by FROM perlengkapan.pengajuan_kebutuhan_bmn WHERE id = $1",
                        &[&entity_id],
                    )
                    .await?;
                let satker_codes = client
                    .query(
                        "SELECT DISTINCT satker_id FROM perlengkapan.pengajuan_kebutuhan_bmn_satker \
                         WHERE pengajuan_id = $1",
                        &[&entity_id],
                    )
                    .await?
                    .iter()
                    .map(|r| r.get::<_, String>("satker_id"))
                    .collect();
                Ok(Identitas {
                    pemohon: row.get("created_by"),
                    satker_codes,
                })
            }
            "penghapusan_bmn" => {
                // `penghapusan_bmn` has no `satker_id`: that column was dropped when `satker_code`
                // (text, the kode satker) became the satker, and
                // `the_two_unresolvable_uuid_columns_are_gone` pins that it stays gone.
                let row = client
                    .query_one(
                        "SELECT created_by, satker_code FROM perlengkapan.penghapusan_bmn WHERE id = $1",
                        &[&entity_id],
                    )
                    .await?;
                Ok(Identitas {
                    pemohon: row.get("created_by"),
                    satker_codes: row
                        .get::<_, Option<String>>("satker_code")
                        .into_iter()
                        .collect(),
                })
            }
            "pemakaian_bmn" => {
                // The requester is `created_by` (nullable); the unit is `satker_code`, derived from
                // the caller's claims at create (V003) and NOT the client-supplied
                // `pegawai_satker_id` uuid.
                let row = client
                    .query_one(
                        "SELECT created_by, satker_code FROM perlengkapan.izin_pemakaian_bmn WHERE id = $1",
                        &[&entity_id],
                    )
                    .await?;
                let pemohon: Option<Uuid> = row.get("created_by");
                if pemohon.is_none() {
                    tracing::warn!(
                        entity_id = %entity_id,
                        "pemakaian_bmn permit has no created_by; requester will not be notified"
                    );
                }
                Ok(Identitas {
                    pemohon,
                    satker_codes: row
                        .get::<_, Option<String>>("satker_code")
                        .into_iter()
                        .collect(),
                })
            }
            _ => Err(WorkflowError::InvalidState(format!(
                "Notification not supported for entity type: {}",
                entity_type
            ))),
        }
    }
}
