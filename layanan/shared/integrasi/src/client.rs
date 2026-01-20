use crate::config::Config;
use crate::error::MonsaktiError;
use crate::response::MonsaktiResponse;
use crate::siman::models::{SimanAssetCategory, SimanTokenResponse};
use reqwest::Client;
use serde::Deserialize;
use std::collections::HashMap;
use std::future::Future;
use std::path::Path;
use std::pin::Pin;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio_postgres::NoTls;
use tracing::{error, info, warn};

/// Klien utama untuk berinteraksi dengan API MonSAKTI
pub struct MonsaktiClient {
    /// Klien HTTP
    client: Client,
    /// Konfigurasi
    config: Config,
    /// Token saat ini (dapat diperbarui)
    current_tokens: HashMap<String, String>,
    /// Token SIMAN OAuth2 dan waktu expire
    siman_token: Option<(String, u64)>,
    /// Klien database opsional
    db_client: Option<tokio_postgres::Client>,
}

impl MonsaktiClient {
    /// Membuat klien MonSAKTI baru
    pub async fn new(config: Config) -> Result<Self, MonsaktiError> {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(120))
            .pool_max_idle_per_host(10)
            .cookie_store(true) // Enable cookie handling untuk PHPSESSID
            .user_agent("curl/8.5.0") // Match curl exactly
            .http1_only() // Force HTTP/1.1 like curl
            .build()?;

        let current_tokens = config.tokens.clone();

        let db_client = if let Some(db_url) = &config.db_config {
            let (client, connection) = tokio_postgres::connect(db_url, NoTls).await?;
            tokio::spawn(async move {
                if let Err(e) = connection.await {
                    error!("Error koneksi database: {}", e);
                }
            });

            // Set search_path ke integrasi schema
            if let Err(e) = client
                .execute("SET search_path TO integrasi, public", &[])
                .await
            {
                warn!("Gagal set search_path: {}", e);
            } else {
                info!("Database search_path set to: integrasi, public");
            }

            Some(client)
        } else {
            None
        };

