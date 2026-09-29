//! # Pemakaian BMN HTTP Handlers
//!
//! Request handlers for BMN usage permit REST API.
//! All endpoints require authentication via JWT middleware.

use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use tracing::info;
use uuid::Uuid;

use crate::shared::error::AppError;
use crate::shared::middleware::{Claims, ClientIp};
use crate::shared::satker_scope::SatkerScope;
use lib_perlengkapan::response::{ApiResponse, PaginatedResponse};

use super::models::*;
use super::services::PemakaianBmnService;

/// GET /pemakaian-bmn/:id
/// Get a single permit with full details
pub async fn get_permit_by_id(
    State(service): State<PemakaianBmnService>,
    Path(id): Path<Uuid>,
    claims: Claims,
) -> Result<Json<ApiResponse<IzinPemakaianDetailResponse>>, AppError> {
    use crate::shared::policy::{PemakaianBmnAction, PemakaianBmnPolicy, WorkflowPolicy};

    let scope = SatkerScope::from_claims(&claims);
    let mut response = service.get_permit_detail(id, &scope).await?;

    // The service answers "what may happen next" from the STATE. Narrow it to
    // "what may THIS caller do" by handing each candidate back to the same
    // policy the action handlers enforce with — so the API never advertises a
    // move that would come back 403, and the FE needs no RBAC table of its own.
    let from_state = response.izin.status.clone();
    response.allowed_transitions.retain(|t| {
        PemakaianBmnAction::for_transition(&from_state, &t.status).is_some_and(|action| {
            PemakaianBmnPolicy
                .authorize(&claims, action, Some(&from_state))
                .is_ok()
        })
    });

    Ok(Json(ApiResponse::success(
        response,
        "Permit retrieved successfully".to_string(),
    )))
}

/// POST /pemakaian-bmn
/// Create a new permit request
pub async fn create_permit(
    State(service): State<PemakaianBmnService>,
    claims: Claims,
    Json(request): Json<CreateIzinPemakaianRequest>,
) -> Result<(StatusCode, Json<ApiResponse<IzinPemakaianBmn>>), AppError> {
    use crate::shared::policy::{PemakaianBmnAction, PemakaianBmnPolicy, WorkflowPolicy};
    // Before this, ANY authenticated caller — a validator_pusat, an approver of
    // another satker — could open a permit request. Only an operator drafts, and
    // only for the satker their token names (never one taken from the body).
    PemakaianBmnPolicy.authorize(&claims, PemakaianBmnAction::Create, None)?;
    // (A caller whose token names no satker is refused by the service with a 400
    // — the permit must belong to exactly one satker.)
    info!("Creating new permit for BMN {}", request.bmn_nup);

    let permit = service
        .create_permit(
            request,
            claims.user_id,
            claims.username.clone(),
            claims.satker_code.clone(),
        )
        .await?;

    Ok((
        StatusCode::CREATED,
        Json(ApiResponse::success(
            permit,
            "Permit created successfully".to_string(),
        )),
    ))
}

/// PUT /pemakaian-bmn/:id
/// Update a permit (only in DRAFT status)
pub async fn update_permit(
    State(service): State<PemakaianBmnService>,
    Path(id): Path<Uuid>,
    claims: Claims,
    Json(request): Json<UpdateIzinPemakaianRequest>,
) -> Result<Json<ApiResponse<IzinPemakaianBmn>>, AppError> {
    use crate::shared::policy::{PemakaianBmnAction, PemakaianBmnPolicy, WorkflowPolicy};
    // The scoped read is also the object-level check: a permit outside the
    // caller's satker is a 404 here, before any state or role is disclosed.
    let permit_now = service
        .get_permit_detail(id, &SatkerScope::from_claims(&claims))
        .await?
        .izin;
    PemakaianBmnPolicy.authorize(
        &claims,
        PemakaianBmnAction::UpdateDraft,
        Some(permit_now.status.as_str()),
    )?;
    info!("Updating permit {}", id);

    let permit = service
        .update_permit(id, request, claims.user_id, claims.username.clone())
        .await?;

    Ok(Json(ApiResponse::success(
        permit,
        "Permit updated successfully".to_string(),
    )))
}

