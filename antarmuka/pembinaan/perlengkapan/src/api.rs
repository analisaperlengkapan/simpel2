pub use lib_perlengkapan::models::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;

// ============ API Client Functions ============

#[cfg(target_arch = "wasm32")]
pub async fn fetch_assets(
    page: i32,
    per_page: i32,
    category: Option<String>,
) -> Result<PaginatedResponse<Asset>, gloo_net::Error> {
    use crate::components::auth::get_auth_token;
    use gloo_net::http::Request;

    let mut url = format!(
        "/api/pembinaan/perlengkapan/assets?page={}&per_page={}",
        page, per_page
    );
    if let Some(cat) = category {
        use urlencoding::encode;
        url.push_str(&format!("&category={}", encode(&cat)));
    }

    let token = get_auth_token()
        .ok_or_else(|| gloo_net::Error::GlooError("No authentication token found".to_string()))?;
    let resp = Request::get(&url)
        .header("Authorization", &format!("Bearer {}", token))
        .send()
        .await?;

    if !resp.ok() {
        return Err(gloo_net::Error::GlooError(format!(
            "API Error: {}",
            resp.status()
        )));
    }

    let result: PaginatedResponse<Asset> = resp.json().await?;
    Ok(result)
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn fetch_assets(
    _page: i32,
    _per_page: i32,
    _category: Option<String>,
) -> Result<PaginatedResponse<Asset>, String> {
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
    use crate::components::auth::get_auth_token;
    use gloo_net::http::Request;

    let url = "/api/pembinaan/perlengkapan/dashboard/stats";

    let token = get_auth_token()
        .ok_or_else(|| gloo_net::Error::GlooError("No authentication token found".to_string()))?;
    let resp = Request::get(url)
        .header("Authorization", &format!("Bearer {}", token))
        .send()
        .await?;

    if !resp.ok() {
        return Err(gloo_net::Error::GlooError(format!(
            "API Error: {}",
            resp.status()
        )));
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
    use crate::components::auth::get_auth_token;
    use gloo_net::http::Request;

    let url = format!("/api/pembinaan/perlengkapan/assets/{}", id);

    let token = get_auth_token()
        .ok_or_else(|| gloo_net::Error::GlooError("No authentication token found".to_string()))?;
    let resp = Request::get(&url)
        .header("Authorization", &format!("Bearer {}", token))
        .send()
        .await?;

    if !resp.ok() {
        return Err(gloo_net::Error::GlooError(format!(
            "API Error: {}",
            resp.status()
        )));
    }

    let result: ApiResponse<Asset> = resp.json().await?;
    Ok(result)
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn fetch_asset_by_id(_id: String) -> Result<ApiResponse<Asset>, String> {
    Err("Server-side stub".to_string())
}

#[cfg(target_arch = "wasm32")]
pub async fn fetch_pengadaan(
    page: i32,
    per_page: i32,
) -> Result<PaginatedResponse<Pengadaan>, gloo_net::Error> {
    use crate::components::auth::get_auth_token;
    use gloo_net::http::Request;

    let url = format!(
        "/api/pembinaan/perlengkapan/pengadaan?page={}&per_page={}",
        page, per_page
    );

    let token = get_auth_token()
        .ok_or_else(|| gloo_net::Error::GlooError("No authentication token found".to_string()))?;
    let resp = Request::get(&url)
        .header("Authorization", &format!("Bearer {}", token))
        .send()
        .await?;

    if !resp.ok() {
        return Err(gloo_net::Error::GlooError(format!(
            "API Error: {}",
            resp.status()
        )));
    }

    let result: PaginatedResponse<Pengadaan> = resp.json().await?;
    Ok(result)
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn fetch_pengadaan(
    _page: i32,
    _per_page: i32,
) -> Result<PaginatedResponse<Pengadaan>, String> {
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
pub async fn create_pengadaan(
    request: CreatePengadaanRequest,
) -> Result<ApiResponse<Pengadaan>, gloo_net::Error> {
    use crate::components::auth::get_auth_token;
    use gloo_net::http::Request;

    let url = "/api/pembinaan/perlengkapan/pengadaan";
    let token = get_auth_token()
        .ok_or_else(|| gloo_net::Error::GlooError("No authentication token found".to_string()))?;

    let resp = Request::post(url)
        .header("Authorization", &format!("Bearer {}", token))
        .json(&request)?
        .send()
        .await?;

    if !resp.ok() {
        return Err(gloo_net::Error::GlooError(format!(
            "API Error: {}",
            resp.status()
        )));
    }

    let result: ApiResponse<Pengadaan> = resp.json().await?;
    Ok(result)
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn create_pengadaan(
    _request: CreatePengadaanRequest,
) -> Result<ApiResponse<Pengadaan>, String> {
    Err("Server-side stub".to_string())
}

#[cfg(target_arch = "wasm32")]
pub async fn fetch_pengadaan_hps(
    pengadaan_id: String,
) -> Result<ApiResponse<Vec<PengadaanHps>>, gloo_net::Error> {
    use crate::components::auth::get_auth_token;
    use gloo_net::http::Request;

    let url = format!("/api/pembinaan/perlengkapan/pengadaan/{}/hps", pengadaan_id);
    let token = get_auth_token()
        .ok_or_else(|| gloo_net::Error::GlooError("No authentication token found".to_string()))?;

    let resp = Request::get(&url)
        .header("Authorization", &format!("Bearer {}", token))
        .send()
        .await?;

    if !resp.ok() {
        return Err(gloo_net::Error::GlooError(format!(
            "API Error: {}",
            resp.status()
        )));
    }

    let result: ApiResponse<Vec<PengadaanHps>> = resp.json().await?;
    Ok(result)
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn fetch_pengadaan_hps(
    _pengadaan_id: String,
) -> Result<ApiResponse<Vec<PengadaanHps>>, String> {
    Err("Server-side stub".to_string())
}

#[cfg(target_arch = "wasm32")]
pub async fn create_pengadaan_hps(
    request: CreatePengadaanHpsRequest,
) -> Result<ApiResponse<PengadaanHps>, gloo_net::Error> {
    use crate::components::auth::get_auth_token;
    use gloo_net::http::Request;

    let url = format!(
        "/api/pembinaan/perlengkapan/pengadaan/{}/hps",
        request.pengadaan_id
    );
    let token = get_auth_token()
        .ok_or_else(|| gloo_net::Error::GlooError("No authentication token found".to_string()))?;

    let resp = Request::post(&url)
        .header("Authorization", &format!("Bearer {}", token))
        .json(&request)?
        .send()
        .await?;

    if !resp.ok() {
        return Err(gloo_net::Error::GlooError(format!(
            "API Error: {}",
            resp.status()
        )));
    }

    let result: ApiResponse<PengadaanHps> = resp.json().await?;
    Ok(result)
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn create_pengadaan_hps(
    _request: CreatePengadaanHpsRequest,
) -> Result<ApiResponse<PengadaanHps>, String> {
    Err("Server-side stub".to_string())
}

#[cfg(target_arch = "wasm32")]
pub async fn fetch_analisis(
    page: i32,
    per_page: i32,
) -> Result<PaginatedResponse<AnalisisKebutuhan>, gloo_net::Error> {
    use crate::components::auth::get_auth_token;
    use gloo_net::http::Request;

    let url = format!(
        "/api/pembinaan/perlengkapan/analisis?page={}&per_page={}",
        page, per_page
    );

    let token = get_auth_token()
        .ok_or_else(|| gloo_net::Error::GlooError("No authentication token found".to_string()))?;
    let resp = Request::get(&url)
        .header("Authorization", &format!("Bearer {}", token))
        .send()
        .await?;

    if !resp.ok() {
        return Err(gloo_net::Error::GlooError(format!(
            "API Error: {}",
            resp.status()
        )));
    }

    let result: PaginatedResponse<AnalisisKebutuhan> = resp.json().await?;
    Ok(result)
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn fetch_analisis(
    _page: i32,
    _per_page: i32,
) -> Result<PaginatedResponse<AnalisisKebutuhan>, String> {
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
pub async fn create_analisis(
    request: CreateAnalisisRequest,
) -> Result<ApiResponse<AnalisisKebutuhan>, gloo_net::Error> {
    use crate::components::auth::get_auth_token;
    use gloo_net::http::Request;

    let url = "/api/pembinaan/perlengkapan/analisis";
    let token = get_auth_token()
        .ok_or_else(|| gloo_net::Error::GlooError("No authentication token found".to_string()))?;

    let resp = Request::post(url)
        .header("Authorization", &format!("Bearer {}", token))
        .json(&request)?
        .send()
        .await?;

    if !resp.ok() {
        return Err(gloo_net::Error::GlooError(format!(
            "API Error: {}",
            resp.status()
        )));
    }

    let result: ApiResponse<AnalisisKebutuhan> = resp.json().await?;
    Ok(result)
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn create_analisis(
    _request: CreateAnalisisRequest,
) -> Result<ApiResponse<AnalisisKebutuhan>, String> {
    Err("Server-side stub".to_string())
}

#[cfg(target_arch = "wasm32")]
pub async fn fetch_pemakaian(
    page: i32,
    per_page: i32,
) -> Result<PaginatedResponse<Pemakaian>, gloo_net::Error> {
    use crate::components::auth::get_auth_token;
    use gloo_net::http::Request;

    let url = format!(
        "/api/pembinaan/perlengkapan/pemakaian?page={}&per_page={}",
        page, per_page
    );

    let token = get_auth_token()
        .ok_or_else(|| gloo_net::Error::GlooError("No authentication token found".to_string()))?;
    let resp = Request::get(&url)
        .header("Authorization", &format!("Bearer {}", token))
        .send()
        .await?;

    if !resp.ok() {
        return Err(gloo_net::Error::GlooError(format!(
            "API Error: {}",
            resp.status()
        )));
    }

    let result: PaginatedResponse<Pemakaian> = resp.json().await?;
    Ok(result)
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn fetch_pemakaian(
    _page: i32,
    _per_page: i32,
) -> Result<PaginatedResponse<Pemakaian>, String> {
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
pub async fn create_pemakaian(
    request: CreatePemakaianRequest,
) -> Result<ApiResponse<Pemakaian>, gloo_net::Error> {
    use crate::components::auth::get_auth_token;
    use gloo_net::http::Request;

    let url = "/api/pembinaan/perlengkapan/pemakaian";
    let token = get_auth_token()
        .ok_or_else(|| gloo_net::Error::GlooError("No authentication token found".to_string()))?;

    let resp = Request::post(url)
        .header("Authorization", &format!("Bearer {}", token))
        .json(&request)?
        .send()
        .await?;

    if !resp.ok() {
        return Err(gloo_net::Error::GlooError(format!(
            "API Error: {}",
            resp.status()
        )));
    }

    let result: ApiResponse<Pemakaian> = resp.json().await?;
    Ok(result)
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn create_pemakaian(
    _request: CreatePemakaianRequest,
) -> Result<ApiResponse<Pemakaian>, String> {
    Err("Server-side stub".to_string())
}

#[cfg(target_arch = "wasm32")]
pub async fn fetch_hibah(
    page: i32,
    per_page: i32,
) -> Result<PaginatedResponse<Hibah>, gloo_net::Error> {
    use crate::components::auth::get_auth_token;
    use gloo_net::http::Request;

    let url = format!(
        "/api/pembinaan/perlengkapan/hibah?page={}&per_page={}",
        page, per_page
    );

    let token = get_auth_token()
        .ok_or_else(|| gloo_net::Error::GlooError("No authentication token found".to_string()))?;
    let resp = Request::get(&url)
        .header("Authorization", &format!("Bearer {}", token))
        .send()
        .await?;

    if !resp.ok() {
        return Err(gloo_net::Error::GlooError(format!(
            "API Error: {}",
            resp.status()
        )));
    }

    let result: PaginatedResponse<Hibah> = resp.json().await?;
    Ok(result)
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn fetch_hibah(_page: i32, _per_page: i32) -> Result<PaginatedResponse<Hibah>, String> {
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
pub async fn create_hibah(
    request: CreateHibahRequest,
) -> Result<ApiResponse<Hibah>, gloo_net::Error> {
    use crate::components::auth::get_auth_token;
    use gloo_net::http::Request;

    let url = "/api/pembinaan/perlengkapan/hibah";
    let token = get_auth_token()
        .ok_or_else(|| gloo_net::Error::GlooError("No authentication token found".to_string()))?;

    let resp = Request::post(url)
        .header("Authorization", &format!("Bearer {}", token))
        .json(&request)?
        .send()
        .await?;

    if !resp.ok() {
        return Err(gloo_net::Error::GlooError(format!(
            "API Error: {}",
            resp.status()
        )));
    }

    let result: ApiResponse<Hibah> = resp.json().await?;
    Ok(result)
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn create_hibah(_request: CreateHibahRequest) -> Result<ApiResponse<Hibah>, String> {
    Err("Server-side stub".to_string())
}

#[cfg(target_arch = "wasm32")]
pub async fn fetch_mutasi(
    page: i32,
    per_page: i32,
) -> Result<PaginatedResponse<Mutasi>, gloo_net::Error> {
    use crate::components::auth::get_auth_token;
    use gloo_net::http::Request;

    let url = format!(
        "/api/pembinaan/perlengkapan/mutasi?page={}&per_page={}",
        page, per_page
    );

    let token = get_auth_token()
        .ok_or_else(|| gloo_net::Error::GlooError("No authentication token found".to_string()))?;
    let resp = Request::get(&url)
        .header("Authorization", &format!("Bearer {}", token))
        .send()
        .await?;

    if !resp.ok() {
        return Err(gloo_net::Error::GlooError(format!(
            "API Error: {}",
            resp.status()
        )));
    }

    let result: PaginatedResponse<Mutasi> = resp.json().await?;
    Ok(result)
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn fetch_mutasi(_page: i32, _per_page: i32) -> Result<PaginatedResponse<Mutasi>, String> {
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
pub async fn create_mutasi(
    request: CreateMutasiRequest,
) -> Result<ApiResponse<Mutasi>, gloo_net::Error> {
    use crate::components::auth::get_auth_token;
    use gloo_net::http::Request;

    let url = "/api/pembinaan/perlengkapan/mutasi";
    let token = get_auth_token()
        .ok_or_else(|| gloo_net::Error::GlooError("No authentication token found".to_string()))?;

    let resp = Request::post(url)
        .header("Authorization", &format!("Bearer {}", token))
        .json(&request)?
        .send()
        .await?;

    if !resp.ok() {
        return Err(gloo_net::Error::GlooError(format!(
            "API Error: {}",
            resp.status()
        )));
    }

    let result: ApiResponse<Mutasi> = resp.json().await?;
    Ok(result)
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn create_mutasi(_request: CreateMutasiRequest) -> Result<ApiResponse<Mutasi>, String> {
    Err("Server-side stub".to_string())
}

#[cfg(target_arch = "wasm32")]
pub async fn fetch_penghapusan(
    page: i32,
    per_page: i32,
) -> Result<PaginatedResponse<Penghapusan>, gloo_net::Error> {
    use crate::components::auth::get_auth_token;
    use gloo_net::http::Request;

    let url = format!(
        "/api/pembinaan/perlengkapan/penghapusan?page={}&per_page={}",
        page, per_page
    );

    let token = get_auth_token()
        .ok_or_else(|| gloo_net::Error::GlooError("No authentication token found".to_string()))?;
    let resp = Request::get(&url)
        .header("Authorization", &format!("Bearer {}", token))
        .send()
        .await?;

    if !resp.ok() {
        return Err(gloo_net::Error::GlooError(format!(
            "API Error: {}",
            resp.status()
        )));
    }

    let result: PaginatedResponse<Penghapusan> = resp.json().await?;
    Ok(result)
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn fetch_penghapusan(
    _page: i32,
    _per_page: i32,
) -> Result<PaginatedResponse<Penghapusan>, String> {
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
pub async fn create_penghapusan(
    request: CreatePenghapusanRequest,
) -> Result<ApiResponse<Penghapusan>, gloo_net::Error> {
    use crate::components::auth::get_auth_token;
    use gloo_net::http::Request;

    let url = "/api/pembinaan/perlengkapan/penghapusan";
    let token = get_auth_token()
        .ok_or_else(|| gloo_net::Error::GlooError("No authentication token found".to_string()))?;

    let resp = Request::post(url)
        .header("Authorization", &format!("Bearer {}", token))
        .json(&request)?
        .send()
        .await?;

    if !resp.ok() {
        return Err(gloo_net::Error::GlooError(format!(
            "API Error: {}",
            resp.status()
        )));
    }

    let result: ApiResponse<Penghapusan> = resp.json().await?;
    Ok(result)
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn create_penghapusan(
    _request: CreatePenghapusanRequest,
) -> Result<ApiResponse<Penghapusan>, String> {
    Err("Server-side stub".to_string())
}

#[cfg(target_arch = "wasm32")]
pub async fn fetch_pengalihan(
    page: i32,
    per_page: i32,
) -> Result<PaginatedResponse<Pengalihan>, gloo_net::Error> {
    use crate::components::auth::get_auth_token;
    use gloo_net::http::Request;

    let url = format!(
        "/api/pembinaan/perlengkapan/pengalihan?page={}&per_page={}",
        page, per_page
    );

    let token = get_auth_token()
        .ok_or_else(|| gloo_net::Error::GlooError("No authentication token found".to_string()))?;
    let resp = Request::get(&url)
        .header("Authorization", &format!("Bearer {}", token))
        .send()
        .await?;

    if !resp.ok() {
        return Err(gloo_net::Error::GlooError(format!(
            "API Error: {}",
            resp.status()
        )));
    }

    let result: PaginatedResponse<Pengalihan> = resp.json().await?;
    Ok(result)
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn fetch_pengalihan(
    _page: i32,
    _per_page: i32,
) -> Result<PaginatedResponse<Pengalihan>, String> {
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
pub async fn create_pengalihan(
    request: CreatePengalihanRequest,
) -> Result<ApiResponse<Pengalihan>, gloo_net::Error> {
    use crate::components::auth::get_auth_token;
    use gloo_net::http::Request;

    let url = "/api/pembinaan/perlengkapan/pengalihan";
    let token = get_auth_token()
        .ok_or_else(|| gloo_net::Error::GlooError("No authentication token found".to_string()))?;

    let resp = Request::post(url)
        .header("Authorization", &format!("Bearer {}", token))
        .json(&request)?
        .send()
        .await?;

    if !resp.ok() {
        return Err(gloo_net::Error::GlooError(format!(
            "API Error: {}",
            resp.status()
        )));
    }

    let result: ApiResponse<Pengalihan> = resp.json().await?;
    Ok(result)
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn create_pengalihan(
    _request: CreatePengalihanRequest,
) -> Result<ApiResponse<Pengalihan>, String> {
    Err("Server-side stub".to_string())
}

#[cfg(target_arch = "wasm32")]
pub async fn fetch_pemeliharaan(
    page: i32,
    per_page: i32,
) -> Result<PaginatedResponse<Pemeliharaan>, gloo_net::Error> {
    use crate::components::auth::get_auth_token;
    use gloo_net::http::Request;

    let url = format!(
        "/api/pembinaan/perlengkapan/pemeliharaan?page={}&per_page={}",
        page, per_page
    );

    let token = get_auth_token()
        .ok_or_else(|| gloo_net::Error::GlooError("No authentication token found".to_string()))?;
    let resp = Request::get(&url)
        .header("Authorization", &format!("Bearer {}", token))
        .send()
        .await?;

    if !resp.ok() {
        return Err(gloo_net::Error::GlooError(format!(
            "API Error: {}",
            resp.status()
        )));
    }

    let result: PaginatedResponse<Pemeliharaan> = resp.json().await?;
    Ok(result)
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn fetch_pemeliharaan(
    _page: i32,
    _per_page: i32,
) -> Result<PaginatedResponse<Pemeliharaan>, String> {
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
pub async fn create_pemeliharaan(
    request: CreatePemeliharaanRequest,
) -> Result<ApiResponse<Pemeliharaan>, gloo_net::Error> {
    use crate::components::auth::get_auth_token;
    use gloo_net::http::Request;

    let url = "/api/pembinaan/perlengkapan/pemeliharaan";
    let token = get_auth_token()
        .ok_or_else(|| gloo_net::Error::GlooError("No authentication token found".to_string()))?;

    let resp = Request::post(url)
        .header("Authorization", &format!("Bearer {}", token))
        .json(&request)?
        .send()
        .await?;

    if !resp.ok() {
        return Err(gloo_net::Error::GlooError(format!(
            "API Error: {}",
            resp.status()
        )));
    }

    let result: ApiResponse<Pemeliharaan> = resp.json().await?;
    Ok(result)
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn create_pemeliharaan(
    _request: CreatePemeliharaanRequest,
) -> Result<ApiResponse<Pemeliharaan>, String> {
    Err("Server-side stub".to_string())
}

// ============ PAKAIAN DINAS (Official Uniform) Models ============

/// Gender enum for uniform sizing
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum Gender {
    #[serde(rename = "L")]
    Laki,
    #[serde(rename = "P")]
    Perempuan,
}

impl std::fmt::Display for Gender {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Gender::Laki => write!(f, "L"),
            Gender::Perempuan => write!(f, "P"),
        }
    }
}

/// Size group categories
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum UkuranGroup {
    Baju,
    Celana,
    Sepatu,
}

/// Activity status for pengajuan workflow
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum AktivitasStatus {
    Draft,
    Diajukan,
    VerifikasiKorwil,
    ApprovalKorwil,
    DisetujuiKorwil,
    TolakKorwil,
    VerifikasiPusat,
    ApprovalPusat,
    DisetujuiPusat,
    TolakPusat,
    Selesai,
}

impl std::fmt::Display for AktivitasStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AktivitasStatus::Draft => write!(f, "draft"),
            AktivitasStatus::Diajukan => write!(f, "diajukan"),
            AktivitasStatus::VerifikasiKorwil => write!(f, "verifikasi_korwil"),
            AktivitasStatus::ApprovalKorwil => write!(f, "approval_korwil"),
            AktivitasStatus::DisetujuiKorwil => write!(f, "disetujui_korwil"),
            AktivitasStatus::TolakKorwil => write!(f, "tolak_korwil"),
            AktivitasStatus::VerifikasiPusat => write!(f, "verifikasi_pusat"),
            AktivitasStatus::ApprovalPusat => write!(f, "approval_pusat"),
            AktivitasStatus::DisetujuiPusat => write!(f, "disetujui_pusat"),
            AktivitasStatus::TolakPusat => write!(f, "tolak_pusat"),
            AktivitasStatus::Selesai => write!(f, "selesai"),
        }
    }
}

/// Jenis Pakaian Dinas (Type of official uniform)
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct JenisPakaianDinas {
    pub id: String,
    pub nama: String,
    pub keterangan: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CreateJenisPakaianDinasRequest {
    pub nama: String,
    pub keterangan: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UpdateJenisPakaianDinasRequest {
    pub nama: String,
    pub keterangan: Option<String>,
}

/// Spesifikasi Pakaian Dinas (Uniform specification)
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct SpesifikasiPakaianDinas {
    pub id: String,
    pub jenis_pakaian_dinas_id: String,
    pub nama: String,
    pub keterangan: Option<String>,
    pub foto: Option<String>,
    pub jenis_pakaian_nama: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CreateSpesifikasiRequest {
    pub jenis_pakaian_dinas_id: String,
    pub nama: String,
    pub keterangan: Option<String>,
    pub foto: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UpdateSpesifikasiRequest {
    pub nama: String,
    pub keterangan: Option<String>,
    pub foto: Option<String>,
}

/// Sub-spesifikasi Pakaian Dinas
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct SubSpesifikasiPakaianDinas {
    pub id: String,
    pub spesifikasi_pakaian_dinas_id: String,
    pub nama: String,
    pub keterangan: Option<String>,
    pub spesifikasi_nama: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CreateSubSpesifikasiRequest {
    pub spesifikasi_pakaian_dinas_id: String,
    pub nama: String,
    pub keterangan: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UpdateSubSpesifikasiRequest {
    pub nama: String,
    pub keterangan: Option<String>,
}

/// Master Ukuran (Size master data)
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Ukuran {
    pub id: String,
    pub group: String,
    pub size: String,
    pub created_at: String,
    pub updated_at: String,
}

/// Pengajuan Pakaian Dinas (Uniform request/application)
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct PengajuanPakaianDinas {
    pub id: String,
    pub nama: String,
    pub tahun: i32,
    pub is_open: bool,
    pub tgl_open: Option<String>,
    pub tgl_close: Option<String>,
    pub status: String,
    pub keterangan: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CreatePengajuanPakaianDinasRequest {
    pub nama: String,
    pub tahun: i32,
    pub tgl_open: Option<String>,
    pub tgl_close: Option<String>,
    pub keterangan: Option<String>,
}

/// Pengajuan Satker (Work unit submission)
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct PengajuanSatker {
    pub id: String,
    pub pengajuan_pakaian_dinas_id: String,
    pub satker_id: String,
    pub satker_nama: Option<String>,
    pub status: String,
    pub jumlah_pegawai: i64,
    pub created_at: String,
    pub updated_at: String,
}

/// Pengajuan Satker with activities
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct PengajuanSatkerWithActivities {
    pub pengajuan_satker: PengajuanSatker,
    pub aktivitas: Vec<PengajuanAktivitas>,
}

/// Activity log for pengajuan workflow
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct PengajuanAktivitas {
    pub id: String,
    pub pengajuan_satker_id: String,
    pub user_id: String,
    pub user_nama: Option<String>,
    pub status: String,
    pub catatan: Option<String>,
    pub created_at: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ValidatorActionRequest {
    pub pengajuan_satker_id: String,
    pub action: String,
    pub catatan: Option<String>,
}

/// Pegawai Pakaian Dinas (Employee uniform sizes)
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct PegawaiPakaianDinas {
    pub id: String,
    pub pegawai_id: String,
    pub pegawai_nama: Option<String>,
    pub pegawai_nip: Option<String>,
    pub ukuran_baju: Option<String>,
    pub ukuran_celana: Option<String>,
    pub ukuran_sepatu: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UpsertPegawaiUkuranRequest {
    pub pegawai_id: String,
    pub ukuran_baju: Option<String>,
    pub ukuran_celana: Option<String>,
    pub ukuran_sepatu: Option<String>,
}

/// MySIMKARI Employee data (from integrasi)
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct MysimkariPegawai {
    pub id: String,
    pub nip: String,
    pub nama: String,
    pub satker_id: String,
    pub satker_nama: Option<String>,
    pub jabatan: Option<String>,
    pub pangkat: Option<String>,
    pub golongan: Option<String>,
    pub jenis_kelamin: Option<String>,
}

/// Employee with sizes for display
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct PegawaiWithSizes {
    pub pegawai: MysimkariPegawai,
    pub ukuran: Option<PegawaiPakaianDinas>,
}

/// Report: Rekap Ukuran (Size summary)
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct LaporanRekapUkuran {
    pub group: String,
    pub size: String,
    pub jumlah: i64,
    pub jenis_kelamin: Option<String>,
}

/// Report: Daftar Pegawai (Employee list)
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct LaporanDaftarPegawai {
    pub nip: String,
    pub nama: String,
    pub satker_nama: String,
    pub jabatan: Option<String>,
    pub jenis_kelamin: Option<String>,
    pub ukuran_baju: Option<String>,
    pub ukuran_celana: Option<String>,
    pub ukuran_sepatu: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LaporanQuery {
    pub pengajuan_id: Option<String>,
    pub satker_id: Option<String>,
    pub jenis_pakaian_id: Option<String>,
}

// ============ PAKAIAN DINAS API Functions ============

// --- Jenis Pakaian Dinas ---

#[cfg(target_arch = "wasm32")]
pub async fn fetch_jenis_pakaian_dinas(
    page: i32,
    per_page: i32,
) -> Result<PaginatedResponse<JenisPakaianDinas>, gloo_net::Error> {
    use crate::components::auth::get_auth_token;
    use gloo_net::http::Request;

    let url = format!(
        "/api/pembinaan/perlengkapan/pakaian-dinas/jenis?page={}&per_page={}",
        page, per_page
    );

    let token = get_auth_token()
        .ok_or_else(|| gloo_net::Error::GlooError("No authentication token found".to_string()))?;
    let resp = Request::get(&url)
        .header("Authorization", &format!("Bearer {}", token))
        .send()
        .await?;

    if !resp.ok() {
        return Err(gloo_net::Error::GlooError(format!(
            "API Error: {}",
            resp.status()
        )));
    }

    let result: PaginatedResponse<JenisPakaianDinas> = resp.json().await?;
    Ok(result)
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn fetch_jenis_pakaian_dinas(
    _page: i32,
    _per_page: i32,
) -> Result<PaginatedResponse<JenisPakaianDinas>, String> {
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
pub async fn create_jenis_pakaian_dinas(
    request: CreateJenisPakaianDinasRequest,
) -> Result<ApiResponse<JenisPakaianDinas>, gloo_net::Error> {
    use crate::components::auth::get_auth_token;
    use gloo_net::http::Request;

    let url = "/api/pembinaan/perlengkapan/pakaian-dinas/jenis";
    let token = get_auth_token()
        .ok_or_else(|| gloo_net::Error::GlooError("No authentication token found".to_string()))?;

    let resp = Request::post(url)
        .header("Authorization", &format!("Bearer {}", token))
        .json(&request)?
        .send()
        .await?;

    if !resp.ok() {
        return Err(gloo_net::Error::GlooError(format!(
            "API Error: {}",
            resp.status()
        )));
    }

    let result: ApiResponse<JenisPakaianDinas> = resp.json().await?;
    Ok(result)
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn create_jenis_pakaian_dinas(
    _request: CreateJenisPakaianDinasRequest,
) -> Result<ApiResponse<JenisPakaianDinas>, String> {
    Err("Server-side stub".to_string())
}

#[cfg(target_arch = "wasm32")]
pub async fn update_jenis_pakaian_dinas(
    id: String,
    request: UpdateJenisPakaianDinasRequest,
) -> Result<ApiResponse<JenisPakaianDinas>, gloo_net::Error> {
    use crate::components::auth::get_auth_token;
    use gloo_net::http::Request;

    let url = format!("/api/pembinaan/perlengkapan/pakaian-dinas/jenis/{}", id);
    let token = get_auth_token()
        .ok_or_else(|| gloo_net::Error::GlooError("No authentication token found".to_string()))?;

    let resp = Request::put(&url)
        .header("Authorization", &format!("Bearer {}", token))
        .json(&request)?
        .send()
        .await?;

    if !resp.ok() {
        return Err(gloo_net::Error::GlooError(format!(
            "API Error: {}",
            resp.status()
        )));
    }

    let result: ApiResponse<JenisPakaianDinas> = resp.json().await?;
    Ok(result)
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn update_jenis_pakaian_dinas(
    _id: String,
    _request: UpdateJenisPakaianDinasRequest,
) -> Result<ApiResponse<JenisPakaianDinas>, String> {
    Err("Server-side stub".to_string())
}

#[cfg(target_arch = "wasm32")]
pub async fn delete_jenis_pakaian_dinas(id: String) -> Result<ApiResponse<()>, gloo_net::Error> {
    use crate::components::auth::get_auth_token;
    use gloo_net::http::Request;

    let url = format!("/api/pembinaan/perlengkapan/pakaian-dinas/jenis/{}", id);
    let token = get_auth_token()
        .ok_or_else(|| gloo_net::Error::GlooError("No authentication token found".to_string()))?;

    let resp = Request::delete(&url)
        .header("Authorization", &format!("Bearer {}", token))
        .send()
        .await?;

    if !resp.ok() {
        return Err(gloo_net::Error::GlooError(format!(
            "API Error: {}",
            resp.status()
        )));
    }

    let result: ApiResponse<()> = resp.json().await?;
    Ok(result)
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn delete_jenis_pakaian_dinas(_id: String) -> Result<ApiResponse<()>, String> {
    Err("Server-side stub".to_string())
}

// --- Spesifikasi ---

#[cfg(target_arch = "wasm32")]
pub async fn fetch_spesifikasi_pakaian(
    page: i32,
    per_page: i32,
    jenis_id: Option<String>,
) -> Result<PaginatedResponse<SpesifikasiPakaianDinas>, gloo_net::Error> {
    use crate::components::auth::get_auth_token;
    use gloo_net::http::Request;

    let mut url = format!(
        "/api/pembinaan/perlengkapan/pakaian-dinas/spesifikasi?page={}&per_page={}",
        page, per_page
    );
    if let Some(jid) = jenis_id {
        url.push_str(&format!("&jenis_pakaian_dinas_id={}", jid));
    }

    let token = get_auth_token()
        .ok_or_else(|| gloo_net::Error::GlooError("No authentication token found".to_string()))?;
    let resp = Request::get(&url)
        .header("Authorization", &format!("Bearer {}", token))
        .send()
        .await?;

    if !resp.ok() {
        return Err(gloo_net::Error::GlooError(format!(
            "API Error: {}",
            resp.status()
        )));
    }

    let result: PaginatedResponse<SpesifikasiPakaianDinas> = resp.json().await?;
    Ok(result)
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn fetch_spesifikasi_pakaian(
    _page: i32,
    _per_page: i32,
    _jenis_id: Option<String>,
) -> Result<PaginatedResponse<SpesifikasiPakaianDinas>, String> {
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
pub async fn create_spesifikasi_pakaian(
    request: CreateSpesifikasiRequest,
) -> Result<ApiResponse<SpesifikasiPakaianDinas>, gloo_net::Error> {
    use crate::components::auth::get_auth_token;
    use gloo_net::http::Request;

    let url = "/api/pembinaan/perlengkapan/pakaian-dinas/spesifikasi";
    let token = get_auth_token()
        .ok_or_else(|| gloo_net::Error::GlooError("No authentication token found".to_string()))?;

    let resp = Request::post(url)
        .header("Authorization", &format!("Bearer {}", token))
        .json(&request)?
        .send()
        .await?;

    if !resp.ok() {
        return Err(gloo_net::Error::GlooError(format!(
            "API Error: {}",
            resp.status()
        )));
    }

    let result: ApiResponse<SpesifikasiPakaianDinas> = resp.json().await?;
    Ok(result)
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn create_spesifikasi_pakaian(
    _request: CreateSpesifikasiRequest,
) -> Result<ApiResponse<SpesifikasiPakaianDinas>, String> {
    Err("Server-side stub".to_string())
}

#[cfg(target_arch = "wasm32")]
pub async fn update_spesifikasi_pakaian(
    id: String,
    request: UpdateSpesifikasiRequest,
) -> Result<ApiResponse<SpesifikasiPakaianDinas>, gloo_net::Error> {
    use crate::components::auth::get_auth_token;
    use gloo_net::http::Request;

    let url = format!(
        "/api/pembinaan/perlengkapan/pakaian-dinas/spesifikasi/{}",
        id
    );
    let token = get_auth_token()
        .ok_or_else(|| gloo_net::Error::GlooError("No authentication token found".to_string()))?;

    let resp = Request::put(&url)
        .header("Authorization", &format!("Bearer {}", token))
        .json(&request)?
        .send()
        .await?;

    if !resp.ok() {
        return Err(gloo_net::Error::GlooError(format!(
            "API Error: {}",
            resp.status()
        )));
    }

    let result: ApiResponse<SpesifikasiPakaianDinas> = resp.json().await?;
    Ok(result)
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn update_spesifikasi_pakaian(
    _id: String,
    _request: UpdateSpesifikasiRequest,
) -> Result<ApiResponse<SpesifikasiPakaianDinas>, String> {
    Err("Server-side stub".to_string())
}

#[cfg(target_arch = "wasm32")]
pub async fn delete_spesifikasi_pakaian(id: String) -> Result<ApiResponse<()>, gloo_net::Error> {
    use crate::components::auth::get_auth_token;
    use gloo_net::http::Request;

    let url = format!(
        "/api/pembinaan/perlengkapan/pakaian-dinas/spesifikasi/{}",
        id
    );
    let token = get_auth_token()
        .ok_or_else(|| gloo_net::Error::GlooError("No authentication token found".to_string()))?;

    let resp = Request::delete(&url)
        .header("Authorization", &format!("Bearer {}", token))
        .send()
        .await?;

    if !resp.ok() {
        return Err(gloo_net::Error::GlooError(format!(
            "API Error: {}",
            resp.status()
        )));
    }

    let result: ApiResponse<()> = resp.json().await?;
    Ok(result)
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn delete_spesifikasi_pakaian(_id: String) -> Result<ApiResponse<()>, String> {
    Err("Server-side stub".to_string())
}

// --- Sub-Spesifikasi ---

#[cfg(target_arch = "wasm32")]
pub async fn fetch_subspesifikasi_pakaian(
    page: i32,
    per_page: i32,
    spesifikasi_id: Option<String>,
) -> Result<PaginatedResponse<SubSpesifikasiPakaianDinas>, gloo_net::Error> {
    use crate::components::auth::get_auth_token;
    use gloo_net::http::Request;

    let mut url = format!(
        "/api/pembinaan/perlengkapan/pakaian-dinas/subspesifikasi?page={}&per_page={}",
        page, per_page
    );
    if let Some(sid) = spesifikasi_id {
        url.push_str(&format!("&spesifikasi_pakaian_dinas_id={}", sid));
    }

    let token = get_auth_token()
        .ok_or_else(|| gloo_net::Error::GlooError("No authentication token found".to_string()))?;
    let resp = Request::get(&url)
        .header("Authorization", &format!("Bearer {}", token))
        .send()
        .await?;

    if !resp.ok() {
        return Err(gloo_net::Error::GlooError(format!(
            "API Error: {}",
            resp.status()
        )));
    }

    let result: PaginatedResponse<SubSpesifikasiPakaianDinas> = resp.json().await?;
    Ok(result)
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn fetch_subspesifikasi_pakaian(
    _page: i32,
    _per_page: i32,
    _spesifikasi_id: Option<String>,
) -> Result<PaginatedResponse<SubSpesifikasiPakaianDinas>, String> {
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
pub async fn create_subspesifikasi_pakaian(
    request: CreateSubSpesifikasiRequest,
) -> Result<ApiResponse<SubSpesifikasiPakaianDinas>, gloo_net::Error> {
    use crate::components::auth::get_auth_token;
    use gloo_net::http::Request;

    let url = "/api/pembinaan/perlengkapan/pakaian-dinas/subspesifikasi";
    let token = get_auth_token()
        .ok_or_else(|| gloo_net::Error::GlooError("No authentication token found".to_string()))?;

    let resp = Request::post(url)
        .header("Authorization", &format!("Bearer {}", token))
        .json(&request)?
        .send()
        .await?;

    if !resp.ok() {
        return Err(gloo_net::Error::GlooError(format!(
            "API Error: {}",
            resp.status()
        )));
    }

    let result: ApiResponse<SubSpesifikasiPakaianDinas> = resp.json().await?;
    Ok(result)
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn create_subspesifikasi_pakaian(
    _request: CreateSubSpesifikasiRequest,
) -> Result<ApiResponse<SubSpesifikasiPakaianDinas>, String> {
    Err("Server-side stub".to_string())
}

// --- Master Ukuran ---

#[cfg(target_arch = "wasm32")]
pub async fn fetch_master_ukuran(
    group: Option<String>,
) -> Result<ApiResponse<Vec<Ukuran>>, gloo_net::Error> {
    use crate::components::auth::get_auth_token;
    use gloo_net::http::Request;

    let mut url = "/api/pembinaan/perlengkapan/pakaian-dinas/ukuran".to_string();
    if let Some(g) = group {
        url.push_str(&format!("?group={}", g));
    }

    let token = get_auth_token()
        .ok_or_else(|| gloo_net::Error::GlooError("No authentication token found".to_string()))?;
    let resp = Request::get(&url)
        .header("Authorization", &format!("Bearer {}", token))
        .send()
        .await?;

    if !resp.ok() {
        return Err(gloo_net::Error::GlooError(format!(
            "API Error: {}",
            resp.status()
        )));
    }

    let result: ApiResponse<Vec<Ukuran>> = resp.json().await?;
    Ok(result)
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn fetch_master_ukuran(
    _group: Option<String>,
) -> Result<ApiResponse<Vec<Ukuran>>, String> {
    Ok(ApiResponse {
        success: true,
        data: vec![],
        message: "Server-side stub".to_string(),
    })
}

// --- Pengajuan Pakaian Dinas ---

#[cfg(target_arch = "wasm32")]
pub async fn fetch_pengajuan_pakaian_dinas(
    page: i32,
    per_page: i32,
    tahun: Option<i32>,
) -> Result<PaginatedResponse<PengajuanPakaianDinas>, gloo_net::Error> {
    use crate::components::auth::get_auth_token;
    use gloo_net::http::Request;

    let mut url = format!(
        "/api/pembinaan/perlengkapan/pakaian-dinas/pengajuan?page={}&per_page={}",
        page, per_page
    );
    if let Some(t) = tahun {
        url.push_str(&format!("&tahun={}", t));
    }

    let token = get_auth_token()
        .ok_or_else(|| gloo_net::Error::GlooError("No authentication token found".to_string()))?;
    let resp = Request::get(&url)
        .header("Authorization", &format!("Bearer {}", token))
        .send()
        .await?;

    if !resp.ok() {
        return Err(gloo_net::Error::GlooError(format!(
            "API Error: {}",
            resp.status()
        )));
    }

    let result: PaginatedResponse<PengajuanPakaianDinas> = resp.json().await?;
    Ok(result)
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn fetch_pengajuan_pakaian_dinas(
    _page: i32,
    _per_page: i32,
    _tahun: Option<i32>,
) -> Result<PaginatedResponse<PengajuanPakaianDinas>, String> {
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
pub async fn create_pengajuan_pakaian_dinas(
    request: CreatePengajuanPakaianDinasRequest,
) -> Result<ApiResponse<PengajuanPakaianDinas>, gloo_net::Error> {
    use crate::components::auth::get_auth_token;
    use gloo_net::http::Request;

    let url = "/api/pembinaan/perlengkapan/pakaian-dinas/pengajuan";
    let token = get_auth_token()
        .ok_or_else(|| gloo_net::Error::GlooError("No authentication token found".to_string()))?;

    let resp = Request::post(url)
        .header("Authorization", &format!("Bearer {}", token))
        .json(&request)?
        .send()
        .await?;

    if !resp.ok() {
        return Err(gloo_net::Error::GlooError(format!(
            "API Error: {}",
            resp.status()
        )));
    }

    let result: ApiResponse<PengajuanPakaianDinas> = resp.json().await?;
    Ok(result)
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn create_pengajuan_pakaian_dinas(
    _request: CreatePengajuanPakaianDinasRequest,
) -> Result<ApiResponse<PengajuanPakaianDinas>, String> {
    Err("Server-side stub".to_string())
}

#[cfg(target_arch = "wasm32")]
pub async fn delete_pengajuan_pakaian_dinas(
    id: String,
) -> Result<ApiResponse<()>, gloo_net::Error> {
    use crate::components::auth::get_auth_token;
    use gloo_net::http::Request;

    let url = format!("/api/pembinaan/perlengkapan/pakaian-dinas/pengajuan/{}", id);
    let token = get_auth_token()
        .ok_or_else(|| gloo_net::Error::GlooError("No authentication token found".to_string()))?;

    let resp = Request::delete(&url)
        .header("Authorization", &format!("Bearer {}", token))
        .send()
        .await?;

    if !resp.ok() {
        return Err(gloo_net::Error::GlooError(format!(
            "API Error: {}",
            resp.status()
        )));
    }

    let result: ApiResponse<()> = resp.json().await?;
    Ok(result)
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn delete_pengajuan_pakaian_dinas(_id: String) -> Result<ApiResponse<()>, String> {
    Err("Server-side stub".to_string())
}

// --- Pengajuan Satker ---

#[cfg(target_arch = "wasm32")]
pub async fn fetch_pengajuan_satker(
    pengajuan_id: String,
    page: i32,
    per_page: i32,
) -> Result<PaginatedResponse<PengajuanSatkerWithActivities>, gloo_net::Error> {
    use crate::components::auth::get_auth_token;
    use gloo_net::http::Request;

    let url = format!(
        "/api/pembinaan/perlengkapan/pakaian-dinas/satker?pengajuan_id={}&page={}&per_page={}",
        pengajuan_id, page, per_page
    );

    let token = get_auth_token()
        .ok_or_else(|| gloo_net::Error::GlooError("No authentication token found".to_string()))?;
    let resp = Request::get(&url)
        .header("Authorization", &format!("Bearer {}", token))
        .send()
        .await?;

    if !resp.ok() {
        return Err(gloo_net::Error::GlooError(format!(
            "API Error: {}",
            resp.status()
        )));
    }

    let result: PaginatedResponse<PengajuanSatkerWithActivities> = resp.json().await?;
    Ok(result)
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn fetch_pengajuan_satker(
    _pengajuan_id: String,
    _page: i32,
    _per_page: i32,
) -> Result<PaginatedResponse<PengajuanSatkerWithActivities>, String> {
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
pub async fn process_validator_action(
    request: ValidatorActionRequest,
) -> Result<ApiResponse<PengajuanSatker>, gloo_net::Error> {
    use crate::components::auth::get_auth_token;
    use gloo_net::http::Request;

    let url = "/api/pembinaan/perlengkapan/pakaian-dinas/validator-action";
    let token = get_auth_token()
        .ok_or_else(|| gloo_net::Error::GlooError("No authentication token found".to_string()))?;

    let resp = Request::post(url)
        .header("Authorization", &format!("Bearer {}", token))
        .json(&request)?
        .send()
        .await?;

    if !resp.ok() {
        return Err(gloo_net::Error::GlooError(format!(
            "API Error: {}",
            resp.status()
        )));
    }

    let result: ApiResponse<PengajuanSatker> = resp.json().await?;
    Ok(result)
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn process_validator_action(
    _request: ValidatorActionRequest,
) -> Result<ApiResponse<PengajuanSatker>, String> {
    Err("Server-side stub".to_string())
}

// --- Pegawai Ukuran ---

#[cfg(target_arch = "wasm32")]
pub async fn fetch_pegawai_ukuran(
    pegawai_id: String,
) -> Result<ApiResponse<Option<PegawaiPakaianDinas>>, gloo_net::Error> {
    use crate::components::auth::get_auth_token;
    use gloo_net::http::Request;

    let url = format!(
        "/api/pembinaan/perlengkapan/pakaian-dinas/ukuran-pakaian-pegawai/{}",
        pegawai_id
    );

    let token = get_auth_token()
        .ok_or_else(|| gloo_net::Error::GlooError("No authentication token found".to_string()))?;
    let resp = Request::get(&url)
        .header("Authorization", &format!("Bearer {}", token))
        .send()
        .await?;

    if !resp.ok() {
        return Err(gloo_net::Error::GlooError(format!(
            "API Error: {}",
            resp.status()
        )));
    }

    let result: ApiResponse<Option<PegawaiPakaianDinas>> = resp.json().await?;
    Ok(result)
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn fetch_pegawai_ukuran(
    _pegawai_id: String,
) -> Result<ApiResponse<Option<PegawaiPakaianDinas>>, String> {
    Ok(ApiResponse {
        success: true,
        data: None,
        message: "Server-side stub".to_string(),
    })
}

#[cfg(target_arch = "wasm32")]
pub async fn upsert_pegawai_ukuran(
    request: UpsertPegawaiUkuranRequest,
) -> Result<ApiResponse<PegawaiPakaianDinas>, gloo_net::Error> {
    use crate::components::auth::get_auth_token;
    use gloo_net::http::Request;

    let url = "/api/pembinaan/perlengkapan/pakaian-dinas/ukuran-pakaian-pegawai";
    let token = get_auth_token()
        .ok_or_else(|| gloo_net::Error::GlooError("No authentication token found".to_string()))?;

    let resp = Request::post(url)
        .header("Authorization", &format!("Bearer {}", token))
        .json(&request)?
        .send()
        .await?;

    if !resp.ok() {
        return Err(gloo_net::Error::GlooError(format!(
            "API Error: {}",
            resp.status()
        )));
    }

    let result: ApiResponse<PegawaiPakaianDinas> = resp.json().await?;
    Ok(result)
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn upsert_pegawai_ukuran(
    _request: UpsertPegawaiUkuranRequest,
) -> Result<ApiResponse<PegawaiPakaianDinas>, String> {
    Err("Server-side stub".to_string())
}

// --- Pegawai by Satker (from MySIMKARI) ---

#[cfg(target_arch = "wasm32")]
pub async fn fetch_pegawai_by_satker(
    satker_id: String,
    page: i32,
    per_page: i32,
) -> Result<PaginatedResponse<MysimkariPegawai>, gloo_net::Error> {
    use crate::components::auth::get_auth_token;
    use gloo_net::http::Request;

    let url = format!(
        "/api/pembinaan/perlengkapan/pakaian-dinas/pegawai-satker/{}?page={}&per_page={}",
        satker_id, page, per_page
    );

    let token = get_auth_token()
        .ok_or_else(|| gloo_net::Error::GlooError("No authentication token found".to_string()))?;
    let resp = Request::get(&url)
        .header("Authorization", &format!("Bearer {}", token))
        .send()
        .await?;

    if !resp.ok() {
        return Err(gloo_net::Error::GlooError(format!(
            "API Error: {}",
            resp.status()
        )));
    }

    let result: PaginatedResponse<MysimkariPegawai> = resp.json().await?;
    Ok(result)
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn fetch_pegawai_by_satker(
    _satker_id: String,
    _page: i32,
    _per_page: i32,
) -> Result<PaginatedResponse<MysimkariPegawai>, String> {
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
pub async fn fetch_pegawai_with_sizes(
    satker_id: String,
    page: i32,
    per_page: i32,
) -> Result<PaginatedResponse<PegawaiWithSizes>, gloo_net::Error> {
    use crate::components::auth::get_auth_token;
    use gloo_net::http::Request;

    let url = format!(
        "/api/pembinaan/perlengkapan/pakaian-dinas/pegawai-satker/{}/with-sizes?page={}&per_page={}",
        satker_id, page, per_page
    );

    let token = get_auth_token()
        .ok_or_else(|| gloo_net::Error::GlooError("No authentication token found".to_string()))?;
    let resp = Request::get(&url)
        .header("Authorization", &format!("Bearer {}", token))
        .send()
        .await?;

    if !resp.ok() {
        return Err(gloo_net::Error::GlooError(format!(
            "API Error: {}",
            resp.status()
        )));
    }

    let result: PaginatedResponse<PegawaiWithSizes> = resp.json().await?;
    Ok(result)
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn fetch_pegawai_with_sizes(
    _satker_id: String,
    _page: i32,
    _per_page: i32,
) -> Result<PaginatedResponse<PegawaiWithSizes>, String> {
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

// --- Laporan (Reports) ---

#[cfg(target_arch = "wasm32")]
pub async fn fetch_laporan_rekap_ukuran(
    query: LaporanQuery,
) -> Result<ApiResponse<Vec<LaporanRekapUkuran>>, gloo_net::Error> {
    use crate::components::auth::get_auth_token;
    use gloo_net::http::Request;

    let mut url = "/api/pembinaan/perlengkapan/pakaian-dinas/laporan/rekap-ukuran".to_string();
    let mut params = vec![];
    if let Some(ref pid) = query.pengajuan_id {
        params.push(format!("pengajuan_id={}", pid));
    }
    if let Some(ref sid) = query.satker_id {
        params.push(format!("satker_id={}", sid));
    }
    if let Some(ref jid) = query.jenis_pakaian_id {
        params.push(format!("jenis_pakaian_id={}", jid));
    }
    if !params.is_empty() {
        url.push_str(&format!("?{}", params.join("&")));
    }

    let token = get_auth_token()
        .ok_or_else(|| gloo_net::Error::GlooError("No authentication token found".to_string()))?;
    let resp = Request::get(&url)
        .header("Authorization", &format!("Bearer {}", token))
        .send()
        .await?;

    if !resp.ok() {
        return Err(gloo_net::Error::GlooError(format!(
            "API Error: {}",
            resp.status()
        )));
    }

    let result: ApiResponse<Vec<LaporanRekapUkuran>> = resp.json().await?;
    Ok(result)
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn fetch_laporan_rekap_ukuran(
    _query: LaporanQuery,
) -> Result<ApiResponse<Vec<LaporanRekapUkuran>>, String> {
    Ok(ApiResponse {
        success: true,
        data: vec![],
        message: "Server-side stub".to_string(),
    })
}

#[cfg(target_arch = "wasm32")]
pub async fn fetch_laporan_daftar_pegawai(
    query: LaporanQuery,
    page: i32,
    per_page: i32,
) -> Result<PaginatedResponse<LaporanDaftarPegawai>, gloo_net::Error> {
    use crate::components::auth::get_auth_token;
    use gloo_net::http::Request;

    let mut url = format!(
        "/api/pembinaan/perlengkapan/pakaian-dinas/laporan/daftar-pegawai?page={}&per_page={}",
        page, per_page
    );
    if let Some(ref pid) = query.pengajuan_id {
        url.push_str(&format!("&pengajuan_id={}", pid));
    }
    if let Some(ref sid) = query.satker_id {
        url.push_str(&format!("&satker_id={}", sid));
    }
    if let Some(ref jid) = query.jenis_pakaian_id {
        url.push_str(&format!("&jenis_pakaian_id={}", jid));
    }

    let token = get_auth_token()
        .ok_or_else(|| gloo_net::Error::GlooError("No authentication token found".to_string()))?;
    let resp = Request::get(&url)
        .header("Authorization", &format!("Bearer {}", token))
        .send()
        .await?;

    if !resp.ok() {
        return Err(gloo_net::Error::GlooError(format!(
            "API Error: {}",
            resp.status()
        )));
    }

    let result: PaginatedResponse<LaporanDaftarPegawai> = resp.json().await?;
    Ok(result)
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn fetch_laporan_daftar_pegawai(
    _query: LaporanQuery,
    _page: i32,
    _per_page: i32,
) -> Result<PaginatedResponse<LaporanDaftarPegawai>, String> {
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

// ============================================================================
// KEBUTUHAN BMN MODELS
// ============================================================================

/// Workflow status codes for BMN needs requests
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(i32)]
pub enum KebutuhanBmnStatus {
    Draft = 2000,
    InputBarang = 2001,
    SubmitSatker = 2002,
    RevisiSatker = 2003,
    AnalisisKelayakan = 2004,
    PenyusunanPrioritas = 2005,
    Approved = 2006,
    Rejected = 2007,
    Completed = 2008,
    Cancelled = 2009,
}

impl KebutuhanBmnStatus {
    pub fn from_code(code: i32) -> Option<Self> {
        match code {
            2000 => Some(Self::Draft),
            2001 => Some(Self::InputBarang),
            2002 => Some(Self::SubmitSatker),
            2003 => Some(Self::RevisiSatker),
            2004 => Some(Self::AnalisisKelayakan),
            2005 => Some(Self::PenyusunanPrioritas),
            2006 => Some(Self::Approved),
            2007 => Some(Self::Rejected),
            2008 => Some(Self::Completed),
            2009 => Some(Self::Cancelled),
            _ => None,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::Draft => "Draft",
            Self::InputBarang => "Input Barang",
            Self::SubmitSatker => "Diajukan ke Validator",
            Self::RevisiSatker => "Revisi Satker",
            Self::AnalisisKelayakan => "Analisis Kelayakan",
            Self::PenyusunanPrioritas => "Penyusunan Prioritas",
            Self::Approved => "Disetujui",
            Self::Rejected => "Ditolak",
            Self::Completed => "Selesai",
            Self::Cancelled => "Dibatalkan",
        }
    }

    pub fn badge_class(&self) -> &'static str {
        match self {
            Self::Draft => "bg-gray-100 text-gray-800",
            Self::InputBarang => "bg-blue-100 text-blue-800",
            Self::SubmitSatker => "bg-yellow-100 text-yellow-800",
            Self::RevisiSatker => "bg-orange-100 text-orange-800",
            Self::AnalisisKelayakan => "bg-purple-100 text-purple-800",
            Self::PenyusunanPrioritas => "bg-indigo-100 text-indigo-800",
            Self::Approved => "bg-green-100 text-green-800",
            Self::Rejected => "bg-red-100 text-red-800",
            Self::Completed => "bg-emerald-100 text-emerald-800",
            Self::Cancelled => "bg-slate-100 text-slate-800",
        }
    }
}

impl Default for KebutuhanBmnStatus {
    fn default() -> Self {
        Self::Draft
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PilihanSatker {
    Semua,
    Sebagian,
}

impl Default for PilihanSatker {
    fn default() -> Self {
        Self::Semua
    }
}

/// Main entity for BMN needs analysis request
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PengajuanKebutuhanBmn {
    pub id: String,
    pub nama: String,
    pub deskripsi: Option<String>,
    pub tahun: i32,
    pub tgl_mulai: String,
    pub tgl_selesai: String,
    pub pilihan_satker: PilihanSatker,
    pub id_jenis_asset: Value,
    pub is_appv_daskrimti: bool,
    pub status_kode: i32,
    pub created_by: Option<String>,
    pub updated_by: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub version: i32,
}

/// Asset type included in a BMN request
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PengajuanKebutuhanBmnAsset {
    pub id: String,
    pub pengajuan_id: String,
    pub kode_barang: Option<String>,
    pub nm_barang: Option<String>,
    pub ms_jenis_asset_id: Option<i32>,
    pub keterangan: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// Per-satker tracking for BMN request
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PengajuanKebutuhanBmnSatker {
    pub id: String,
    pub pengajuan_id: String,
    pub ms_satker_id: String,
    pub ms_satker_pusat_id: Option<String>,
    pub nm_satker: Option<String>,
    pub status_kode: i32,
    pub prioritas: i32,
    pub created_by: Option<String>,
    pub updated_by: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// Individual goods requested by a satker
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PengajuanKebutuhanBmnBarang {
    pub id: String,
    pub pengajuan_satker_id: String,
    pub nama: String,
    pub kode_barang: Option<String>,
    pub jumlah: i32,
    pub satuan: String,
    pub jml_setuju: i32,
    pub alasan: Option<String>,
    pub keterangan: Option<String>,
    pub prioritas: i32,
    pub skor: f64,
    pub file_pendukung: Value,
    pub existing_count: i32,
    pub existing_condition: Option<String>,
    pub created_by: Option<String>,
    pub updated_by: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// Workflow history audit trail
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PengajuanKebutuhanBmnAktivitas {
    pub id: String,
    pub pengajuan_satker_id: String,
    pub from_status_kode: Option<i32>,
    pub to_status_kode: i32,
    pub user_id: Option<String>,
    pub nip: Option<String>,
    pub nama: Option<String>,
    pub pangkat: Option<String>,
    pub jabatan: Option<String>,
    pub role: Option<String>,
    pub aksi: String,
    pub komentar: Option<String>,
    pub created_at: String,
}

/// Summary data for dashboard and lists
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct KebutuhanBmnSummary {
    pub id: String,
    pub nama: String,
    pub tahun: i32,
    pub status_kode: i32,
    pub status_nama: String,
    pub total_satker: i64,
    pub total_barang: i64,
    pub total_jumlah_diminta: i64,
    pub total_jumlah_disetujui: i64,
    pub created_at: String,
    pub updated_at: String,
}

/// Workflow transition info
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WorkflowTransitionInfo {
    pub status_kode: i32,
    pub status_nama: String,
    pub requires_comment: bool,
}

/// Response with pengajuan and related data
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PengajuanDetailResponse {
    #[serde(flatten)]
    pub pengajuan: PengajuanKebutuhanBmn,
    pub assets: Vec<PengajuanKebutuhanBmnAsset>,
    pub satkers: Vec<PengajuanKebutuhanBmnSatker>,
    pub allowed_transitions: Vec<WorkflowTransitionInfo>,
}

/// Response for satker with its goods
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SatkerWithBarangResponse {
    #[serde(flatten)]
    pub satker: PengajuanKebutuhanBmnSatker,
    pub barang_list: Vec<PengajuanKebutuhanBmnBarang>,
    pub total_barang: i64,
    pub total_jumlah: i64,
}

/// Response for feasibility analysis
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AnalisisKelayakanResponse {
    pub satker: PengajuanKebutuhanBmnSatker,
    pub barang_list: Vec<BarangWithExistingInventory>,
    pub summary: AnalisisSummary,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BarangWithExistingInventory {
    #[serde(flatten)]
    pub barang: PengajuanKebutuhanBmnBarang,
    pub existing_assets: Vec<ExistingAssetInfo>,
    pub gap: i32,
    pub recommendation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExistingAssetInfo {
    pub no_aset: String,
    pub nama_aset: String,
    pub kondisi: String,
    pub lokasi: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AnalisisSummary {
    pub total_diminta: i64,
    pub total_existing: i64,
    pub total_gap: i64,
    pub kelayakan_persen: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct KebutuhanBmnDashboardStats {
    pub total_pengajuan: i64,
    pub pengajuan_draft: i64,
    pub pengajuan_in_progress: i64,
    pub pengajuan_completed: i64,
    pub total_satker_terlibat: i64,
    pub total_barang_diminta: i64,
    pub total_barang_disetujui: i64,
    pub by_tahun: Vec<StatsByTahun>,
    pub by_status: Vec<StatsByStatus>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StatsByTahun {
    pub tahun: i32,
    pub total: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StatsByStatus {
    pub status_kode: i32,
    pub status_nama: String,
    pub total: i64,
}

// ============================================================================
// SIMAN Integration Types
// ============================================================================

/// Asset data from SIMAN (Sistem Informasi Manajemen Aset Negara)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SimanAsset {
    pub no_aset: String,
    pub nama_aset: String,
    pub nup: Option<String>,
    pub kondisi: String,
    pub tahun_perolehan: Option<i32>,
    pub nilai_perolehan: Option<f64>,
    pub nilai_buku: Option<f64>,
    pub lokasi: Option<String>,
    pub kategori: String,
    pub satker_id: String,
    pub metadata: Option<serde_json::Value>,
}

/// Summary of existing assets for a satker from SIMAN
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SatkerAssetSummary {
    pub satker_id: String,
    pub satker_name: Option<String>,
    pub total_assets: i64,
    pub total_value: f64,
    pub by_category: Vec<CategoryAssetCount>,
    pub by_condition: Vec<ConditionAssetCount>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CategoryAssetCount {
    pub category: String,
    pub count: i64,
    pub value: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ConditionAssetCount {
    pub condition: String,
    pub count: i64,
}

// Request DTOs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateKebutuhanBmnRequest {
    pub nama: String,
    pub deskripsi: Option<String>,
    pub tahun: i32,
    pub tgl_mulai: String,
    pub tgl_selesai: String,
    pub pilihan_satker: Option<String>,
    #[serde(default)]
    pub satker_ids: Vec<String>,
    #[serde(default)]
    pub asset_types: Vec<CreateAssetTypeRequest>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateAssetTypeRequest {
    pub kode_barang: Option<String>,
    pub nm_barang: Option<String>,
    pub ms_jenis_asset_id: Option<i32>,
    pub keterangan: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateKebutuhanBmnRequest {
    pub nama: Option<String>,
    pub deskripsi: Option<String>,
    pub tgl_mulai: Option<String>,
    pub tgl_selesai: Option<String>,
    pub pilihan_satker: Option<String>,
    pub version: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateKebutuhanBmnBarangRequest {
    pub nama: String,
    pub kode_barang: Option<String>,
    pub jumlah: i32,
    pub satuan: String,
    pub alasan: Option<String>,
    pub keterangan: Option<String>,
    #[serde(default)]
    pub file_pendukung: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateBarangApprovalRequest {
    pub jml_setuju: i32,
    pub keterangan: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetPrioritasRequest {
    pub items: Vec<PrioritasItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrioritasItem {
    pub barang_id: String,
    pub prioritas: i32,
    pub skor: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowTransitionRequest {
    pub target_status: i32,
    pub komentar: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct KebutuhanBmnQuery {
    pub tahun: Option<i32>,
    pub status_kode: Option<i32>,
    pub satker_id: Option<String>,
    pub search: Option<String>,
}

// ============================================================================
// KEBUTUHAN BMN API FUNCTIONS
// ============================================================================

const KEBUTUHAN_BMN_BASE: &str = "/api/pembinaan/perlengkapan/kebutuhan-bmn";

// --- Dashboard ---
#[cfg(target_arch = "wasm32")]
pub async fn fetch_kebutuhan_bmn_dashboard()
-> Result<ApiResponse<KebutuhanBmnDashboardStats>, gloo_net::Error> {
    use crate::components::auth::get_auth_token;
    use gloo_net::http::Request;

    let token = get_auth_token()
        .ok_or_else(|| gloo_net::Error::GlooError("No authentication token found".to_string()))?;

    let resp = Request::get(&format!("{}/dashboard", KEBUTUHAN_BMN_BASE))
        .header("Authorization", &format!("Bearer {}", token))
        .send()
        .await?;

    if !resp.ok() {
        return Err(gloo_net::Error::GlooError(format!(
            "API Error: {}",
            resp.status()
        )));
    }

    resp.json().await
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn fetch_kebutuhan_bmn_dashboard()
-> Result<ApiResponse<KebutuhanBmnDashboardStats>, String> {
    Ok(ApiResponse {
        success: true,
        data: KebutuhanBmnDashboardStats {
            total_pengajuan: 0,
            pengajuan_draft: 0,
            pengajuan_in_progress: 0,
            pengajuan_completed: 0,
            total_satker_terlibat: 0,
            total_barang_diminta: 0,
            total_barang_disetujui: 0,
            by_tahun: vec![],
            by_status: vec![],
        },
        message: "Server-side stub".to_string(),
    })
}

// --- Pengajuan CRUD ---
#[cfg(target_arch = "wasm32")]
pub async fn fetch_kebutuhan_bmn_list(
    query: KebutuhanBmnQuery,
    page: i32,
    per_page: i32,
) -> Result<PaginatedResponse<KebutuhanBmnSummary>, gloo_net::Error> {
    use crate::components::auth::get_auth_token;
    use gloo_net::http::Request;

    let mut url = format!(
        "{}/pengajuan?page={}&per_page={}",
        KEBUTUHAN_BMN_BASE, page, per_page
    );
    if let Some(tahun) = query.tahun {
        url.push_str(&format!("&tahun={}", tahun));
    }
    if let Some(status_kode) = query.status_kode {
        url.push_str(&format!("&status_kode={}", status_kode));
    }
    if let Some(ref satker_id) = query.satker_id {
        url.push_str(&format!("&satker_id={}", satker_id));
    }
    if let Some(ref search) = query.search {
        url.push_str(&format!("&search={}", search));
    }

    let token = get_auth_token()
        .ok_or_else(|| gloo_net::Error::GlooError("No authentication token found".to_string()))?;

    let resp = Request::get(&url)
        .header("Authorization", &format!("Bearer {}", token))
        .send()
        .await?;

    if !resp.ok() {
        return Err(gloo_net::Error::GlooError(format!(
            "API Error: {}",
            resp.status()
        )));
    }

    resp.json().await
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn fetch_kebutuhan_bmn_list(
    _query: KebutuhanBmnQuery,
    _page: i32,
    _per_page: i32,
) -> Result<PaginatedResponse<KebutuhanBmnSummary>, String> {
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
pub async fn fetch_kebutuhan_bmn_detail(
    id: &str,
) -> Result<ApiResponse<PengajuanDetailResponse>, gloo_net::Error> {
    use crate::components::auth::get_auth_token;
    use gloo_net::http::Request;

    let token = get_auth_token()
        .ok_or_else(|| gloo_net::Error::GlooError("No authentication token found".to_string()))?;

    let resp = Request::get(&format!("{}/pengajuan/{}", KEBUTUHAN_BMN_BASE, id))
        .header("Authorization", &format!("Bearer {}", token))
        .send()
        .await?;

    if !resp.ok() {
        return Err(gloo_net::Error::GlooError(format!(
            "API Error: {}",
            resp.status()
        )));
    }

    resp.json().await
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn fetch_kebutuhan_bmn_detail(
    _id: &str,
) -> Result<ApiResponse<PengajuanDetailResponse>, String> {
    Err("Server-side stub".to_string())
}

#[cfg(target_arch = "wasm32")]
pub async fn create_kebutuhan_bmn(
    request: CreateKebutuhanBmnRequest,
) -> Result<ApiResponse<PengajuanDetailResponse>, gloo_net::Error> {
    use crate::components::auth::get_auth_token;
    use gloo_net::http::Request;

    let token = get_auth_token()
        .ok_or_else(|| gloo_net::Error::GlooError("No authentication token found".to_string()))?;

    let resp = Request::post(&format!("{}/pengajuan", KEBUTUHAN_BMN_BASE))
        .header("Authorization", &format!("Bearer {}", token))
        .header("Content-Type", "application/json")
        .json(&request)?
        .send()
        .await?;

    if !resp.ok() {
        let err_text = resp.text().await.unwrap_or_default();
        return Err(gloo_net::Error::GlooError(format!(
            "API Error: {} - {}",
            resp.status(),
            err_text
        )));
    }

    resp.json().await
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn create_kebutuhan_bmn(
    _request: CreateKebutuhanBmnRequest,
) -> Result<ApiResponse<PengajuanDetailResponse>, String> {
    Err("Server-side stub".to_string())
}

#[cfg(target_arch = "wasm32")]
pub async fn update_kebutuhan_bmn(
    id: &str,
    request: UpdateKebutuhanBmnRequest,
) -> Result<ApiResponse<PengajuanDetailResponse>, gloo_net::Error> {
    use crate::components::auth::get_auth_token;
    use gloo_net::http::Request;

    let token = get_auth_token()
        .ok_or_else(|| gloo_net::Error::GlooError("No authentication token found".to_string()))?;

    let resp = Request::put(&format!("{}/pengajuan/{}", KEBUTUHAN_BMN_BASE, id))
        .header("Authorization", &format!("Bearer {}", token))
        .header("Content-Type", "application/json")
        .json(&request)?
        .send()
        .await?;

    if !resp.ok() {
        let err_text = resp.text().await.unwrap_or_default();
        return Err(gloo_net::Error::GlooError(format!(
            "API Error: {} - {}",
            resp.status(),
            err_text
        )));
    }

    resp.json().await
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn update_kebutuhan_bmn(
    _id: &str,
    _request: UpdateKebutuhanBmnRequest,
) -> Result<ApiResponse<PengajuanDetailResponse>, String> {
    Err("Server-side stub".to_string())
}

#[cfg(target_arch = "wasm32")]
pub async fn delete_kebutuhan_bmn(id: &str) -> Result<ApiResponse<()>, gloo_net::Error> {
    use crate::components::auth::get_auth_token;
    use gloo_net::http::Request;

    let token = get_auth_token()
        .ok_or_else(|| gloo_net::Error::GlooError("No authentication token found".to_string()))?;

    let resp = Request::delete(&format!("{}/pengajuan/{}", KEBUTUHAN_BMN_BASE, id))
        .header("Authorization", &format!("Bearer {}", token))
        .send()
        .await?;

    if !resp.ok() {
        return Err(gloo_net::Error::GlooError(format!(
            "API Error: {}",
            resp.status()
        )));
    }

    resp.json().await
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn delete_kebutuhan_bmn(_id: &str) -> Result<ApiResponse<()>, String> {
    Err("Server-side stub".to_string())
}

// --- Workflow Transition ---
#[cfg(target_arch = "wasm32")]
pub async fn transition_kebutuhan_bmn_status(
    id: &str,
    request: WorkflowTransitionRequest,
) -> Result<ApiResponse<PengajuanDetailResponse>, gloo_net::Error> {
    use crate::components::auth::get_auth_token;
    use gloo_net::http::Request;

    let token = get_auth_token()
        .ok_or_else(|| gloo_net::Error::GlooError("No authentication token found".to_string()))?;

    let resp = Request::post(&format!(
        "{}/pengajuan/{}/transition",
        KEBUTUHAN_BMN_BASE, id
    ))
    .header("Authorization", &format!("Bearer {}", token))
    .header("Content-Type", "application/json")
    .json(&request)?
    .send()
    .await?;

    if !resp.ok() {
        let err_text = resp.text().await.unwrap_or_default();
        return Err(gloo_net::Error::GlooError(format!(
            "API Error: {} - {}",
            resp.status(),
            err_text
        )));
    }

    resp.json().await
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn transition_kebutuhan_bmn_status(
    _id: &str,
    _request: WorkflowTransitionRequest,
) -> Result<ApiResponse<PengajuanDetailResponse>, String> {
    Err("Server-side stub".to_string())
}

// --- Satker Operations ---
#[cfg(target_arch = "wasm32")]
pub async fn fetch_pengajuan_satkers(
    pengajuan_id: &str,
    page: i32,
    per_page: i32,
) -> Result<PaginatedResponse<PengajuanKebutuhanBmnSatker>, gloo_net::Error> {
    use crate::components::auth::get_auth_token;
    use gloo_net::http::Request;

    let token = get_auth_token()
        .ok_or_else(|| gloo_net::Error::GlooError("No authentication token found".to_string()))?;

    let url = format!(
        "{}/pengajuan/{}/satker?page={}&per_page={}",
        KEBUTUHAN_BMN_BASE, pengajuan_id, page, per_page
    );

    let resp = Request::get(&url)
        .header("Authorization", &format!("Bearer {}", token))
        .send()
        .await?;

    if !resp.ok() {
        return Err(gloo_net::Error::GlooError(format!(
            "API Error: {}",
            resp.status()
        )));
    }

    resp.json().await
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn fetch_pengajuan_satkers(
    _pengajuan_id: &str,
    _page: i32,
    _per_page: i32,
) -> Result<PaginatedResponse<PengajuanKebutuhanBmnSatker>, String> {
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
pub async fn fetch_satker_with_barang(
    satker_id: &str,
) -> Result<ApiResponse<SatkerWithBarangResponse>, gloo_net::Error> {
    use crate::components::auth::get_auth_token;
    use gloo_net::http::Request;

    let token = get_auth_token()
        .ok_or_else(|| gloo_net::Error::GlooError("No authentication token found".to_string()))?;

    let resp = Request::get(&format!("{}/satker/{}", KEBUTUHAN_BMN_BASE, satker_id))
        .header("Authorization", &format!("Bearer {}", token))
        .send()
        .await?;

    if !resp.ok() {
        return Err(gloo_net::Error::GlooError(format!(
            "API Error: {}",
            resp.status()
        )));
    }

    resp.json().await
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn fetch_satker_with_barang(
    _satker_id: &str,
) -> Result<ApiResponse<SatkerWithBarangResponse>, String> {
    Err("Server-side stub".to_string())
}

#[cfg(target_arch = "wasm32")]
pub async fn transition_satker_status(
    satker_id: &str,
    request: WorkflowTransitionRequest,
) -> Result<ApiResponse<PengajuanKebutuhanBmnSatker>, gloo_net::Error> {
    use crate::components::auth::get_auth_token;
    use gloo_net::http::Request;

    let token = get_auth_token()
        .ok_or_else(|| gloo_net::Error::GlooError("No authentication token found".to_string()))?;

    let resp = Request::post(&format!(
        "{}/satker/{}/transition",
        KEBUTUHAN_BMN_BASE, satker_id
    ))
    .header("Authorization", &format!("Bearer {}", token))
    .header("Content-Type", "application/json")
    .json(&request)?
    .send()
    .await?;

    if !resp.ok() {
        let err_text = resp.text().await.unwrap_or_default();
        return Err(gloo_net::Error::GlooError(format!(
            "API Error: {} - {}",
            resp.status(),
            err_text
        )));
    }

    resp.json().await
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn transition_satker_status(
    _satker_id: &str,
    _request: WorkflowTransitionRequest,
) -> Result<ApiResponse<PengajuanKebutuhanBmnSatker>, String> {
    Err("Server-side stub".to_string())
}

#[cfg(target_arch = "wasm32")]
pub async fn fetch_satker_aktivitas(
    satker_id: &str,
) -> Result<ApiResponse<Vec<PengajuanKebutuhanBmnAktivitas>>, gloo_net::Error> {
    use crate::components::auth::get_auth_token;
    use gloo_net::http::Request;

    let token = get_auth_token()
        .ok_or_else(|| gloo_net::Error::GlooError("No authentication token found".to_string()))?;

    let resp = Request::get(&format!(
        "{}/satker/{}/aktivitas",
        KEBUTUHAN_BMN_BASE, satker_id
    ))
    .header("Authorization", &format!("Bearer {}", token))
    .send()
    .await?;

    if !resp.ok() {
        return Err(gloo_net::Error::GlooError(format!(
            "API Error: {}",
            resp.status()
        )));
    }

    resp.json().await
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn fetch_satker_aktivitas(
    _satker_id: &str,
) -> Result<ApiResponse<Vec<PengajuanKebutuhanBmnAktivitas>>, String> {
    Ok(ApiResponse {
        success: true,
        data: vec![],
        message: "Server-side stub".to_string(),
    })
}

#[cfg(target_arch = "wasm32")]
pub async fn fetch_satker_analisis(
    satker_id: &str,
) -> Result<ApiResponse<AnalisisKelayakanResponse>, gloo_net::Error> {
    use crate::components::auth::get_auth_token;
    use gloo_net::http::Request;

    let token = get_auth_token()
        .ok_or_else(|| gloo_net::Error::GlooError("No authentication token found".to_string()))?;

    let resp = Request::get(&format!(
        "{}/satker/{}/analisis",
        KEBUTUHAN_BMN_BASE, satker_id
    ))
    .header("Authorization", &format!("Bearer {}", token))
    .send()
    .await?;

    if !resp.ok() {
        return Err(gloo_net::Error::GlooError(format!(
            "API Error: {}",
            resp.status()
        )));
    }

    resp.json().await
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn fetch_satker_analisis(
    _satker_id: &str,
) -> Result<ApiResponse<AnalisisKelayakanResponse>, String> {
    Err("Server-side stub".to_string())
}

// --- Barang Operations ---
#[cfg(target_arch = "wasm32")]
pub async fn create_kebutuhan_bmn_barang(
    satker_id: &str,
    request: CreateKebutuhanBmnBarangRequest,
) -> Result<ApiResponse<PengajuanKebutuhanBmnBarang>, gloo_net::Error> {
    use crate::components::auth::get_auth_token;
    use gloo_net::http::Request;

    let token = get_auth_token()
        .ok_or_else(|| gloo_net::Error::GlooError("No authentication token found".to_string()))?;

    let resp = Request::post(&format!(
        "{}/satker/{}/barang",
        KEBUTUHAN_BMN_BASE, satker_id
    ))
    .header("Authorization", &format!("Bearer {}", token))
    .header("Content-Type", "application/json")
    .json(&request)?
    .send()
    .await?;

    if !resp.ok() {
        let err_text = resp.text().await.unwrap_or_default();
        return Err(gloo_net::Error::GlooError(format!(
            "API Error: {} - {}",
            resp.status(),
            err_text
        )));
    }

    resp.json().await
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn create_kebutuhan_bmn_barang(
    _satker_id: &str,
    _request: CreateKebutuhanBmnBarangRequest,
) -> Result<ApiResponse<PengajuanKebutuhanBmnBarang>, String> {
    Err("Server-side stub".to_string())
}

#[cfg(target_arch = "wasm32")]
pub async fn update_kebutuhan_bmn_barang(
    barang_id: &str,
    request: UpdateBarangApprovalRequest,
) -> Result<ApiResponse<PengajuanKebutuhanBmnBarang>, gloo_net::Error> {
    use crate::components::auth::get_auth_token;
    use gloo_net::http::Request;

    let token = get_auth_token()
        .ok_or_else(|| gloo_net::Error::GlooError("No authentication token found".to_string()))?;

    let resp = Request::put(&format!("{}/barang/{}", KEBUTUHAN_BMN_BASE, barang_id))
        .header("Authorization", &format!("Bearer {}", token))
        .header("Content-Type", "application/json")
        .json(&request)?
        .send()
        .await?;

    if !resp.ok() {
        let err_text = resp.text().await.unwrap_or_default();
        return Err(gloo_net::Error::GlooError(format!(
            "API Error: {} - {}",
            resp.status(),
            err_text
        )));
    }

    resp.json().await
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn update_kebutuhan_bmn_barang(
    _barang_id: &str,
    _request: UpdateBarangApprovalRequest,
) -> Result<ApiResponse<PengajuanKebutuhanBmnBarang>, String> {
    Err("Server-side stub".to_string())
}

#[cfg(target_arch = "wasm32")]
pub async fn delete_kebutuhan_bmn_barang(
    barang_id: &str,
) -> Result<ApiResponse<()>, gloo_net::Error> {
    use crate::components::auth::get_auth_token;
    use gloo_net::http::Request;

    let token = get_auth_token()
        .ok_or_else(|| gloo_net::Error::GlooError("No authentication token found".to_string()))?;

    let resp = Request::delete(&format!("{}/barang/{}", KEBUTUHAN_BMN_BASE, barang_id))
        .header("Authorization", &format!("Bearer {}", token))
        .send()
        .await?;

    if !resp.ok() {
        return Err(gloo_net::Error::GlooError(format!(
            "API Error: {}",
            resp.status()
        )));
    }

    resp.json().await
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn delete_kebutuhan_bmn_barang(_barang_id: &str) -> Result<ApiResponse<()>, String> {
    Err("Server-side stub".to_string())
}

// --- Priority Operations ---
#[cfg(target_arch = "wasm32")]
pub async fn set_kebutuhan_bmn_prioritas(
    request: SetPrioritasRequest,
) -> Result<ApiResponse<Vec<PengajuanKebutuhanBmnBarang>>, gloo_net::Error> {
    use crate::components::auth::get_auth_token;
    use gloo_net::http::Request;

    let token = get_auth_token()
        .ok_or_else(|| gloo_net::Error::GlooError("No authentication token found".to_string()))?;

    let resp = Request::post(&format!("{}/prioritas", KEBUTUHAN_BMN_BASE))
        .header("Authorization", &format!("Bearer {}", token))
        .header("Content-Type", "application/json")
        .json(&request)?
        .send()
        .await?;

    if !resp.ok() {
        let err_text = resp.text().await.unwrap_or_default();
        return Err(gloo_net::Error::GlooError(format!(
            "API Error: {} - {}",
            resp.status(),
            err_text
        )));
    }

    resp.json().await
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn set_kebutuhan_bmn_prioritas(
    _request: SetPrioritasRequest,
) -> Result<ApiResponse<Vec<PengajuanKebutuhanBmnBarang>>, String> {
    Err("Server-side stub".to_string())
}

// --- Export ---
#[cfg(target_arch = "wasm32")]
pub async fn export_kebutuhan_bmn(id: &str) -> Result<Vec<u8>, gloo_net::Error> {
    use crate::components::auth::get_auth_token;
    use gloo_net::http::Request;

    let token = get_auth_token()
        .ok_or_else(|| gloo_net::Error::GlooError("No authentication token found".to_string()))?;

    let resp = Request::get(&format!("{}/pengajuan/{}/export", KEBUTUHAN_BMN_BASE, id))
        .header("Authorization", &format!("Bearer {}", token))
        .send()
        .await?;

    if !resp.ok() {
        return Err(gloo_net::Error::GlooError(format!(
            "API Error: {}",
            resp.status()
        )));
    }

    resp.binary().await
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn export_kebutuhan_bmn(_id: &str) -> Result<Vec<u8>, String> {
    Err("Server-side stub".to_string())
}

// ============================================================================
// SIMAN Integration API
// ============================================================================

/// Search for existing assets from SIMAN
#[cfg(target_arch = "wasm32")]
pub async fn search_siman_assets(
    search: &str,
    kategori: Option<&str>,
    limit: Option<usize>,
) -> Result<ApiResponse<Vec<SimanAsset>>, gloo_net::Error> {
    use crate::components::auth::get_auth_token;
    use gloo_net::http::Request;

    let token = get_auth_token()
        .ok_or_else(|| gloo_net::Error::GlooError("No authentication token found".to_string()))?;

    let mut url = format!("{}/siman/search?search={}", KEBUTUHAN_BMN_BASE, search);
    if let Some(kat) = kategori {
        url.push_str(&format!("&kategori={}", kat));
    }
    if let Some(lim) = limit {
        url.push_str(&format!("&limit={}", lim));
    }

    let resp = Request::get(&url)
        .header("Authorization", &format!("Bearer {}", token))
        .send()
        .await?;

    if !resp.ok() {
        return Err(gloo_net::Error::GlooError(format!(
            "API Error: {}",
            resp.status()
        )));
    }

    resp.json().await
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn search_siman_assets(
    _search: &str,
    _kategori: Option<&str>,
    _limit: Option<usize>,
) -> Result<ApiResponse<Vec<SimanAsset>>, String> {
    Ok(ApiResponse {
        success: true,
        data: vec![],
        message: "Server-side stub".to_string(),
    })
}

/// Get SIMAN asset summary for a satker
#[cfg(target_arch = "wasm32")]
pub async fn fetch_siman_satker_summary(
    satker_id: &str,
) -> Result<ApiResponse<SatkerAssetSummary>, gloo_net::Error> {
    use crate::components::auth::get_auth_token;
    use gloo_net::http::Request;

    let token = get_auth_token()
        .ok_or_else(|| gloo_net::Error::GlooError("No authentication token found".to_string()))?;

    let resp = Request::get(&format!(
        "{}/siman/summary/{}",
        KEBUTUHAN_BMN_BASE, satker_id
    ))
    .header("Authorization", &format!("Bearer {}", token))
    .send()
    .await?;

    if !resp.ok() {
        return Err(gloo_net::Error::GlooError(format!(
            "API Error: {}",
            resp.status()
        )));
    }

    resp.json().await
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn fetch_siman_satker_summary(
    _satker_id: &str,
) -> Result<ApiResponse<SatkerAssetSummary>, String> {
    Ok(ApiResponse {
        success: true,
        data: SatkerAssetSummary {
            satker_id: String::new(),
            satker_name: None,
            total_assets: 0,
            total_value: 0.0,
            by_category: vec![],
            by_condition: vec![],
        },
        message: "Server-side stub".to_string(),
    })
}
