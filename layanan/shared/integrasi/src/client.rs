use crate::config::Config;
use crate::error::MonsaktiError;
use crate::response::MonsaktiResponse;
use reqwest::Client;
use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;
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
    /// Klien database opsional
    db_client: Option<tokio_postgres::Client>,
}

impl MonsaktiClient {
    /// Membuat klien MonSAKTI baru
    pub async fn new(config: Config) -> Result<Self, MonsaktiError> {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(120))
            .pool_max_idle_per_host(10)
            .use_rustls_tls()
            .build()?;

        let current_tokens = config.tokens.clone();

        let db_client = if let Some(db_url) = &config.db_config {
            let (client, connection) = tokio_postgres::connect(db_url, NoTls).await?;
            tokio::spawn(async move {
                if let Err(e) = connection.await {
                    error!("Error koneksi database: {}", e);
                }
            });
            Some(client)
        } else {
            None
        };

        Ok(Self { client, config, current_tokens, db_client })
    }

    /// Clone untuk parallel processing - Token tidak di-share
    pub fn clone(&self) -> Self {
        Self {
            client: self.client.clone(),
            config: self.config.clone(),
            current_tokens: self.current_tokens.clone(),
            db_client: None, // DB client tidak di-clone untuk keamanan
        }
    }

    /// Fungsi fetch generik untuk semua endpoint dengan auto-retry pada token expired
    pub async fn fetch(&mut self, module: &str, tipe_data: &str, variables: Vec<String>) -> Result<MonsaktiResponse, MonsaktiError> {
        self.fetch_with_retry(module, tipe_data, variables, 1).await
    }

    /// Fungsi fetch dengan retry logic
    async fn fetch_with_retry(&mut self, module: &str, tipe_data: &str, variables: Vec<String>, retry_count: u8) -> Result<MonsaktiResponse, MonsaktiError> {
        let token = self.get_current_token(module)?;
        // Format URL sesuai dokumentasi: /API/MODULE/tipeData/variable1/variable2/...
        let mut url = format!("{}/API/{}/{}", self.config.base_url, module, tipe_data);
        for var in &variables {
            if !var.is_empty() {
                url.push_str(&format!("/{}", var));
            }
        }

        info!("Fetching: {} (attempt {})", url, retry_count);

        let response = self.client.get(&url).header("Authorization", format!("Bearer {}", token)).send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            error!("Request gagal: {} - {}", status, body);

            // Jika unauthorized atau forbidden, coba reset token
            if (status == 401 || status == 403) && retry_count == 1 {
                warn!("Token mungkin kadaluarsa, mencoba reset token...");
                if let Ok(_) = self.reset_token_auto(module, tipe_data).await {
                    return self.fetch_with_retry(module, tipe_data, variables, retry_count + 1).await;
                }
            }

            return Err(MonsaktiError::ApiError(format!("HTTP {}: {}", status, body)));
        }

        let result: MonsaktiResponse = response.json().await?;

        if let Some(error_msg) = &result.error {
            if error_msg.contains("Token Expired") || error_msg.contains("token") && error_msg.contains("expired") {
                warn!("Token kadaluarsa untuk modul {} (attempt {})", module, retry_count);

                // Auto-retry dengan reset token
                if retry_count == 1 {
                    info!("Mencoba reset token dan retry...");
                    if let Ok(_) = self.reset_token_auto(module, tipe_data).await {
                        return self.fetch_with_retry(module, tipe_data, variables, retry_count + 1).await;
                    }
                }

                return Err(MonsaktiError::TokenExpired);
            }
            return Err(MonsaktiError::ApiError(error_msg.clone()));
        }

        if let Some(new_token) = &result.new_token {
            info!("Memperbarui token untuk modul {}", module);
            self.current_tokens.insert(module.to_string(), new_token.clone());
        }

        Ok(result)
    }

    /// Fungsi fetch khusus untuk MySIMKARI API
    pub async fn fetch_mysimkari(&mut self, endpoint: &str, variables: Vec<String>) -> Result<MonsaktiResponse, MonsaktiError> {
        let token = self.get_current_token("MYSIMKARI")?;
        // Format URL: /base_url/endpoint/variable1/variable2/...
        let mut url = format!("{}/{}", self.config.mysimkari_base_url, endpoint);
        for var in &variables {
            if !var.is_empty() {
                url.push_str(&format!("/{}", var));
            }
        }

        info!("Fetching MySIMKARI: {}", url);

        let response = self.client.get(&url).header("Authorization", format!("Bearer {}", token)).send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            error!("MySIMKARI request gagal: {} - {}", status, body);
            return Err(MonsaktiError::ApiError(format!("HTTP {}: {}", status, body)));
        }

        let result: MonsaktiResponse = response.json().await?;

        if let Some(error_msg) = &result.error {
            return Err(MonsaktiError::ApiError(error_msg.clone()));
        }

        Ok(result)
    }

    /// Reset token yang kadaluarsa
    pub async fn reset_token(&mut self, module: &str, tipe_data: &str, variable2: &str) -> Result<String, MonsaktiError> {
        let url = format!("{}/resetToken/{}/{}/{}", self.config.base_url, module, tipe_data, variable2);
        info!("Mereset token untuk modul {}", module);

        let response = self.client.get(&url).send().await?;
        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            error!("Gagal mereset token: {} - {}", status, body);
            return Err(MonsaktiError::ApiError(format!("Gagal mereset token: HTTP {}", status)));
        }

        #[derive(Deserialize)]
        struct ResetResponse {
            #[serde(rename = "newToken")]
            new_token: String,
        }

        let result: ResetResponse = response.json().await?;
        info!("Token berhasil direset untuk modul {}", module);
        self.current_tokens.insert(module.to_string(), result.new_token.clone());
        Ok(result.new_token)
    }

    /// Reset token otomatis dengan KL006 sebagai default
    async fn reset_token_auto(&mut self, module: &str, tipe_data: &str) -> Result<String, MonsaktiError> {
        // Default menggunakan KL006 untuk Kejaksaan RI
        self.reset_token(module, tipe_data, "KL006").await
    }

    /// Simpan data ke file JSON
    pub async fn save_to_json<P: AsRef<Path>>(&self, data: &serde_json::Value, filename: P) -> Result<(), MonsaktiError> {
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
    pub async fn save_to_csv<P: AsRef<Path>>(&self, data: &serde_json::Value, filename: P) -> Result<(), MonsaktiError> {
        let path = Path::new(&self.config.output_dir).join(filename);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        if let Some(array) = data.as_array() {
            if array.is_empty() {
                warn!("Tidak ada data untuk disimpan");
                return Ok(());
            }

            let mut wtr = csv::Writer::from_path(&path)?;
            if let Some(first) = array.first() {
                if let Some(obj) = first.as_object() {
                    let headers: Vec<&String> = obj.keys().collect();
                    wtr.write_record(&headers)?;

                    for item in array {
                        if let Some(obj) = item.as_object() {
                            let row: Vec<String> = headers.iter().map(|h| {
                                obj.get(*h).and_then(|v| match v {
                                    serde_json::Value::String(s) => Some(s.clone()),
                                    serde_json::Value::Number(n) => Some(n.to_string()),
                                    serde_json::Value::Bool(b) => Some(b.to_string()),
                                    serde_json::Value::Null => Some("".to_string()),
                                    _ => Some(v.to_string()),
                                }).unwrap_or_default()
                            }).collect();
                            wtr.write_record(&row)?;
                        }
                    }
                }
            }
            wtr.flush()?;
            info!("CSV disimpan ke: {}", path.display());
        }
        Ok(())
    }

    /// Simpan data ke database PostgreSQL
    pub async fn save_to_postgres(&self, table_name: &str, data: &serde_json::Value) -> Result<usize, MonsaktiError> {
        let db = self.db_client.as_ref()
            .ok_or_else(|| MonsaktiError::ConfigError("Database tidak dikonfigurasi".to_string()))?;

        if let Some(array) = data.as_array() {
            if array.is_empty() {
                warn!("Tidak ada data untuk diinsert ke tabel {}", table_name);
                return Ok(0);
            }
            let insert_count = crate::db::bulk_insert_postgres(db, table_name, array).await?;
            info!("{} baris berhasil diinsert ke tabel {}", insert_count, table_name);
            Ok(insert_count)
        } else {
            warn!("Data bukan array, tidak dapat diinsert ke tabel {}", table_name);
            Ok(0)
        }
    }

    /// Simpan data ke database dengan mapping modul dan endpoint
    pub async fn save_to_database(&self, module: &str, endpoint: &str, data: &serde_json::Value) -> Result<usize, MonsaktiError> {
        let db = self.db_client.as_ref()
            .ok_or_else(|| MonsaktiError::ConfigError("Database tidak dikonfigurasi".to_string()))?;

        crate::db::save_to_database(db, module, endpoint, data).await
    }

    fn get_current_token(&self, module: &str) -> Result<String, MonsaktiError> {
        self.current_tokens.get(module).cloned()
            .or_else(|| self.config.tokens.get(module).cloned())
            .ok_or_else(|| MonsaktiError::ConfigError(format!("No token for module: {}", module)))
    }
}