/// GET /pemakaian-bmn
/// List permits with pagination and filters
/// Returns the shared `PaginatedResponse` envelope — a FLAT `data: [...]` array,
/// same as `analisis`, `bank_aset` and `penghapusan_bmn`. This used to be
/// `ApiResponse<PaginatedPermitsResponse>`, which nested the page struct inside
/// the envelope and put the rows at `data.data`. Nothing consumed that shape:
/// the FE fetcher is typed `PaginatedResponse<IzinPemakaianBmn>` and could not
/// deserialize it, so the list page was broken in the browser, and the e2e
/// helper threw `.map is not a function`.
pub async fn list_permits(
    State(service): State<PemakaianBmnService>,
    axum::extract::Query(query): axum::extract::Query<ListPermitsQuery>,
    claims: Claims,
) -> Result<Json<PaginatedResponse<IzinPemakaianBmn>>, AppError> {
    let scope = crate::shared::satker_scope::SatkerScope::from_claims(&claims);
    let response = service.list_permits(query, &scope).await?;

    Ok(Json(PaginatedResponse::new(
        response.data,
        response.total,
        response.page as i32,
        response.per_page as i32,
        "Permits retrieved successfully",
    )))
}

/// POST /pemakaian-bmn/:id/transition
/// Transition permit to a new workflow status
pub async fn transition_permit_status(
    State(service): State<PemakaianBmnService>,
    Path(id): Path<Uuid>,
    claims: Claims,
    ClientIp(ip): ClientIp,
    Json(request): Json<WorkflowTransitionRequest>,
) -> Result<Json<ApiResponse<IzinPemakaianBmn>>, AppError> {
    use crate::shared::policy::{PemakaianBmnAction, PemakaianBmnPolicy, WorkflowPolicy};
    info!(
        "Transitioning permit {} to status {}",
        id, request.target_status
    );

    // The generic transition is NOT a way around the dedicated endpoints. Those
    // carry the optimistic-lock version, the mandatory return note, and the
    // `validator_satker_id` / `approver_satker_id` stamps the maker-checker rule
    // reads — a move made here would skip all three. So it serves only the two
    // moves that have no dedicated endpoint (submit, cancel), each checked
    // against the same policy the rest of the workflow uses.
    let scope = SatkerScope::from_claims(&claims);
    let permit_now = service.get_permit_detail(id, &scope).await?.izin;
    let from_state = permit_now.status.as_str();
    let action = match PemakaianBmnAction::for_transition(from_state, &request.target_status) {
        Some(a @ (PemakaianBmnAction::Submit | PemakaianBmnAction::Cancel)) => a,
        Some(_) => {
            return Err(AppError::BadRequest(format!(
                "Perpindahan {from_state} → {} dilakukan lewat endpoint aksinya \
                 (validator-satker-action / approver-satker-action / resubmit / revoke)",
                request.target_status
            )));
        }
        None => {
            return Err(AppError::Authorization(format!(
                "Perpindahan {from_state} → {} tidak dapat dilakukan lewat API",
                request.target_status
            )));
        }
    };
    let acting_role = PemakaianBmnPolicy.authorize_as(&claims, action, Some(from_state))?;

    let permit = service
        .transition_permit_status(id, request, claims.user_id, acting_role, ip, &scope)
        .await?;

    Ok(Json(ApiResponse::success(
        permit,
        "Status updated successfully".to_string(),
    )))
}

/// POST /pemakaian-bmn/:id/activate
/// Activate a permit (generate permit number and set to ACTIVE)
pub async fn activate_permit(
    State(service): State<PemakaianBmnService>,
    Path(id): Path<Uuid>,
    claims: Claims,
) -> Result<Json<ApiResponse<IzinPemakaianBmn>>, AppError> {
    use crate::shared::policy::{PemakaianBmnAction, PemakaianBmnPolicy, WorkflowPolicy};
    let scope = SatkerScope::from_claims(&claims);
    // Issuing a permit number is an official act. It had no role check at all —
    // any authenticated user could activate any permit they could name.
    let permit_now = service.get_permit_detail(id, &scope).await?.izin;
    PemakaianBmnPolicy.authorize(
        &claims,
        PemakaianBmnAction::Activate,
        Some(permit_now.status.as_str()),
    )?;
    info!("Activating permit {}", id);

    let permit = service.activate_permit(id, claims.user_id, &scope).await?;

    Ok(Json(ApiResponse::success(
        permit,
        "Permit activated successfully".to_string(),
    )))
}

