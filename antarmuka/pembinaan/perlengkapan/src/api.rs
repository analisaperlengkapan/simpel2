use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Asset {
    pub id: String,
    pub kategori_aset: String,
    pub no_aset: String,
    pub nama_aset: Option<String>,
    pub kode_barang: Option<String>,
    pub merk: Option<String>,
    pub tipe: Option<String>,
    pub kondisi: Option<String>,
    pub lokasi: Option<String>,
    pub satker: Option<String>,
    pub nilai_perolehan: Option<f64>,
    pub tgl_perolehan: Option<String>,
    pub updated_at: String,
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

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DashboardStats {
    pub total_aset: i64,
    pub total_nilai_aset: f64,
    pub total_satker: i64,
    pub aset_baik: i64,
    pub aset_rusak: i64,
    pub categories: Vec<CategoryStat>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CategoryStat {
    pub category: String,
    pub count: i64,
    pub value: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: T,
    pub message: String,
}

#[cfg(target_arch = "wasm32")]
pub async fn fetch_assets(page: i32, per_page: i32, category: Option<String>) -> Result<PaginatedResponse<Asset>, gloo_net::Error> {
    use gloo_net::http::Request;
    use crate::components::auth::get_auth_token;

    let mut url = format!("/api/pembinaan/perlengkapan/assets?page={}&per_page={}", page, per_page);
    if let Some(cat) = category {
        use gloo_net::http::QueryParams;
        // Simple append because gloo_net doesn't have easy query builder for existing URL string
        // But better is to just append string
        use urlencoding::encode;
        url.push_str(&format!("&category={}", encode(&cat)));
    }

    let token = get_auth_token().unwrap_or_default();
    let resp = Request::get(&url)
        .header("Authorization", &format!("Bearer {}", token))
        .send()
        .await?;

    if !resp.ok() {
         return Err(gloo_net::Error::GlooError(format!("API Error: {}", resp.status())));
    }

    let result: PaginatedResponse<Asset> = resp.json().await?;
    Ok(result)
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn fetch_assets(_page: i32, _per_page: i32, _category: Option<String>) -> Result<PaginatedResponse<Asset>, String> {
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
            total_nilai_aset: 0.0,
            total_satker: 0,
            aset_baik: 0,
            aset_rusak: 0,
            categories: vec![],
        },
        message: "Server-side stub".to_string(),
    })
}

#[cfg(target_arch = "wasm32")]
pub async fn fetch_asset_by_id(id: String) -> Result<ApiResponse<Asset>, gloo_net::Error> {
    use gloo_net::http::Request;
    use crate::components::auth::get_auth_token;

    let url = format!("/api/pembinaan/perlengkapan/assets/{}", id);

    let token = get_auth_token().unwrap_or_default();
    let resp = Request::get(&url)
        .header("Authorization", &format!("Bearer {}", token))
        .send()
        .await?;

    if !resp.ok() {
         return Err(gloo_net::Error::GlooError(format!("API Error: {}", resp.status())));
    }

    let result: ApiResponse<Asset> = resp.json().await?;
    Ok(result)
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn fetch_asset_by_id(_id: String) -> Result<ApiResponse<Asset>, String> {
    Err("Server-side stub".to_string())
}
