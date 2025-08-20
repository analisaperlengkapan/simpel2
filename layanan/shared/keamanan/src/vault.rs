use crate::error::AppError;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{error, info};

#[derive(Debug, Clone)]
pub struct VaultClient {
    client: Client,
    base_url: String,
    token: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VaultSecret {
    pub data: HashMap<String, String>,
    pub metadata: VaultMetadata,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VaultMetadata {
    pub created_time: String,
    pub version: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VaultResponse<T> {
    pub data: T,
}

impl VaultClient {
    pub fn new(base_url: &str, token: &str) -> Result<Self, AppError> {
        let client = Client::builder()
            .danger_accept_invalid_certs(true) // For development
            .build()?;

        Ok(Self {
            client,
            base_url: base_url.to_string(),
            token: token.to_string(),
        })
    }

    pub async fn get_secret(&self, path: &str) -> Result<HashMap<String, String>, AppError> {
        let url = format!("{}/v1/secret/data/{}", self.base_url, path);
        
        let response = self.client
            .get(&url)
            .header("X-Vault-Token", &self.token)
            .send()
            .await?;

        if response.status().is_success() {
            let vault_response: VaultResponse<VaultSecret> = response.json().await?;
            Ok(vault_response.data.data)
        } else {
            error!("Vault error: {}", response.status());
            Err(AppError::VaultError("Failed to retrieve secret".to_string()))
        }
    }

    pub async fn set_secret(&self, path: &str, data: HashMap<String, String>) -> Result<(), AppError> {
        let url = format!("{}/v1/secret/data/{}", self.base_url, path);
        
        let payload = VaultSecret {
            data,
            metadata: VaultMetadata {
                created_time: chrono::Utc::now().to_rfc3339(),
                version: 1,
            },
        };

        let response = self.client
            .post(&url)
            .header("X-Vault-Token", &self.token)
            .json(&payload)
            .send()
            .await?;

        if response.status().is_success() {
            info!("Secret stored successfully at path: {}", path);
            Ok(())
        } else {
            error!("Failed to store secret: {}", response.status());
            Err(AppError::VaultError("Failed to store secret".to_string()))
        }
    }

    pub async fn rotate_secret(&self, path: &str) -> Result<HashMap<String, String>, AppError> {
        // Generate new secret
        let new_secret = self.generate_secret()?;
        
        // Store new secret
        self.set_secret(path, new_secret.clone()).await?;
        
        info!("Secret rotated successfully at path: {}", path);
        Ok(new_secret)
    }

    pub async fn get_database_credentials(&self, role: &str) -> Result<DatabaseCredentials, AppError> {
        let url = format!("{}/v1/database/creds/{}", self.base_url, role);
        
        let response = self.client
            .get(&url)
            .header("X-Vault-Token", &self.token)
            .send()
            .await?;

        if response.status().is_success() {
            let credentials: VaultResponse<DatabaseCredentials> = response.json().await?;
            Ok(credentials.data)
        } else {
            error!("Failed to get database credentials: {}", response.status());
            Err(AppError::VaultError("Failed to get database credentials".to_string()))
        }
    }

    pub async fn health_check(&self) -> Result<bool, AppError> {
        let url = format!("{}/v1/sys/health", self.base_url);
        
        let response = self.client
            .get(&url)
            .send()
            .await?;

        Ok(response.status().is_success())
    }

    fn generate_secret(&self) -> Result<HashMap<String, String>, AppError> {
        use rand::Rng;
        
        let mut rng = rand::thread_rng();
        let secret: String = (0..32)
            .map(|_| rng.sample(rand::distributions::Alphanumeric) as char)
            .collect();

        let mut data = HashMap::new();
        data.insert("secret".to_string(), secret);
        
        Ok(data)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DatabaseCredentials {
    pub username: String,
    pub password: String,
    pub lease_id: String,
    pub lease_duration: i32,
    pub renewable: bool,
} 