pub mod secreton {
    pub mod v1 {
        tonic::include_proto!("secreton.v1");
    }
}
pub mod common {
    pub mod v1 {
        tonic::include_proto!("common.v1");
    }
}

use secreton::v1::secreton_service_client::SecretonServiceClient;
use secreton::v1::GetSecretRequest;
use tonic::transport::Channel;
use std::collections::HashMap;
use anyhow::Result;

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