/// GET /pemakaian-bmn/:id/document
/// Download permit document
///
/// Requirements: REQ-P006, REQ-D005
pub async fn get_permit_document(
    State(service): State<PemakaianBmnService>,
    Path(id): Path<Uuid>,
    claims: Claims,
) -> Result<axum::response::Redirect, AppError> {
    info!("Fetching document for permit {}", id);

    let permit = service
        .get_permit_detail(id, &SatkerScope::from_claims(&claims))
        .await?;

    // Creator or cross-satker role (pusat/admin) may access; everyone else is
    // rejected until authenc exposes per-user satker codes (commit 20 lands a
    // richer ValidateTokenResponse).
    // `created_by` is nullable: a permit with no recorded creator has no
    // creator to match, so only a cross-satker role gets through (fail-closed).
    if permit.izin.created_by != Some(claims.user_id) && !claims.is_cross_satker_role() {
        return Err(AppError::Authorization(
            "Anda tidak memiliki akses ke dokumen izin ini".to_string(),
        ));
    }

    // Redirect to the DOCX konsep (editable). Use `/konsep-surat.pdf` to grab
    // the PDF variant directly. `document_url` is the post-signature artifact
    // and is the right pick once the user has uploaded the signed PDF.
    let document_url = permit
        .izin
        .konsep_surat_url
        .or(permit.izin.signed_pdf_url)
        .or(permit.izin.document_url)
        .ok_or_else(|| AppError::NotFound("Document not found for this permit".to_string()))?;

    // Redirect to document URL
    Ok(axum::response::Redirect::temporary(&document_url))
}

/// POST /pemakaian-bmn/:id/generate-konsep-surat
/// Generate konsep surat izin pemakaian BMN — produces BOTH the editable
/// DOCX and the final PDF side-by-side.
pub async fn generate_konsep_surat(
    State(service): State<PemakaianBmnService>,
    Path(id): Path<Uuid>,
    claims: Claims,
) -> Result<Json<lib_perlengkapan::response::ApiResponse<IzinPemakaianBmn>>, AppError> {
    use crate::shared::policy::{PemakaianBmnAction, PemakaianBmnPolicy, WorkflowPolicy};
    let scope = SatkerScope::from_claims(&claims);
    let permit_now = service.get_permit_detail(id, &scope).await?.izin;
    PemakaianBmnPolicy.authorize(
        &claims,
        PemakaianBmnAction::GenerateDocument,
        Some(permit_now.status.as_str()),
    )?;
    let permit = service
        .generate_konsep_surat(id, claims.user_id, &scope)
        .await?;

    Ok(Json(lib_perlengkapan::response::ApiResponse::success(
        permit,
        "Konsep surat izin pemakaian BMN berhasil digenerate (DOCX + PDF)".to_string(),
    )))
}

/// GET /pemakaian-bmn/:id/konsep-surat.docx
/// GET /pemakaian-bmn/:id/konsep-surat.pdf
///
/// Stream the on-disk konsep surat artifact. `format` is "docx" or "pdf".
/// Path is read from the permit row written by `generate_konsep_surat`.
pub async fn serve_konsep_surat(
    State(service): State<PemakaianBmnService>,
    Path((id, format)): Path<(Uuid, String)>,
    claims: Claims,
) -> Result<axum::response::Response, AppError> {
    use axum::body::Body;
    use axum::http::header;
    use axum::response::IntoResponse;

    let format = format.as_str();
    if format != "docx" && format != "pdf" {
        return Err(AppError::BadRequest(format!(
            "format harus 'docx' atau 'pdf', dapat: '{}'",
            format
        )));
    }

    let path = service
        .konsep_surat_path(id, format, &SatkerScope::from_claims(&claims))
        .await?
        .ok_or_else(|| {
            AppError::NotFound(format!(
                "Konsep surat ({}) belum digenerate untuk permit {}",
                format, id
            ))
        })?;

    let bytes = tokio::fs::read(&path)
        .await
        .map_err(|e| AppError::Internal(format!("read {}: {}", path, e)))?;

    let mime = match format {
        "pdf" => "application/pdf",
        _ => "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
    };
    let filename = format!("konsep-surat-{}.{}", id, format);

    let resp = (
        [
            (header::CONTENT_TYPE, mime),
            (
                header::CONTENT_DISPOSITION,
                &format!("inline; filename=\"{}\"", filename),
            ),
        ],
        Body::from(bytes),
    )
        .into_response();
    Ok(resp)
}