        Ok(Self {
            client,
            config,
            current_tokens,
            siman_token: None,
            db_client,
        })
    }

    /// Clone untuk parallel processing - Token tidak di-share
    pub fn clone(&self) -> Self {
        Self {
            client: self.client.clone(),
            config: self.config.clone(),
            current_tokens: self.current_tokens.clone(),
            siman_token: self.siman_token.clone(),
            db_client: None, // DB client tidak di-clone untuk keamanan
        }
    }

    /// Fungsi fetch generik untuk semua endpoint dengan auto-retry pada token expired
    pub async fn fetch(
        &mut self,
        module: &str,
        tipe_data: &str,
        variables: Vec<String>,
    ) -> Result<MonsaktiResponse, MonsaktiError> {
        self.fetch_with_retry(module, tipe_data, variables, 1).await
    }

    /// Fungsi fetch dengan retry logic
    fn fetch_with_retry<'a>(
        &'a mut self,
        module: &'a str,
        tipe_data: &'a str,
        variables: Vec<String>,
        retry_count: u8,
    ) -> Pin<Box<dyn Future<Output = Result<MonsaktiResponse, MonsaktiError>> + Send + 'a>> {
        Box::pin(async move {
            let token = self.get_current_token(module)?;
            // Format URL sesuai dokumentasi: /API/MODULE/tipeData/variable1/variable2/...
            let mut url = format!("{}/API/{}/{}", self.config.base_url, module, tipe_data);
            for var in &variables {
                if !var.is_empty() {
                    url.push_str(&format!("/{}", var));
                }
            }

            if retry_count == 1 {
                info!("→ Fetching: {} (attempt {})", url, retry_count);
                info!(
                    "Token (8 karakter pertama): {}...",
                    &token.chars().take(8).collect::<String>()
                );
            } else {
                info!(
                    "⟳ Retry fetching: {} (attempt {} dengan token baru)",
                    url, retry_count
                );
                info!(
                    "Token baru (8 karakter pertama): {}...",
                    &token.chars().take(8).collect::<String>()
                );
            }

            // Try dengan ureq untuk data fetch juga
            info!("Using ureq for data fetch...");
            let url_clone = url.clone();
            let token_clone = token.clone();

            let ureq_result = tokio::task::spawn_blocking(move || {
                Self::fetch_with_ureq_static(&url_clone, &token_clone)
            })
            .await;

            match ureq_result {
                Ok(Ok(json_response)) => {
                    info!("✓ ureq data fetch SUCCESS!");

                    // MonSAKTI response format: [[{"TOKEN":"..."}], [data, data, ...]]
                    // Parse as array dan extract token + data
                    if let Some(arr) = json_response.as_array()
                        && arr.len() >= 2
                    {
                        // Element 0: token array
                        let mut new_token_opt = None;
                        if let Some(token_arr) = arr[0].as_array()
                            && let Some(token_obj) = token_arr.first()
                            && let Some(token_str) = token_obj.get("TOKEN").and_then(|t| t.as_str())
                        {
                            new_token_opt = Some(token_str.to_string());
                            info!("Memperbarui token untuk modul {}", module);
                            self.current_tokens
                                .insert(module.to_string(), token_str.to_string());

                            // Save token baru ke database
                            if let Some(db) = &self.db_client {
                                match self.save_token_to_db(db, module, token_str).await {
                                    Ok(_) => info!(
                                        "✓ Token dari response disimpan ke database (modul: {})",
                                        module
                                    ),
                                    Err(e) => warn!(
                                        "⚠ Gagal simpan token dari response ke database: {:?}",
                                        e
                                    ),
                                }
                            }
                        }

                        // Element 1: data array
                        let data_value = arr[1].clone();

                        return Ok(MonsaktiResponse {
                            new_token: new_token_opt,
                            data: Some(data_value),
                            error: None,
                        });
                    }

                    // Fallback: treat as plain JSON
                    return Ok(MonsaktiResponse {
                        new_token: None,
                        data: Some(json_response),
                        error: None,
                    });
                }
                Ok(Err(e)) => {
                    warn!(
                        "⚠ ureq data fetch failed: {}, trying reqwest fallback...",
                        e
                    );
                }
                Err(e) => {
                    warn!("⚠ ureq spawn failed: {:?}, trying reqwest fallback...", e);
                }
            }

            let response = self
                .client
                .get(&url)
                .header("Authorization", format!("Bearer {}", token))
                .send()
                .await?;

            if !response.status().is_success() {
                let status = response.status();
                let body = response.text().await.unwrap_or_default();
                error!("Request gagal: {} - {}", status, body);

                // Jika unauthorized atau forbidden, coba reset token
                if (status == 401 || status == 403) && retry_count == 1 {
                    warn!(
                        "⚠ Token kadaluarsa (HTTP {}), mencoba reset token dengan Bearer token dari .env...",
                        status
                    );
                    match self.reset_token_auto(module, tipe_data).await {
                        Ok(_new_token) => {
                            info!("✓ Token baru diterima dari resetToken endpoint");
                            info!(
                                "⟳ Retry request dengan token baru (attempt {})",
                                retry_count + 1
                            );
                            return self
                                .fetch_with_retry(module, tipe_data, variables, retry_count + 1)
                                .await;
                        }
                        Err(e) => {
                            error!("✗ Gagal reset token untuk modul {}: {:?}", module, e);
                            error!(
                                "⚠ Token di .env sudah tidak valid atau expired, perlu regenerasi manual dari portal Kemenkeu"
                            );
                        }
                    }
                }

                return Err(MonsaktiError::ApiError(format!(
                    "HTTP {}: {}",
                    status, body
                )));
            }

            let result: MonsaktiResponse = response.json().await?;

            if let Some(error_msg) = &result.error {
                if error_msg.contains("Token Expired")
                    || error_msg.contains("token") && error_msg.contains("expired")
                {
                    warn!(
                        "Token kadaluarsa untuk modul {} (attempt {})",
                        module, retry_count
                    );

                    // Auto-retry dengan reset token
                    if retry_count == 1 {
                        warn!(
                            "⚠ Response error: 'Token Expired', mencoba reset token dengan Bearer token dari .env..."
                        );
                        match self.reset_token_auto(module, tipe_data).await {
                            Ok(_new_token) => {
                                info!("✓ Token baru diterima dari resetToken endpoint");
                                info!(
                                    "⟳ Retry request dengan token baru (attempt {})",
                                    retry_count + 1
                                );
                                return self
                                    .fetch_with_retry(module, tipe_data, variables, retry_count + 1)
                                    .await;
                            }
                            Err(e) => {
                                error!("✗ Gagal reset token untuk modul {}: {:?}", module, e);
                                error!(
                                    "⚠ Token di .env sudah tidak valid atau expired, perlu regenerasi manual dari portal Kemenkeu"
                                );
                            }
                        }
                    }

                    return Err(MonsaktiError::TokenExpired);
                }
                return Err(MonsaktiError::ApiError(error_msg.clone()));
            }

            if let Some(new_token) = &result.new_token {
                info!("Memperbarui token untuk modul {}", module);
                self.current_tokens
                    .insert(module.to_string(), new_token.clone());

                // Save token baru ke database
                if let Some(db) = &self.db_client {
                    match self.save_token_to_db(db, module, new_token).await {
                        Ok(_) => info!(
                            "✓ Token dari response disimpan ke database (modul: {})",
                            module
                        ),
                        Err(e) => warn!("⚠ Gagal simpan token dari response ke database: {:?}", e),
                    }
                }
            }

            Ok(result)
        })
    }

    /// Fungsi fetch khusus untuk MySIMKARI API
    pub async fn fetch_mysimkari(
        &mut self,
        endpoint: &str,
        variables: Vec<String>,
    ) -> Result<MonsaktiResponse, MonsaktiError> {
        let token = self.get_current_token("MYSIMKARI")?;
        // Format URL: /base_url/endpoint/variable1/variable2/...
        let mut url = format!("{}/{}", self.config.mysimkari_base_url, endpoint);
        for var in &variables {
            if !var.is_empty() {
                url.push_str(&format!("/{}", var));
            }
        }

        info!("Fetching MySIMKARI: {}", url);

        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            error!("MySIMKARI request gagal: {} - {}", status, body);
            return Err(MonsaktiError::ApiError(format!(
                "HTTP {}: {}",
                status, body
            )));
        }

        let result: MonsaktiResponse = response.json().await?;

        if let Some(error_msg) = &result.error {
            return Err(MonsaktiError::ApiError(error_msg.clone()));
        }

        Ok(result)
    }

    /// Reset token yang kadaluarsa
    /// Endpoint reset token SELALU menggunakan literal "tipedata" bukan variable tipe_data
    /// Contoh: /resetToken/ADM/tipedata/KL006
    pub async fn reset_token(
        &mut self,
        module: &str,
        _tipe_data: &str, // Tidak dipakai, endpoint selalu pakai "tipedata" literal
        variable2: &str,
    ) -> Result<String, MonsaktiError> {
        // Hard-coded "tipedata" sesuai dokumentasi MonSAKTI
        let url = format!(
            "{}/resetToken/{}/tipedata/{}",
            self.config.base_url, module, variable2
        );

        // Get token dari .env untuk reset request
        let token = self.get_current_token(module)?;

        info!("→ Reset token request: {}", url);
        info!(
            "Authorization: Bearer {}...",
            &token.chars().take(20).collect::<String>()
        );
        info!("Token length: {} bytes", token.len());

        // Try dengan ureq (synchronous client) untuk comparison
        info!("Testing with ureq (synchronous HTTP client)...");
        let url_clone = url.clone();
        let token_clone = token.clone();

        let ureq_result = tokio::task::spawn_blocking(move || {
            Self::try_reset_with_ureq_static(&url_clone, &token_clone)
        })
        .await;

        match ureq_result {
            Ok(Ok(new_token)) => {
                info!("✓ ureq SUCCESS! Got token from ureq");
                info!(
                    "Token baru (20 karakter): {}...",
                    &new_token.chars().take(20).collect::<String>()
                );

                // Update cache
                self.current_tokens
                    .insert(module.to_string(), new_token.clone());

                // Save to database
                if let Some(db) = &self.db_client {
                    match self.save_token_to_db(db, module, &new_token).await {
                        Ok(_) => info!("✓ Token baru disimpan ke database (modul: {})", module),
                        Err(e) => warn!("⚠ Gagal simpan token ke database: {:?}", e),
                    }
                }

                return Ok(new_token);
            }
            Ok(Err(e)) => {
                warn!("⚠ ureq failed: {}, falling back to reqwest...", e);
            }
            Err(e) => {
                warn!("⚠ ureq spawn failed: {:?}, falling back to reqwest...", e);
            }
        }

        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", token))
            .header("Accept", "*/*") // Match curl exactly
            .send()
            .await?;

        info!("Response status: {}", response.status());

        // Log full response untuk debugging
        let response_text = response.text().await?;
        info!("Response body: {}", &response_text);

        // Parse response sebagai JSON
        let response_value: serde_json::Value = serde_json::from_str(&response_text)
            .map_err(|e| MonsaktiError::ApiError(format!("Failed to parse JSON: {}", e)))?;
        // Response format: [{"TOKEN":"eyJ0eXAi..."}]
        #[derive(Deserialize, Debug)]
        struct ResetTokenItem {
            #[serde(rename = "TOKEN")]
            token: String,
        }

        let result: Vec<ResetTokenItem> = serde_json::from_value(response_value)
            .map_err(|e| MonsaktiError::ApiError(format!("Failed to parse token array: {}", e)))?;

        if result.is_empty() {
            error!("Reset token response kosong");
            return Err(MonsaktiError::ApiError(
                "Reset token response kosong".to_string(),
            ));
        }

        let new_token = result[0].token.clone();

        // Update token internal cache dengan token baru
        self.current_tokens
            .insert(module.to_string(), new_token.clone());

        info!("✓ Token baru diterima untuk modul {}", module);
        info!(
            "Token baru (20 karakter pertama): {}...",
            &new_token.chars().take(20).collect::<String>()
        );

        // Simpan token baru ke database jika ada koneksi database
        if let Some(db) = &self.db_client {
            match self.save_token_to_db(db, module, &new_token).await {
                Ok(_) => info!("✓ Token baru disimpan ke database (modul: {})", module),
                Err(e) => warn!("⚠ Gagal simpan token ke database: {:?}", e),
            }
        }

        Ok(new_token)
    }

    /// Reset token otomatis dengan KL006 sebagai default
    async fn reset_token_auto(
        &mut self,
        module: &str,
        tipe_data: &str,
    ) -> Result<String, MonsaktiError> {
        // Default menggunakan KL006 untuk Kejaksaan RI
        self.reset_token(module, tipe_data, "KL006").await
    }

    /// Fetch data with ureq (synchronous client)
    fn fetch_with_ureq_static(url: &str, token: &str) -> Result<serde_json::Value, String> {
        let mut response = ureq::get(url)
            .header("Authorization", &format!("Bearer {}", token))
            .header("Accept", "*/*")
            .call()
            .map_err(|e| format!("ureq request error: {}", e))?;

        let status = response.status();
        tracing::info!("ureq data fetch response status: {}", status);

        if status != 200 {
            let body = response
                .body_mut()
                .read_to_string()
                .map_err(|e| format!("Read body error: {}", e))?;
            return Err(format!("ureq HTTP {}: {}", status, body));
        }

        let json: serde_json::Value = response
            .body_mut()
            .read_json()
            .map_err(|e| format!("JSON parse error: {}", e))?;

        Ok(json)
    }

    /// Try reset token with ureq (synchronous client) for debugging - static version
    fn try_reset_with_ureq_static(url: &str, token: &str) -> Result<String, String> {
        #[derive(serde::Deserialize)]
        struct TokenItem {
            #[serde(rename = "TOKEN")]
            token: String,
        }

        let mut response = ureq::get(url)
            .header("Authorization", &format!("Bearer {}", token))
            .header("Accept", "*/*")
            .call()
            .map_err(|e| format!("ureq request error: {}", e))?;

        let status = response.status();
        tracing::info!("ureq response status: {}", status);

        if status != 200 {
            let body = response
                .body_mut()
                .read_to_string()
                .map_err(|e| format!("Read body error: {}", e))?;
            return Err(format!("ureq HTTP {}: {}", status, body));
        }

        let tokens: Vec<TokenItem> = response
            .body_mut()
            .read_json()
            .map_err(|e| format!("JSON parse error: {}", e))?;

        if tokens.is_empty() {
            return Err("Empty token array".to_string());
        }

        Ok(tokens[0].token.clone())
    }

    /// Simpan token baru ke database
    async fn save_token_to_db(
        &self,
        db: &tokio_postgres::Client,
        module: &str,
        token: &str,
    ) -> Result<(), MonsaktiError> {
        // Hash token untuk verifikasi (SHA256)
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(token.as_bytes());
        let token_hash = format!("{:x}", hasher.finalize());

        // Upsert token ke tabel api_tokens
        let query = r#"
            INSERT INTO api_tokens (
                module,
                token_value,
                token_hash,
                is_active,
                is_expired,
                refreshed_at,
                updated_at
            )
            VALUES ($1, $2, $3, true, false, NOW(), NOW())
            ON CONFLICT (module)
            DO UPDATE SET
                token_value = $2,
                token_hash = $3,
                is_active = true,
                is_expired = false,
                refreshed_at = NOW(),
                updated_at = NOW()
        "#;

        db.execute(query, &[&module, &token, &token_hash]).await?;
        Ok(())
    }

    /// Simpan data ke file JSON
    pub async fn save_to_json<P: AsRef<Path>>(
        &self,
        data: &serde_json::Value,
        filename: P,
    ) -> Result<(), MonsaktiError> {
        let path = Path::new(&self.config.output_dir).join(filename);
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }
        let json = serde_json::to_string_pretty(data)?;
        tokio::fs::write(&path, json).await?;
        info!("Data disimpan ke: {}", path.display());
        Ok(())
    }

    /// Simpan data ke file CSV
    pub async fn save_to_csv<P: AsRef<Path>>(
        &self,
        data: &serde_json::Value,
        filename: P,
    ) -> Result<(), MonsaktiError> {
        let path = Path::new(&self.config.output_dir).join(filename);

        // Check data validity before spawning blocking task
        let array = match data.as_array() {
            Some(arr) if !arr.is_empty() => arr.clone(),
            Some(_) => {
                warn!("Tidak ada data untuk disimpan");
                return Ok(());
            }
            None => return Ok(()),
        };

        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        // Offload blocking I/O to a blocking thread
        tokio::task::spawn_blocking(move || -> Result<(), MonsaktiError> {
            let mut wtr = csv::Writer::from_path(&path)?;
            if let Some(first) = array.first()
                && let Some(obj) = first.as_object()
            {
                let headers: Vec<&String> = obj.keys().collect();
                wtr.write_record(&headers)?;

                for item in &array {
                    if let Some(obj) = item.as_object() {
                        let row: Vec<std::borrow::Cow<'_, [u8]>> = headers
                            .iter()
                            .map(|h| {
                                obj.get(*h)
                                    .map(|v| match v {
                                        serde_json::Value::String(s) => {
                                            std::borrow::Cow::Borrowed(s.as_bytes())
                                        }
                                        serde_json::Value::Number(n) => {
                                            std::borrow::Cow::Owned(n.to_string().into_bytes())
                                        }
                                        serde_json::Value::Bool(b) => {
                                            std::borrow::Cow::Owned(b.to_string().into_bytes())
                                        }
                                        serde_json::Value::Null => {
                                            std::borrow::Cow::Borrowed(&[] as &[u8])
                                        }
                                        _ => std::borrow::Cow::Owned(v.to_string().into_bytes()),
                                    })
                                    .unwrap_or(std::borrow::Cow::Borrowed(&[]))
                            })
                            .collect();
                        wtr.write_record(&row)?;
                    }
                }
            }
            wtr.flush()?;
            info!("CSV disimpan ke: {}", path.display());
            Ok(())
        })
        .await
        .map_err(|e| MonsaktiError::IoError(std::io::Error::other(e)))??;

        Ok(())
    }

    /// Simpan data ke database PostgreSQL
    pub async fn save_to_postgres(
        &self,
        table_name: &str,
        data: &serde_json::Value,
    ) -> Result<usize, MonsaktiError> {
        let db = self.db_client.as_ref().ok_or_else(|| {
            MonsaktiError::ConfigError("Database tidak dikonfigurasi".to_string())
        })?;

        if let Some(array) = data.as_array() {
            if array.is_empty() {
                warn!("Tidak ada data untuk diinsert ke tabel {}", table_name);
                return Ok(0);
            }
            let insert_count = crate::db::bulk_insert_postgres(db, table_name, array).await?;
            info!(
                "{} baris berhasil diinsert ke tabel {}",
                insert_count, table_name
            );
            Ok(insert_count)
        } else {
            warn!(
                "Data bukan array, tidak dapat diinsert ke tabel {}",
                table_name
            );
            Ok(0)
        }
    }

    /// Simpan data ke database dengan mapping modul dan endpoint
    pub async fn save_to_database(
        &self,
        module: &str,
        endpoint: &str,
        data: &serde_json::Value,
    ) -> Result<usize, MonsaktiError> {
        let db = self.db_client.as_ref().ok_or_else(|| {
            MonsaktiError::ConfigError("Database tidak dikonfigurasi".to_string())
        })?;

        crate::db::save_to_database(db, module, endpoint, data).await
    }

    fn get_current_token(&self, module: &str) -> Result<String, MonsaktiError> {
        self.current_tokens
            .get(module)
            .cloned()
            .or_else(|| self.config.tokens.get(module).cloned())
            .ok_or_else(|| MonsaktiError::ConfigError(format!("No token for module: {}", module)))
    }

    // === SIMAN API Methods ===

    /// Mendapatkan OAuth2 token dari SSO Kemenkeu untuk SIMAN API
    /// Token akan di-cache dan di-refresh otomatis saat expired
    async fn get_siman_token(&mut self) -> Result<String, MonsaktiError> {
        // Cek apakah token masih valid (dengan buffer 60 detik)
        if let Some((token, expires_at)) = &self.siman_token {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();
            if now + 60 < *expires_at {
                return Ok(token.clone());
            }
        }

        // Token tidak ada atau sudah expired, request token baru
        info!("Requesting new SIMAN OAuth2 token from SSO Kemenkeu");

        let client_id =
            self.config.siman_client_id.as_ref().ok_or_else(|| {
                MonsaktiError::ConfigError("SIMAN_CLIENT_ID not configured".into())
            })?;

        let client_secret = self.config.siman_client_secret.as_ref().ok_or_else(|| {
            MonsaktiError::ConfigError("SIMAN_CLIENT_SECRET not configured".into())
        })?;

        let params = [
            ("client_id", client_id.as_str()),
            ("client_secret", client_secret.as_str()),
            ("grant_type", "client_credentials"),
        ];

        let response = self
            .client
            .post(&self.config.siman_token_url)
            .form(&params)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            error!(
                "Failed to get SIMAN token from SSO Kemenkeu: {} - {}",
                status, body
            );
            return Err(MonsaktiError::ApiError(format!(
                "OAuth2 token request failed: HTTP {}",
                status
            )));
        }

        let token_response: SimanTokenResponse = response.json().await?;
        info!(
            "SIMAN token successfully obtained, expires in {} seconds",
            token_response.expires_in
        );

        // Simpan token dengan waktu expire
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let expires_at = now + token_response.expires_in;
        let token = token_response.access_token.clone();

        self.siman_token = Some((token.clone(), expires_at));

        Ok(token)
    }

    /// Mendapatkan jumlah baris untuk kategori aset SIMAN
    pub async fn fetch_siman_row_count(
        &mut self,
        category: SimanAssetCategory,
    ) -> Result<MonsaktiResponse, MonsaktiError> {
        let token = self.get_siman_token().await?;
        let ba_key = self
            .config
            .siman_ba_key
            .as_ref()
            .ok_or_else(|| MonsaktiError::ConfigError("SIMAN_BA_KEY not configured".into()))?;

        let url = format!(
            "{}/gateway/SLDKSimanKL/2.0/getRowCount/{}/{}",
            self.config.siman_base_url,
            ba_key,
            category.table_name()
        );

        info!("Fetching SIMAN row count: {}", url);

        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            error!("SIMAN getRowCount request failed: {} - {}", status, body);
            return Err(MonsaktiError::ApiError(format!(
                "HTTP {}: {}",
                status, body
            )));
        }

        // SIMAN mengembalikan structure yang berbeda, wrap dalam MonsaktiResponse
        let data: serde_json::Value = response.json().await?;

        Ok(MonsaktiResponse {
            new_token: None,
            data: Some(data),
            error: None,
        })
    }

    /// Mengambil data aset SIMAN dengan pagination
    pub async fn fetch_siman_data(
        &mut self,
        category: SimanAssetCategory,
        start_id: u32,
        end_id: u32,
    ) -> Result<MonsaktiResponse, MonsaktiError> {
        let token = self.get_siman_token().await?;
        let ba_key = self
            .config
            .siman_ba_key
            .as_ref()
            .ok_or_else(|| MonsaktiError::ConfigError("SIMAN_BA_KEY not configured".into()))?;

        let url = format!(
            "{}/gateway/SLDKSimanKL/2.0/{}",
            self.config.siman_base_url,
            category.endpoint()
        );

        info!(
            "Fetching SIMAN data: {} ({}-{})",
            category.description(),
            start_id,
            end_id
        );

        let params = [
            ("BA_KEY", ba_key.as_str()),
            ("ID_1", &start_id.to_string()),
            ("ID_2", &end_id.to_string()),
        ];

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", token))
            .form(&params)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            error!(
                "SIMAN {} request failed: {} - {}",
                category.endpoint(),
                status,
                body
            );
            return Err(MonsaktiError::ApiError(format!(
                "HTTP {}: {}",
                status, body
            )));
        }

        // Parse response
        let data: serde_json::Value = response.json().await?;

        // Cek apakah ada error dalam response
        if let Some(error_msg) = data.get("error").and_then(|e| e.as_str()) {
            return Err(MonsaktiError::ApiError(error_msg.to_string()));
        }

        Ok(MonsaktiResponse {
            new_token: None,
            data: Some(data),
            error: None,
        })
    }
}
