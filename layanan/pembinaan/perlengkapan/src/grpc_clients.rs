use tonic::transport::Channel;
use std::collections::HashMap;
use anyhow::Result;

pub mod secreton {
    pub mod v1 {
        tonic::include_proto!("secreton.v1");
    }
}

pub mod authenc {
    pub mod v1 {
        tonic::include_proto!("authenc.v1");
    }
}

pub mod common {
    pub mod v1 {
        tonic::include_proto!("common.v1");
    }
}

use secreton::v1::secreton_service_client::SecretonServiceClient;
use secreton::v1::GetSecretRequest;

use authenc::v1::authenc_service_client::AuthencServiceClient;
use authenc::v1::ValidateTokenRequest;

#[derive(Clone)]
pub struct SecretonClient {
    client: SecretonServiceClient<Channel>,
}

impl SecretonClient {
    pub async fn connect(addr: String) -> Result<Self> {
        let client = SecretonServiceClient::connect(addr).await?;
        Ok(Self { client })
    }

    pub async fn get_secret(&self, path: &str) -> Result<HashMap<String, String>> {
        let mut client = self.client.clone();
        let request = tonic::Request::new(GetSecretRequest {
            path: path.to_string(),
            version: None,
        });

        let response = client.get_secret(request).await?;
        Ok(response.into_inner().data)
    }
}

#[derive(Clone)]
pub struct AuthencClient {
    client: AuthencServiceClient<Channel>,
}

impl AuthencClient {
    pub async fn connect(addr: String) -> Result<Self> {
        let client = AuthencServiceClient::connect(addr).await?;
        Ok(Self { client })
    }

    pub async fn validate_token(&self, token: &str) -> Result<authenc::v1::ValidateTokenResponse> {
        let mut client = self.client.clone();
        let request = tonic::Request::new(ValidateTokenRequest {
            token: token.to_string(),
            required_scopes: vec![],
        });

        let response = client.validate_token(request).await?;
        Ok(response.into_inner())
    }
}