/// GET /pemakaian-bmn/:id/sk-izin.pdf
///
/// Fase 1.10: SK Izin Pemakaian BMN format 2-halaman dgn struktur:
/// - Hal 1: info pegawai (NIP, nama, pangkat, jabatan, satker, foto)
/// - Hal 2: daftar BMN (kode, nama, NUP, merk, tipe, mulai, berakhir)
///
/// Berbeda dgn `/konsep-surat.pdf` yg generic — endpoint ini struktur
/// spesifik sesuai spesifikasi stakeholder (plan §5.1). Stream PDF
/// inline (Content-Disposition: inline) sehingga FE dapat
/// menampilkannya di iframe / new tab.
pub async fn serve_sk_izin_pdf(
    State(service): State<PemakaianBmnService>,
    // The photo on page 1 lives on the MySIMKARI media host, and
    // `layanan-integrasi` is the only pod allowed to reach it. `None` (integrasi
    // not wired) prints the placeholder box rather than failing the download.
    State(integrasi): State<Option<crate::shared::grpc::clients::IntegrasiClient>>,
    Path(id): Path<Uuid>,
    claims: Claims,
) -> Result<axum::response::Response, AppError> {
    use axum::http::header;
    use axum::response::IntoResponse;
    // Measured on staging: this served another satker's 5 057-byte decision
    // letter to an operator with no claim to the permit.
    let bytes = super::sk_izin_pdf::generate_sk_izin_pdf(
        &service,
        id,
        &SatkerScope::from_claims(&claims),
        integrasi.as_ref(),
    )
    .await?;
    let filename = format!("SK-Izin-Pemakaian-BMN-{}.pdf", id);
    let headers = [
        (header::CONTENT_TYPE, "application/pdf"),
        (
            header::CONTENT_DISPOSITION,
            &format!("inline; filename=\"{}\"", filename),
        ),
    ];
    Ok((headers, bytes).into_response())
}

/// POST /pemakaian-bmn/:id/upload-signed-pdf
/// Upload signed PDF izin pemakaian and mark as completed
pub async fn upload_signed_pdf(
    State(service): State<PemakaianBmnService>,
    Path(id): Path<Uuid>,
    claims: Claims,
    Json(body): Json<UploadSignedPdfRequest>,
) -> Result<Json<lib_perlengkapan::response::ApiResponse<IzinPemakaianBmn>>, AppError> {
    use crate::shared::policy::{PemakaianBmnAction, PemakaianBmnPolicy, WorkflowPolicy};
    let scope = SatkerScope::from_claims(&claims);
    let permit_now = service.get_permit_detail(id, &scope).await?.izin;
    PemakaianBmnPolicy.authorize(
        &claims,
        PemakaianBmnAction::UploadSigned,
        Some(permit_now.status.as_str()),
    )?;
    let permit = service
        .upload_signed_pdf(id, body.signed_pdf_url, claims.user_id, &scope)
        .await?;

    Ok(Json(lib_perlengkapan::response::ApiResponse::success(
        permit,
        "PDF izin pemakaian BMN yang ditandatangani berhasil diupload. Proses selesai.".to_string(),
    )))
}

/// POST /pemakaian-bmn/:id/revoke
/// Revoke a permit. Hanya Approver Satker yg boleh; Admin secara eksplisit
/// DIBLOKIR (stakeholder mandate) lewat `enforce_no_admin_revoke`.
pub async fn revoke_permit(
    State(service): State<PemakaianBmnService>,
    Path(id): Path<Uuid>,
    claims: Claims,
    Json(request): Json<RevokePermitRequest>,
) -> Result<Json<ApiResponse<IzinPemakaianBmn>>, AppError> {
    use crate::shared::policy::{
        PemakaianBmnAction, PemakaianBmnPolicy, WorkflowPolicy, enforce_no_admin_revoke,
    };
    // Admin tidak boleh — guard ini di-cek SEBELUM policy.authorize() agar
    // admin bypass di policy.authorize() tidak overwrite stakeholder mandate.
    enforce_no_admin_revoke(&claims)?;
    let permit_now = service
        .get_permit_detail(id, &SatkerScope::from_claims(&claims))
        .await?
        .izin;
    PemakaianBmnPolicy.authorize(
        &claims,
        PemakaianBmnAction::Revoke,
        Some(permit_now.status.as_str()),
    )?;
    info!("Revoking permit {}", id);

    let permit = service
        .revoke_permit(
            id,
            request,
            claims.user_id,
            claims.username.clone(),
            &SatkerScope::from_claims(&claims),
        )
        .await?;

    Ok(Json(ApiResponse::success(
        permit,
        "Permit revoked successfully".to_string(),
    )))
}

