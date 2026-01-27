use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Aset {
    pub id: String,
    pub nama: String,
    pub kategori: String,
    pub kode_bmn: String,
    pub merk: Option<String>,
    pub nup: Option<String>,
    pub kondisi: String,
    pub lokasi: String,
    pub nilai_perolehan: Option<f64>,
    pub tanggal_perolehan: Option<String>,
    pub status: String,
    pub keterangan: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PaginatedResponse<T> {
    pub success: bool,
    pub data: Vec<T>,
    pub total: i64,
    pub page: i32,
    pub per_page: i32,
    pub total_pages: i32,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DashboardStats {
    pub total_aset: i64,
    pub total_pengadaan: i64,
    pub total_analisis: i64,
    pub aset_aktif: i64,
    pub pengadaan_berjalan: i64,
    pub analisis_pending: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: T,
    pub message: String,
}

#[cfg(target_arch = "wasm32")]
pub async fn fetch_assets(page: i32, per_page: i32) -> Result<PaginatedResponse<Aset>, gloo_net::Error> {
    use gloo_net::http::Request;
    use crate::components::auth::get_auth_token;

    // Assuming the API is available at /api/pembinaan/perlengkapan/aset
    // Adjust the path if necessary (e.g. via proxy or absolute URL)
    let url = format!("/api/pembinaan/perlengkapan/aset?page={}&per_page={}", page, per_page);

    let token = get_auth_token().unwrap_or_default();
    let resp = Request::get(&url)
        .header("Authorization", &format!("Bearer {}", token))
        .send()
        .await?;

    if !resp.ok() {
         return Err(gloo_net::Error::GlooError(format!("API Error: {}", resp.status())));
    }

    let result: PaginatedResponse<Aset> = resp.json().await?;
    Ok(result)
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn fetch_assets(_page: i32, _per_page: i32) -> Result<PaginatedResponse<Aset>, String> {
    // Stub for server-side rendering or non-wasm environments
    Ok(PaginatedResponse {
        success: true,
        data: vec![],
        total: 0,
        page: 1,
        per_page: 20,
        total_pages: 0,
        message: "Server-side stub".to_string(),
    })
}

#[cfg(target_arch = "wasm32")]
pub async fn fetch_dashboard_stats() -> Result<ApiResponse<DashboardStats>, gloo_net::Error> {
    use gloo_net::http::Request;
    use crate::components::auth::get_auth_token;

    let url = "/api/pembinaan/perlengkapan/dashboard/stats";

    let token = get_auth_token().unwrap_or_default();
    let resp = Request::get(url)
        .header("Authorization", &format!("Bearer {}", token))
        .send()
        .await?;

    if !resp.ok() {
         return Err(gloo_net::Error::GlooError(format!("API Error: {}", resp.status())));
    }

    let result: ApiResponse<DashboardStats> = resp.json().await?;
    Ok(result)
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn fetch_dashboard_stats() -> Result<ApiResponse<DashboardStats>, String> {
     Ok(ApiResponse {
        success: true,
        data: DashboardStats {
            total_aset: 0,
            total_pengadaan: 0,
            total_analisis: 0,
            aset_aktif: 0,
            pengadaan_berjalan: 0,
            analisis_pending: 0,
        },
        message: "Server-side stub".to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_aset_deserialization() {
        let json_data = json!({
            "id": "550e8400-e29b-41d4-a716-446655440000",
            "nama": "Laptop Dell",
            "kategori": "Elektronik",
            "kode_bmn": "101.1",
            "merk": "Dell Latitude",
            "nup": "1",
            "kondisi": "baik",
            "lokasi": "Ruang IT",
            "nilai_perolehan": 15000000.0,
            "tanggal_perolehan": "2023-01-01",
            "status": "aktif",
            "keterangan": "Pengadaan 2023"
        });

        let aset: Aset = serde_json::from_value(json_data).expect("Failed to deserialize Aset");

        assert_eq!(aset.nama, "Laptop Dell");
        assert_eq!(aset.nilai_perolehan, Some(15000000.0));
        assert_eq!(aset.kondisi, "baik");
    }

    #[test]
    fn test_paginated_response_deserialization() {
        let json_data = json!({
            "success": true,
            "data": [],
            "total": 0,
            "page": 1,
            "per_page": 20,
            "total_pages": 0,
            "message": "Success"
        });

        let resp: PaginatedResponse<Aset> = serde_json::from_value(json_data).expect("Failed to deserialize PaginatedResponse");

        assert!(resp.success);
        assert_eq!(resp.total, 0);
        assert_eq!(resp.data.len(), 0);
    }

    #[test]
    fn test_dashboard_stats_deserialization() {
        let json_data = json!({
            "total_aset": 100,
            "total_pengadaan": 10,
            "total_analisis": 5,
            "aset_aktif": 90,
            "pengadaan_berjalan": 8,
            "analisis_pending": 2
        });

        let stats: DashboardStats = serde_json::from_value(json_data).expect("Failed to deserialize DashboardStats");
        assert_eq!(stats.total_aset, 100);
    }
}
