//! Dashboard API — the recap export.
//!
//! `fetch_dashboard_stats` used to live here, against `GET /dashboard/stats`.
//! That endpoint was a second aggregate over the same SIMAN table whose payload
//! was a strict subset of `GET /bank-aset/dashboard` — and unlike that one it
//! was never scoped, so it answered every operator with all 624 533 national
//! assets while the bank-aset card beside it correctly answered their own
//! 1 681. The home page now reads the scoped superset via
//! [`crate::api::bank_aset::fetch_dashboard`]; there is one definition again.

/// Download the perlengkapan dashboard recap as `format` ("excel" | "pdf").
///
/// `tahun_anggaran` is REQUIRED by the endpoint (`DashboardParams.tahun_anggaran`
/// is a bare `i32`), so omitting it is a legitimate 400 — the caller always
/// passes one.
///
/// These two endpoints had no FE caller at all (#97), which is exactly why the
/// 500 they returned went unnoticed until an e2e probed them directly (#116).
#[cfg(target_arch = "wasm32")]
pub async fn export_dashboard(
    format: &str,
    tahun_anggaran: i32,
) -> Result<Vec<u8>, crate::api::AppError> {
    super::client::auth_get_binary(&format!(
        "{}/dashboard/perlengkapan/export/{format}?tahun_anggaran={tahun_anggaran}",
        super::client::API_BASE
    ))
    .await
}