/// POST /pemakaian-bmn/:id/renew
/// Renew a permit (create new permit based on existing one)
pub async fn renew_permit(
    State(service): State<PemakaianBmnService>,
    Path(id): Path<Uuid>,
    claims: Claims,
    Json(request): Json<RenewPermitRequest>,
) -> Result<(StatusCode, Json<ApiResponse<IzinPemakaianBmn>>), AppError> {
    use crate::shared::policy::{PemakaianBmnAction, PemakaianBmnPolicy, WorkflowPolicy};
    // A renewal opens a NEW request, so it is a Create.
    PemakaianBmnPolicy.authorize(&claims, PemakaianBmnAction::Create, None)?;
    info!("Renewing permit {}", id);

    let permit = service
        .renew_permit(
            id,
            request,
            claims.user_id,
            claims.username.clone(),
            claims.satker_code.clone(),
            &SatkerScope::from_claims(&claims),
        )
        .await?;

    Ok((
        StatusCode::CREATED,
        Json(ApiResponse::success(
            permit,
            "Permit renewed successfully".to_string(),
        )),
    ))
}

/// Query untuk cek ketersediaan. `kode_barang` wajib — bersama kode satker
/// dan NUP ia melengkapi identitas aset.
#[derive(Debug, serde::Deserialize)]
pub struct AvailabilityQuery {
    pub kode_barang: String,
    /// Satker pemilik aset. Kosong = satker pemanggil sendiri.
    #[serde(default)]
    pub satker_code: Option<String>,
}

/// GET /pemakaian-bmn/bmn/:bmn_nup/availability
/// Check if a BMN is available for new permit
pub async fn check_bmn_availability(
    State(service): State<PemakaianBmnService>,
    Path(bmn_nup): Path<String>,
    axum::extract::Query(query): axum::extract::Query<AvailabilityQuery>,
    claims: Claims,
) -> Result<Json<ApiResponse<BmnAvailabilityResponse>>, AppError> {
    // `kode_barang` wajib: NUP saja tidak mengidentifikasi aset (44.017 aset
    // memakai NUP `1`), jadi tanpa itu jawabannya tak punya arti. Satker
    // default = satker pemanggil; bila disebut eksplisit, harus berada dalam
    // scope pemanggil — di luar itu 404, bukan 403.
    let scope = crate::shared::satker_scope::SatkerScope::from_claims(&claims);
    let response = service
        .check_bmn_availability(AssetIdentityQuery {
            bmn_nup: &bmn_nup,
            bmn_kode_barang: &query.kode_barang,
            scope: &scope,
            milik_sendiri: claims.satker_code.as_deref(),
            satker_diminta: query.satker_code.as_deref(),
        })
        .await?;

    let message = if response.is_available {
        "BMN is available".to_string()
    } else {
        "BMN is currently in use".to_string()
    };

    Ok(Json(ApiResponse::success(response, message)))
}

// ─── Fase 1.11: cek-pegawai + cek-bmn ────────────────────────────────────

#[derive(Debug, serde::Deserialize)]
pub struct CekPegawaiQuery {
    /// Kept only so an existing caller's URL still parses. The satker is
    /// DERIVED from the caller's claims now: taking it from the query let any
    /// authenticated user read another satker's employee record by naming it,
    /// which is personal data (the #870 class).
    #[serde(default)]
    pub satker_id: Option<String>,
}

/// GET /pemakaian-bmn/cek-pegawai/{nip}
///
/// Operator mengetik NIP, sistem mengembalikan identitas pegawai + pemakaian
/// aktif + histori, sehingga tidak ada satu pun field pegawai yang perlu
/// diketik ulang. Endpoint ini sudah ada sejak Fase 1.11 tetapi tidak pernah
/// dipanggil frontend — dan tidak akan berguna kalau dipanggil, karena
/// lookupnya mencocokkan `satker_id` (UUID) dengan sebuah `kode_satker`
/// sehingga tak pernah menemukan siapa pun. Lihat
/// [`crate::shared::pegawai_ref`].
///
/// Cakupan diambil dari klaim, bukan dari query.
pub async fn cek_pegawai(
    State(service): State<PemakaianBmnService>,
    Path(nip): Path<String>,
    axum::extract::Query(_query): axum::extract::Query<CekPegawaiQuery>,
    claims: Claims,
) -> Result<Json<ApiResponse<CekPegawaiResponse>>, AppError> {
    let scope = crate::shared::satker_scope::SatkerScope::from_claims(&claims);
    let resp = service.cek_pegawai_in_satker(&nip, &scope).await?;
    Ok(Json(ApiResponse::success(
        resp,
        "Pegawai terverifikasi".to_string(),
    )))
}

#[derive(Debug, serde::Deserialize)]
pub struct CekBmnQuery {
    pub nup: String,
    /// Wajib: NUP saja tidak mengidentifikasi aset.
    pub kode_barang: String,
    /// Satker pemilik aset. Kosong = satker pemanggil sendiri.
    #[serde(default)]
    pub satker_id: Option<String>,
    pub tgl_mulai: chrono::NaiveDate,
    pub tgl_selesai: chrono::NaiveDate,
}

/// GET /pemakaian-bmn/cek-bmn?nup=...&tgl_mulai=YYYY-MM-DD&tgl_selesai=YYYY-MM-DD
///
/// Fase 1.11: Validate BMN existence + cek availability per periode.
/// Mendukung pemakaian berurutan (existing berakhir sebelum usulan
/// mulai → diizinkan). 422 "BMN tidak ditemukan" jika NUP tidak ada di
/// referensi SIMAN. Status response: Available | PemakaianBerurutan |
/// Overlap.
pub async fn cek_bmn(
    State(service): State<PemakaianBmnService>,
    State(pool): State<deadpool_postgres::Pool>,
    axum::extract::Query(query): axum::extract::Query<CekBmnQuery>,
    claims: Claims,
) -> Result<Json<ApiResponse<CekBmnResponse>>, AppError> {
    // `satker_id` dulu diterima lalu diabaikan, dan `claims` tak dipakai sama
    // sekali (`_claims`) — jadi endpoint ini menjawab secara nasional dan
    // menyebut nama pemegang dari satker mana pun. Kini keduanya dipakai.
    let scope = crate::shared::satker_scope::SatkerScope::from_claims(&claims);
    let resp = service
        .cek_bmn_availability_for_period(
            &pool,
            AssetIdentityQuery {
                bmn_nup: &query.nup,
                bmn_kode_barang: &query.kode_barang,
                scope: &scope,
                milik_sendiri: claims.satker_code.as_deref(),
                satker_diminta: query.satker_id.as_deref(),
            },
            query.tgl_mulai,
            query.tgl_selesai,
        )
        .await?;
    Ok(Json(ApiResponse::success(
        resp,
        "Cek BMN selesai".to_string(),
    )))
}

/// GET /pemakaian-bmn/bmn/:bmn_nup/history
/// Get BMN usage history
pub async fn get_bmn_usage_history(
    State(service): State<PemakaianBmnService>,
    Path(bmn_nup): Path<String>,
    claims: Claims,
) -> Result<Json<ApiResponse<BmnUsageStats>>, AppError> {
    let scope = crate::shared::satker_scope::SatkerScope::from_claims(&claims);
    let stats = service.get_bmn_usage_history(&bmn_nup, &scope).await?;

    Ok(Json(ApiResponse::success(
        stats,
        "BMN usage history retrieved successfully".to_string(),
    )))
}

/// GET /pemakaian-bmn/pegawai/:pegawai_nip/history
/// Get pegawai usage history
pub async fn get_pegawai_usage_history(
    State(service): State<PemakaianBmnService>,
    Path(pegawai_nip): Path<String>,
    claims: Claims,
) -> Result<Json<ApiResponse<PegawaiUsageStats>>, AppError> {
    let scope = crate::shared::satker_scope::SatkerScope::from_claims(&claims);
    let stats = service
        .get_pegawai_usage_history(&pegawai_nip, &scope)
        .await?;

    Ok(Json(ApiResponse::success(
        stats,
        "Pegawai usage history retrieved successfully".to_string(),
    )))
}

/// GET /pemakaian-bmn/expiring
/// Get permits expiring soon (for notifications)
pub async fn get_expiring_permits(
    State(service): State<PemakaianBmnService>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
    claims: Claims,
) -> Result<Json<ApiResponse<Vec<IzinPemakaianBmn>>>, AppError> {
    let days_threshold = params
        .get("days")
        .and_then(|d| d.parse::<i32>().ok())
        .unwrap_or(30);

    let scope = crate::shared::satker_scope::SatkerScope::from_claims(&claims);
    let permits = service.get_expiring_permits(days_threshold, &scope).await?;
    let count = permits.len();

    Ok(Json(ApiResponse::success(
        permits,
        format!(
            "Found {} permits expiring within {} days",
            count, days_threshold
        ),
    )))
}

/// POST /pemakaian-bmn/auto-expire
/// Auto-expire permits (admin/scheduler endpoint)
pub async fn auto_expire_permits(
    State(service): State<PemakaianBmnService>,
    claims: Claims,
) -> Result<Json<ApiResponse<usize>>, AppError> {
    // A maintenance sweep across EVERY satker's permits; it had no check, so any
    // authenticated user could trigger it. The scheduler runs it in-process and
    // does not come through here.
    claims.require_capability(lib_core::authz::Capability::Administer)?;
    let count = service.auto_expire_permits().await?;

    Ok(Json(ApiResponse::success(
        count,
        format!("Auto-expired {} permits", count),
    )))
}

// ============================================================================
// Monitoring Dashboard Handlers
// ============================================================================

/// GET /pemakaian-bmn/monitoring/summary
/// Tiga kartu agregat headline: sedang dipakai / tidak dipakai / akan expired.
/// Read-only — audiens Validator Wilayah & Pusat (Fase 2.6).
pub async fn get_monitoring_summary(
    State(service): State<PemakaianBmnService>,
    axum::extract::Query(query): axum::extract::Query<MonitoringDashboardQuery>,
    claims: Claims,
) -> Result<Json<ApiResponse<MonitoringSummaryCards>>, AppError> {
    crate::shared::policy::enforce_monitoring_read(&claims)?;
    // Role guard says WHETHER you may read monitoring; scope says WHICH rows.
    // Only the first existed, so any monitoring role reading without a filter
    // got the national picture.
    let scope = crate::shared::satker_scope::SatkerScope::from_claims(&claims);
    let aset_scope = crate::bank_aset::scope::AsetScope::from_claims(&claims);
    let summary = service
        .get_monitoring_summary(query, &scope, &aset_scope)
        .await?;

    Ok(Json(ApiResponse::success(
        summary,
        "Ringkasan monitoring pemakaian BMN".to_string(),
    )))
}

/// GET /pemakaian-bmn/monitoring/active-usage
/// Get active usage monitoring dashboard
/// Requirements: REQ-P011
pub async fn get_active_usage_dashboard(
    State(service): State<PemakaianBmnService>,
    axum::extract::Query(query): axum::extract::Query<MonitoringDashboardQuery>,
    claims: Claims,
) -> Result<Json<ApiResponse<ActiveUsageMonitoringDashboard>>, AppError> {
    crate::shared::policy::enforce_monitoring_read(&claims)?;
    let scope = crate::shared::satker_scope::SatkerScope::from_claims(&claims);
    let dashboard = service.get_active_usage_dashboard(query, &scope).await?;

    Ok(Json(ApiResponse::success(
        dashboard,
        "Active usage dashboard retrieved successfully".to_string(),
    )))
}

/// GET /pemakaian-bmn/monitoring/pemakaian
///
/// Daftar pemakaian BMN yang dapat dilihat pemanggil: satker mana, nama
/// barangnya, NUP berapa, siapa pegawai yang memakai, dan berapa jangka waktu
/// pemakaiannya. Batasnya turun dari klaim — pusat melihat semua, validator
/// wilayah sebatas wilayahnya, operator/validator satker sebatas satkernya,
/// dan pemanggil tanpa identitas satker tidak melihat apa pun.
pub async fn list_pemakaian_monitoring(
    State(service): State<PemakaianBmnService>,
    axum::extract::Query(query): axum::extract::Query<PemakaianMonitoringQuery>,
    claims: Claims,
) -> Result<Json<ApiResponse<PemakaianBmnMonitoringPage>>, AppError> {
    crate::shared::policy::enforce_monitoring_read(&claims)?;
    let scope = crate::shared::satker_scope::SatkerScope::from_claims(&claims);
    let page = service.list_pemakaian_monitoring(query, &scope).await?;

    Ok(Json(ApiResponse::success(
        page,
        "Daftar pemakaian BMN".to_string(),
    )))
}

// ============================================================================
// V035 (Fase 1.5): Endpoints alur internal-satker 3-step.
//
// RBAC ditegakkan lewat `PemakaianBmnPolicy` (tanpa bypass admin) ditambah
// maker-checker (`enforce_maker_checker`): pengusul tidak boleh memvalidasi,
// dan pemberi persetujuan tidak boleh pengusul/validator usulan yang sama.
// ============================================================================

use validator::Validate as _SatkerValidate;

/// POST /pemakaian-bmn/{id}/validator-satker-action
/// Body: { "action": "forward"|"return", "expected_version": i32, "catatan": "..." }
#[derive(Debug, serde::Deserialize)]
#[serde(tag = "action", rename_all = "lowercase")]
pub enum ValidatorSatkerActionRequest {
    Forward(SatkerForwardRequest),
    Return(SatkerReturnRequest),
}

pub async fn validator_satker_action(
    State(service): State<PemakaianBmnService>,
    Path(id): Path<Uuid>,
    claims: Claims,
    Json(request): Json<ValidatorSatkerActionRequest>,
) -> Result<Json<ApiResponse<IzinPemakaianBmn>>, AppError> {
    use crate::shared::policy::{PemakaianBmnAction, PemakaianBmnPolicy, WorkflowPolicy};
    let permit_now = service
        .get_permit_detail(id, &SatkerScope::from_claims(&claims))
        .await?
        .izin;
    let state = Some(permit_now.status.as_str());
    let action = match &request {
        ValidatorSatkerActionRequest::Forward(_) => PemakaianBmnAction::ValidatorSatkerForward,
        ValidatorSatkerActionRequest::Return(_) => PemakaianBmnAction::ValidatorSatkerReturn,
    };
    PemakaianBmnPolicy.authorize(&claims, action, state)?;
    crate::shared::policy::enforce_maker_checker(
        action,
        claims.user_id,
        permit_now.created_by,
        permit_now.validator_satker_id,
    )?;
    let permit = match request {
        ValidatorSatkerActionRequest::Forward(req) => {
            service
                .validator_satker_forward(
                    id,
                    claims.user_id,
                    claims.username.clone(),
                    req.expected_version,
                    req.catatan,
                )
                .await?
        }
        ValidatorSatkerActionRequest::Return(req) => {
            req.validate()?;
            service
                .validator_satker_return(
                    id,
                    claims.user_id,
                    claims.username.clone(),
                    req.expected_version,
                    req.catatan,
                )
                .await?
        }
    };
    Ok(Json(ApiResponse::success(
        permit,
        "Aksi Validator Satker berhasil".to_string(),
    )))
}

/// POST /pemakaian-bmn/{id}/approver-satker-action
#[derive(Debug, serde::Deserialize)]
#[serde(tag = "action", rename_all = "lowercase")]
pub enum ApproverSatkerActionRequest {
    Approve(SatkerForwardRequest),
    Return(SatkerReturnRequest),
}

pub async fn approver_satker_action(
    State(service): State<PemakaianBmnService>,
    Path(id): Path<Uuid>,
    claims: Claims,
    Json(request): Json<ApproverSatkerActionRequest>,
) -> Result<Json<ApiResponse<IzinPemakaianBmn>>, AppError> {
    use crate::shared::policy::{PemakaianBmnAction, PemakaianBmnPolicy, WorkflowPolicy};
    let permit_now = service
        .get_permit_detail(id, &SatkerScope::from_claims(&claims))
        .await?
        .izin;
    let state = Some(permit_now.status.as_str());
    let action = match &request {
        ApproverSatkerActionRequest::Approve(_) => PemakaianBmnAction::ApproverSatkerApprove,
        ApproverSatkerActionRequest::Return(_) => PemakaianBmnAction::ApproverSatkerReturn,
    };
    PemakaianBmnPolicy.authorize(&claims, action, state)?;
    crate::shared::policy::enforce_maker_checker(
        action,
        claims.user_id,
        permit_now.created_by,
        permit_now.validator_satker_id,
    )?;
    let permit = match request {
        ApproverSatkerActionRequest::Approve(req) => {
            service
                .approver_satker_approve(
                    id,
                    claims.user_id,
                    claims.username.clone(),
                    req.expected_version,
                    req.catatan,
                )
                .await?
        }
        ApproverSatkerActionRequest::Return(req) => {
            req.validate()?;
            service
                .approver_satker_return(
                    id,
                    claims.user_id,
                    claims.username.clone(),
                    req.expected_version,
                    req.catatan,
                )
                .await?
        }
    };
    Ok(Json(ApiResponse::success(
        permit,
        "Aksi Approver Satker berhasil".to_string(),
    )))
}

/// POST /pemakaian-bmn/{id}/resubmit
pub async fn operator_resubmit(
    State(service): State<PemakaianBmnService>,
    Path(id): Path<Uuid>,
    claims: Claims,
    Json(request): Json<OperatorResubmitRequest>,
) -> Result<Json<ApiResponse<IzinPemakaianBmn>>, AppError> {
    use crate::shared::policy::{PemakaianBmnAction, PemakaianBmnPolicy, WorkflowPolicy};
    let permit_now = service
        .get_permit_detail(id, &SatkerScope::from_claims(&claims))
        .await?
        .izin;
    PemakaianBmnPolicy.authorize(
        &claims,
        PemakaianBmnAction::Resubmit,
        Some(permit_now.status.as_str()),
    )?;
    let permit = service
        .operator_resubmit(
            id,
            claims.user_id,
            claims.username.clone(),
            request.expected_version,
        )
        .await?;
    Ok(Json(ApiResponse::success(
        permit,
        "Usulan berhasil di-resubmit ke Validator Satker".to_string(),
    )))
}
