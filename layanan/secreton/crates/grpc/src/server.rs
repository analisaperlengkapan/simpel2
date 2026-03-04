//! gRPC Server Implementation
//!
//! Implements the SecretonService gRPC interface defined in secreton.proto

use async_trait::async_trait;
#[cfg(feature = "metrics")]
use metrics::{counter, gauge};
use std::net::SocketAddr;
use std::sync::Arc;
use tonic::{Request, Response, Status, transport::Server};
#[cfg(feature = "raft-consensus")]
use tracing::error;
use tracing::{info, instrument, warn};

use secreton_crypto::transit::TransitEngine;
use secreton_storage::StorageBackend;

use crate::generated::common::v1::*;
use crate::generated::secreton::v1::secreton_service_server;
use crate::generated::secreton::v1::*;
use crate::tls::GrpcTlsConfig;
use secreton_core::services::container::ServiceContainer;

// Snapshot types are imported from generated protos

/// gRPC service implementation
#[derive(Clone)]
pub struct SecretonGrpcService {
    /// Storage backend
    storage: Arc<dyn StorageBackend>,
    /// Transit encryption engine
    transit: Arc<TransitEngine>,
    /// gRPC request counter
    request_counter: Option<Arc<std::sync::atomic::AtomicU64>>,
    /// Service container for namespace and other services
    services: Arc<ServiceContainer>,
}

impl SecretonGrpcService {
    /// Create a new gRPC service with full service container
    pub fn new(
        storage: Arc<dyn StorageBackend>,
        transit: Arc<TransitEngine>,
        request_counter: Option<Arc<std::sync::atomic::AtomicU64>>,
        services: Arc<ServiceContainer>,
    ) -> Self {
        Self {
            storage,
            transit,
            request_counter,
            services,
        }
    }

    fn increment_request_count(&self) {
        if let Some(counter) = &self.request_counter {
            counter.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        }
    }

    /// Start the gRPC server without TLS
    pub async fn serve(self, addr: SocketAddr) -> Result<(), Box<dyn std::error::Error>> {
        info!("Starting gRPC server on {} (without TLS)", addr);
        warn!("Running gRPC server without TLS is not recommended for production");

        Server::builder()
            .add_service(secreton_service_server::SecretonServiceServer::new(self))
            .serve(addr)
            .await?;

        Ok(())
    }

    /// Start the gRPC server with TLS
    pub async fn serve_with_tls(
        self,
        addr: SocketAddr,
        tls_config: GrpcTlsConfig,
    ) -> Result<(), Box<dyn std::error::Error>> {
        use tonic::transport::{Identity, ServerTlsConfig};

        info!("Starting gRPC server on {} with mTLS", addr);

        // Validate TLS configuration
        tls_config
            .validate()
            .map_err(|e| format!("TLS configuration validation failed: {}", e))?;

        // Load TLS configuration
        let tls_identity = tls_config.load().await?;

        info!("TLS configuration loaded successfully");

        // Create server identity
        let identity = Identity::from_pem(&tls_identity.cert, &tls_identity.key);

        // Build TLS config
        let mut server_tls_config = ServerTlsConfig::new().identity(identity);

        // Add client CA if mTLS is required
        if let Some(ca_cert) = tls_identity.ca_cert {
            use tonic::transport::Certificate;
            let ca = Certificate::from_pem(ca_cert);
            server_tls_config = server_tls_config.client_ca_root(ca);
            info!("Client certificate authentication is REQUIRED");
        } else {
            info!("Client certificate authentication is OPTIONAL");
        }

        // Build and start server with TLS
        Server::builder()
            .tls_config(server_tls_config)?
            .add_service(secreton_service_server::SecretonServiceServer::new(self))
            .serve(addr)
            .await?;

        Ok(())
    }
}

#[async_trait]
impl secreton_service_server::SecretonService for SecretonGrpcService {
    #[instrument(skip(self, request))]
    async fn store_secret(
        &self,
        request: tonic::Request<StoreSecretRequest>,
    ) -> std::result::Result<tonic::Response<StoreSecretResponse>, tonic::Status> {
        self.increment_request_count();
        // Extract owner from request metadata before consuming request
        let owner_id = request
            .metadata()
            .get("user-id")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("system")
            .to_string();

        let req = request.into_inner();

        info!("Storing secret at path: {}", req.path);

        // Convert security level from proto enum to storage enum
        let security_level = match req.security_level {
            0 => secreton_storage::SecurityLevel::Public,
            1 => secreton_storage::SecurityLevel::Internal,
            2 => secreton_storage::SecurityLevel::Confidential,
            3 => secreton_storage::SecurityLevel::Secret,
            4 => secreton_storage::SecurityLevel::TopSecret,
            _ => return Err(Status::invalid_argument("Invalid security level")),
        };

        // Create engine entry for storage
        let secret_id = uuid::Uuid::new_v4();
        let encrypted_metadata = serde_json::json!({
            "algorithm": "aes-256-gcm",
            "key_id": "default-key",
        });

        // Serialize secret data
        let secret_data = serde_json::to_vec(&req.data)
            .map_err(|e| Status::internal(format!("Failed to serialize secret: {}", e)))?;

        // Create engine entry
        let mut engine_entry = secreton_storage::SecretEntry::new(
            req.path.clone(),
            secret_data,
            encrypted_metadata,
            security_level,
            owner_id,
        );

        // Add tags if provided
        for tag in req.tags {
            engine_entry = engine_entry.add_tag(tag);
        }

        // Set TTL if provided
        if let Some(ttl_seconds) = req.ttl_seconds {
            let expires_at = chrono::Utc::now() + chrono::Duration::seconds(ttl_seconds);
            engine_entry = engine_entry.with_expiration(expires_at);
        }

        let version = engine_entry.version;
        let created_at = engine_entry.created_at.timestamp();

        // Store in backend
        self.storage
            .store(&engine_entry)
            .await
            .map_err(|e| Status::internal(format!("Failed to store secret: {}", e)))?;

        info!(
            "Successfully stored secret at path: {} with ID: {}",
            req.path, secret_id
        );

        let response = StoreSecretResponse {
            id: secret_id.to_string(),
            version,
            created_at,
        };

        Ok(Response::new(response))
    }

    #[instrument(skip(self, request))]
    async fn get_secret(
        &self,
        request: Request<GetSecretRequest>,
    ) -> Result<Response<GetSecretResponse>, Status> {
        self.increment_request_count();
        let req = request.into_inner();

        info!("Getting secret at path: {}", req.path);

        // Retrieve from storage backend
        let engine_entry = self
            .storage
            .get_by_path(&req.path)
            .await
            .map_err(|e| Status::internal(format!("Failed to retrieve secret: {}", e)))?
            .ok_or_else(|| Status::not_found(format!("Secret not found at path: {}", req.path)))?;

        // Check if expired
        if engine_entry.is_expired() {
            return Err(Status::failed_precondition("Secret has expired"));
        }

        // Deserialize secret data
        let secret_data: std::collections::HashMap<String, String> =
            serde_json::from_slice(&engine_entry.encrypted_data)
                .map_err(|e| Status::internal(format!("Failed to deserialize secret: {}", e)))?;

        info!("Successfully retrieved secret at path: {}", req.path);

        let response = GetSecretResponse {
            id: engine_entry.id.to_string(),
            path: engine_entry.path,
            data: secret_data,
            version: engine_entry.version,
            security_level: engine_entry.security_level as i32,
            created_at: engine_entry.created_at.timestamp(),
            updated_at: engine_entry.updated_at.timestamp(),
        };

        Ok(Response::new(response))
    }

    #[instrument(skip(self, request))]
    async fn delete_secret(
        &self,
        request: Request<DeleteSecretRequest>,
    ) -> Result<Response<DeleteSecretResponse>, Status> {
        self.increment_request_count();
        let req = request.into_inner();

        info!("Deleting secret at path: {}", req.path);

        // Delete from storage backend
        let deleted = self
            .storage
            .delete_by_path(&req.path)
            .await
            .map_err(|e| Status::internal(format!("Failed to delete secret: {}", e)))?;

        if !deleted {
            return Err(Status::not_found(format!(
                "Secret not found at path: {}",
                req.path
            )));
        }

        info!("Successfully deleted secret at path: {}", req.path);

        let response = DeleteSecretResponse { success: true };

        Ok(Response::new(response))
    }

    #[instrument(skip(self, request))]
    async fn list_secrets(
        &self,
        request: Request<ListSecretsRequest>,
    ) -> Result<Response<ListSecretsResponse>, Status> {
        self.increment_request_count();
        let req = request.into_inner();

        info!("Listing secrets with prefix: {:?}", req.prefix);

        // Build query parameters
        let mut params = secreton_storage::QueryParams::new();
        if let Some(prefix) = req.prefix {
            params = params.with_path_prefix(prefix);
        }
        if let Some(limit) = req.limit {
            params = params.with_limit(limit as u32);
        }

        // List from storage backend
        let engine_entries = self
            .storage
            .list(&params)
            .await
            .map_err(|e| Status::internal(format!("Failed to list secrets: {}", e)))?;

        // Convert to gRPC response format
        let secrets: Vec<SecretMetadata> = engine_entries
            .into_iter()
            .map(|entry| SecretMetadata {
                id: entry.id.to_string(),
                path: entry.path,
                version: entry.version,
                security_level: entry.security_level as i32,
                created_at: entry.created_at.timestamp(),
            })
            .collect();

        let total = secrets.len() as i32;

        info!("Successfully listed {} secrets", total);

        let response = ListSecretsResponse { secrets, total };

        Ok(Response::new(response))
    }

    #[instrument(skip(self, request))]
    async fn create_key(
        &self,
        request: Request<CreateKeyRequest>,
    ) -> Result<Response<CreateKeyResponse>, Status> {
        self.increment_request_count();
        let req = request.into_inner();

        info!("Creating transit key: {}", req.name);

        let proto_key_type = KeyType::try_from(req.key_type)
            .map_err(|_| Status::invalid_argument("Invalid key type"))?;

        // Convert proto KeyType to crypto KeyType
        use secreton_crypto::transit::KeyType as CryptoKeyType;
        let crypto_key_type = match proto_key_type {
            KeyType::Aes256Gcm => CryptoKeyType::Aes256Gcm,
            KeyType::Chacha20Poly1305 => CryptoKeyType::ChaCha20Poly1305,
            KeyType::Ed25519 => CryptoKeyType::Ed25519,
            KeyType::EcdsaP256 => CryptoKeyType::EcdsaP256,
            _ => return Err(Status::invalid_argument("Unsupported key type")),
        };

        self.transit
            .create_key(req.name.clone(), crypto_key_type, None)
            .await
            .map_err(|e| Status::internal(format!("Failed to create key: {}", e)))?;

        let response = CreateKeyResponse {
            name: req.name,
            key_type: req.key_type,
            created_at: chrono::Utc::now().timestamp(),
        };

        Ok(Response::new(response))
    }

    #[instrument(skip(self, request))]
    async fn encrypt(
        &self,
        request: Request<EncryptRequest>,
    ) -> Result<Response<EncryptResponse>, Status> {
        self.increment_request_count();
        let req = request.into_inner();

        info!("Encrypting data with key: {}", req.key_name);

        let context = req.context.as_deref();

        let ciphertext = self
            .transit
            .encrypt(&req.key_name, &req.plaintext, context, None)
            .await
            .map_err(|e| Status::internal(format!("Encryption failed: {}", e)))?;

        // TODO: Implement get_key_version in TransitEngine
        // Get key version from transit service
        // let key_version = self.transit
        //     .get_key_version(&req.key_name)
        //     .await
        //     .unwrap_or(1);
        let key_version = 1; // Placeholder

        let response = EncryptResponse {
            ciphertext,
            key_version,
        };

        Ok(Response::new(response))
    }

    #[instrument(skip(self, request))]
    async fn decrypt(
        &self,
        request: Request<DecryptRequest>,
    ) -> Result<Response<DecryptResponse>, Status> {
        self.increment_request_count();
        let req = request.into_inner();

        info!("Decrypting data with key: {}", req.key_name);

        let context = req.context.as_deref();

        let plaintext = self
            .transit
            .decrypt(&req.key_name, &req.ciphertext, context)
            .await
            .map_err(|e| Status::internal(format!("Decryption failed: {}", e)))?;

        let response = DecryptResponse { plaintext };

        Ok(Response::new(response))
    }

    #[instrument(skip(self, request))]
    async fn sign(&self, request: Request<SignRequest>) -> Result<Response<SignResponse>, Status> {
        self.increment_request_count();
        let req = request.into_inner();

        info!("Signing data with key: {}", req.key_name);

        let algorithm = SignatureAlgorithm::try_from(req.algorithm)
            .map_err(|_| Status::invalid_argument("Invalid signature algorithm"))?;

        use secreton_crypto::transit::SignatureAlgorithm as CryptoSigAlg;
        let crypto_algorithm = match algorithm {
            SignatureAlgorithm::Ed25519Signature => Some(CryptoSigAlg::Ed25519),
            SignatureAlgorithm::EcdsaP256Sha256 => Some(CryptoSigAlg::EcdsaP256),
            _ => None,
        };

        let signature = self
            .transit
            .sign(&req.key_name, &req.data, crypto_algorithm, None)
            .await
            .map_err(|e| Status::internal(format!("Signing failed: {}", e)))?;

        // TODO: Implement get_key_version in TransitEngine
        // Get key version from transit service
        // let key_version = self.transit
        //     .get_key_version(&req.key_name)
        //     .await
        //     .unwrap_or(1);
        let key_version = 1; // Placeholder

        let response = SignResponse {
            signature,
            key_version,
        };

        Ok(Response::new(response))
    }

    #[instrument(skip(self, request))]
    async fn verify(
        &self,
        request: Request<VerifyRequest>,
    ) -> Result<Response<VerifyResponse>, Status> {
        self.increment_request_count();
        let req = request.into_inner();

        info!("Verifying signature with key: {}", req.key_name);

        let algorithm = SignatureAlgorithm::try_from(req.algorithm)
            .map_err(|_| Status::invalid_argument("Invalid signature algorithm"))?;

        use secreton_crypto::transit::SignatureAlgorithm as CryptoSigAlg;
        let crypto_algorithm = match algorithm {
            SignatureAlgorithm::Ed25519Signature => Some(CryptoSigAlg::Ed25519),
            SignatureAlgorithm::EcdsaP256Sha256 => Some(CryptoSigAlg::EcdsaP256),
            _ => None,
        };

        let valid = self
            .transit
            .verify(&req.key_name, &req.data, &req.signature, crypto_algorithm)
            .await
            .map_err(|e| Status::internal(format!("Verification failed: {}", e)))?;

        let response = VerifyResponse { valid };

        Ok(Response::new(response))
    }

    #[instrument(skip(self, request))]
    async fn rotate_key(
        &self,
        request: Request<RotateKeyRequest>,
    ) -> Result<Response<RotateKeyResponse>, Status> {
        self.increment_request_count();
        let req = request.into_inner();

        info!("Rotating key: {}", req.key_name);

        self.transit
            .rotate_key(&req.key_name)
            .await
            .map_err(|e| Status::internal(format!("Key rotation failed: {}", e)))?;

        // TODO: Implement get_key_version in TransitEngine
        // Get new version after rotation
        // let new_version = self.transit
        //     .get_key_version(&req.key_name)
        //     .await
        //     .unwrap_or(2);
        let new_version = 2; // Placeholder

        let response = RotateKeyResponse {
            new_version,
            rotated_at: chrono::Utc::now().timestamp(),
        };

        Ok(Response::new(response))
    }

    #[instrument(skip(self, _request))]
    async fn get_cluster_status(
        &self,
        _request: Request<ClusterStatusRequest>,
    ) -> Result<Response<ClusterStatusResponse>, Status> {
        #[cfg(not(feature = "raft-consensus"))]
        {
            return Err(Status::unimplemented("Raft consensus feature not enabled"));
        }

        #[cfg(feature = "raft-consensus")]
        {
            info!("Getting cluster status via gRPC");

            // Get Raft storage backend
            let raft_storage = self
                .storage
                .as_any()
                .downcast_ref::<secreton_storage::raft::RaftCluster>()
                .ok_or_else(|| {
                    error!("Storage backend is not a Raft cluster");
                    Status::failed_precondition("Raft cluster not configured")
                })?;

            // Get cluster status
            let status = raft_storage.status().await.map_err(|e| {
                error!("Failed to get cluster status: {}", e);
                Status::internal(format!("Failed to get cluster status: {}", e))
            })?;

            // Determine node state
            let state = if status.is_leader {
                "leader".to_string()
            } else if status.leader_id.is_some() {
                "follower".to_string()
            } else {
                "candidate".to_string()
            };

            // Build node information
            let nodes = status
                .membership
                .iter()
                .map(|&node_id| {
                    let node_state = if Some(node_id) == status.leader_id {
                        "leader".to_string()
                    } else {
                        "follower".to_string()
                    };

                    // Get node address (use node_id as placeholder if not configured)
                    let address = format!("node-{}", node_id);

                    NodeInfo {
                        node_id,
                        address,
                        state: node_state,
                    }
                })
                .collect();

            // Record metrics (only if metrics feature is enabled)
            #[cfg(feature = "metrics")]
            {
                counter!("secreton_grpc_cluster_status_requests").increment(1);
                gauge!("secreton_raft_current_term").set(status.current_term as f64);
                gauge!("secreton_raft_is_leader").set(if status.is_leader { 1.0 } else { 0.0 });
            }

            let response = ClusterStatusResponse {
                node_id: status.node_id,
                state,
                leader_id: status.leader_id.unwrap_or(0),
                term: status.current_term,
                commit_index: status.last_applied.unwrap_or(0),
                nodes,
            };

            info!("Cluster status retrieved successfully via gRPC");
            Ok(Response::new(response))
        }
    }

    #[instrument(skip(self, _request))]
    async fn list_peers(
        &self,
        _request: Request<ListPeersRequest>,
    ) -> Result<Response<ListPeersResponse>, Status> {
        #[cfg(not(feature = "raft-consensus"))]
        {
            return Err(Status::unimplemented("Raft consensus feature not enabled"));
        }

        #[cfg(feature = "raft-consensus")]
        {
            info!("Listing cluster peers via gRPC");

            // Get Raft storage backend
            let raft_storage = self
                .storage
                .as_any()
                .downcast_ref::<secreton_storage::raft::RaftCluster>()
                .ok_or_else(|| {
                    error!("Storage backend is not a Raft cluster");
                    Status::failed_precondition("Raft cluster not configured")
                })?;

            // Get cluster status
            let status = raft_storage.status().await.map_err(|e| {
                error!("Failed to get cluster status: {}", e);
                Status::internal(format!("Failed to get cluster status: {}", e))
            })?;

            // Build peer list
            let peers: Vec<PeerInfo> = status
                .membership
                .iter()
                .map(|&node_id| {
                    let is_leader = Some(node_id) == status.leader_id;
                    let state = if is_leader {
                        "leader".to_string()
                    } else {
                        "follower".to_string()
                    };

                    // Calculate replication lag for followers
                    let replication_lag = if !is_leader
                        && status.last_log_index.is_some()
                        && status.last_applied.is_some()
                    {
                        Some(
                            status
                                .last_log_index
                                .unwrap()
                                .saturating_sub(status.last_applied.unwrap()),
                        )
                    } else {
                        None
                    };

                    // Get node address and health status (use placeholders)
                    let address = format!("node-{}", node_id);
                    let health = "unknown".to_string();
                    let last_heartbeat = None;

                    PeerInfo {
                        node_id,
                        address,
                        state,
                        health,
                        replication_lag,
                        last_heartbeat,
                    }
                })
                .collect();

            let total = peers.len() as i32;

            // Record metrics (only if metrics feature is enabled)
            #[cfg(feature = "metrics")]
            {
                counter!("secreton_grpc_list_peers_requests").increment(1);
            }

            let response = ListPeersResponse { peers, total };

            info!("Listed {} peers via gRPC", total);
            Ok(Response::new(response))
        }
    }

    #[instrument(skip(self))]
    async fn add_node(
        &self,
        request: Request<AddNodeRequest>,
    ) -> Result<Response<AddNodeResponse>, Status> {
        #[cfg(not(feature = "raft-consensus"))]
        {
            let _ = request; // Suppress unused warning
            return Err(Status::unimplemented("Raft consensus feature not enabled"));
        }

        #[cfg(feature = "raft-consensus")]
        {
            let req = request.into_inner();

            info!("Adding node {} at {} via gRPC", req.node_id, req.address);

            // Validate request
            if req.address.is_empty() {
                return Err(Status::invalid_argument("Address cannot be empty"));
            }

            // Get Raft storage backend
            let raft_storage = self
                .storage
                .as_any()
                .downcast_ref::<secreton_storage::raft::RaftCluster>()
                .ok_or_else(|| {
                    error!("Storage backend is not a Raft cluster");
                    Status::failed_precondition("Raft cluster not configured")
                })?;

            // Add the node
            raft_storage
                .add_node(req.node_id, req.address.clone())
                .await
                .map_err(|e| {
                    error!("Failed to add node: {}", e);
                    Status::internal(format!("Failed to add node: {}", e))
                })?;

            // Record metrics (only if metrics feature is enabled)
            #[cfg(feature = "metrics")]
            {
                counter!("secreton_grpc_add_node_requests").increment(1);
                counter!("secreton_raft_peers_added").increment(1);
            }

            let response = AddNodeResponse {
                success: true,
                message: format!("Node {} added successfully", req.node_id),
            };

            info!("Node {} added successfully via gRPC", req.node_id);
            Ok(Response::new(response))
        }
    }

    #[instrument(skip(self, _request))]
    async fn create_snapshot(
        &self,
        _request: Request<CreateSnapshotRequest>,
    ) -> Result<Response<CreateSnapshotResponse>, Status> {
        // Implement using generated types
        Err(Status::unimplemented(
            "Snapshot creation via gRPC not yet implemented",
        ))
    }

    #[instrument(skip(self, _request))]
    async fn list_snapshots(
        &self,
        _request: Request<ListSnapshotsRequest>,
    ) -> Result<Response<ListSnapshotsResponse>, Status> {
        Err(Status::unimplemented(
            "Snapshot listing via gRPC not yet implemented",
        ))
    }

    #[instrument(skip(self, _request))]
    async fn restore_snapshot(
        &self,
        _request: Request<RestoreSnapshotRequest>,
    ) -> Result<Response<RestoreSnapshotResponse>, Status> {
        Err(Status::unimplemented(
            "Snapshot restoration via gRPC not yet implemented",
        ))
    }

    #[instrument(skip(self))]
    async fn remove_node(
        &self,
        request: Request<RemoveNodeRequest>,
    ) -> Result<Response<RemoveNodeResponse>, Status> {
        #[cfg(not(feature = "raft-consensus"))]
        {
            let _ = request; // Suppress unused warning
            return Err(Status::unimplemented("Raft consensus feature not enabled"));
        }

        #[cfg(feature = "raft-consensus")]
        {
            let req = request.into_inner();

            info!("Removing node {} via gRPC", req.node_id);

            // Get Raft storage backend
            let raft_storage = self
                .storage
                .as_any()
                .downcast_ref::<secreton_storage::raft::RaftCluster>()
                .ok_or_else(|| {
                    error!("Storage backend is not a Raft cluster");
                    Status::failed_precondition("Raft cluster not configured")
                })?;

            // Check if removing this node would break quorum
            let status = raft_storage.status().await.map_err(|e| {
                error!("Failed to get cluster status: {}", e);
                Status::internal(format!("Failed to get cluster status: {}", e))
            })?;

            let remaining_nodes = status.membership.len() - 1;
            if remaining_nodes < 2 {
                warn!(
                    "Cannot remove node: would break quorum (remaining nodes: {})",
                    remaining_nodes
                );
                return Err(Status::failed_precondition(
                    "Cannot remove node: would break quorum. Minimum 2 nodes required.",
                ));
            }

            // Remove the node
            raft_storage.remove_node(req.node_id).await.map_err(|e| {
                error!("Failed to remove node: {}", e);
                Status::internal(format!("Failed to remove node: {}", e))
            })?;

            // Record metrics (only if metrics feature is enabled)
            #[cfg(feature = "metrics")]
            {
                counter!("secreton_grpc_remove_node_requests").increment(1);
                counter!("secreton_raft_peers_removed").increment(1);
            }

            let response = RemoveNodeResponse {
                success: true,
                message: format!("Node {} removed successfully", req.node_id),
            };

            info!("Node {} removed successfully via gRPC", req.node_id);
            Ok(Response::new(response))
        }
    }

    #[instrument(skip(self, request))]
    async fn health_check(
        &self,
        request: Request<HealthCheckRequest>,
    ) -> Result<Response<HealthCheckResponse>, Status> {
        let req = request.into_inner();

        info!("Health check for service: {}", req.service);

        // Check storage backend health
        let storage_health = match self.storage.health_check().await {
            Ok(_) => DependencyHealth {
                name: "storage".to_string(),
                status: HealthStatus::Healthy as i32,
                error: None,
                response_time_ms: 0,
            },
            Err(e) => DependencyHealth {
                name: "storage".to_string(),
                status: HealthStatus::Unhealthy as i32,
                error: Some(e.to_string()),
                response_time_ms: 0,
            },
        };

        // Check transit engine health (always healthy if initialized)
        let transit_health = DependencyHealth {
            name: "transit_engine".to_string(),
            status: HealthStatus::Healthy as i32,
            error: None,
            response_time_ms: 0,
        };

        let mut dependencies = std::collections::HashMap::new();
        dependencies.insert("storage".to_string(), storage_health.clone());
        dependencies.insert("transit_engine".to_string(), transit_health);

        // Overall status is unhealthy if any dependency is unhealthy
        let overall_status = if storage_health.status == HealthStatus::Unhealthy as i32 {
            HealthStatus::Unhealthy
        } else {
            HealthStatus::Healthy
        };

        let response = HealthCheckResponse {
            info: Some(HealthInfo {
                service_name: "secreton".to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
                status: overall_status as i32,
                uptime_seconds: self.get_uptime_seconds(),
                dependencies,
            }),
        };

        Ok(Response::new(response))
    }

    #[instrument(skip(self, _request))]
    async fn get_metrics(
        &self,
        _request: Request<MetricsRequest>,
    ) -> Result<Response<MetricsResponse>, Status> {
        info!("Getting metrics");

        // Count metrics from storage
        let total_secrets = self.storage.count_secrets().await.unwrap_or(0);
        let total_keys = self.storage.count_keys().await.unwrap_or(0);
        // Policy count not yet exposed in MetricsResponse
        let _total_policies = self.storage.count_policies().await.unwrap_or(0);

        let response = MetricsResponse {
            total_secrets: total_secrets as i64,
            total_keys: total_keys as i64,
            operations_count: 0,
            avg_response_time_ms: 0.0,
            operation_counts: std::collections::HashMap::new(),
        };

        Ok(Response::new(response))
    }

    // ============================================================================
    // Namespace Management Methods
    // ============================================================================

    #[instrument(skip(self, request))]
    async fn list_namespaces(
        &self,
        request: Request<ListNamespacesRequest>,
    ) -> Result<Response<ListNamespacesResponse>, Status> {
        let req = request.into_inner();

        let hierarchy = self.services.namespace.hierarchy();
        let mut namespaces: Vec<&secreton_core::namespace::Namespace> = hierarchy.list_all();

        // Apply filters
        if let Some(namespace_type) = req.namespace_type {
            let ns_type = match namespace_type {
                1 => secreton_core::namespace::NamespaceType::Pusat,
                2 => secreton_core::namespace::NamespaceType::Wilayah,
                3 => secreton_core::namespace::NamespaceType::Satker,
                _ => return Err(Status::invalid_argument("Invalid namespace type")),
            };
            namespaces.retain(|ns| ns.namespace_type == ns_type);
        }

        if let Some(parent) = &req.parent {
            namespaces.retain(|ns| ns.parent.as_ref() == Some(&parent.clone()));
        }

        if let Some(is_active) = req.is_active {
            namespaces.retain(|ns| ns.is_active == is_active);
        }

        if let Some(search) = &req.search {
            let search_lower = search.to_lowercase();
            namespaces.retain(|ns| {
                ns.name.to_lowercase().contains(&search_lower)
                    || ns.id.to_lowercase().contains(&search_lower)
            });
        }

        let total = namespaces.len() as i64;
        let limit = req.limit.unwrap_or(50) as usize;
        let offset = req.offset.unwrap_or(0) as usize;

        let namespace_infos: Vec<NamespaceInfo> = namespaces
            .iter()
            .skip(offset)
            .take(limit)
            .map(|ns| self.namespace_to_grpc_info(ns, &hierarchy))
            .collect();

        Ok(Response::new(ListNamespacesResponse {
            namespaces: namespace_infos,
            total,
        }))
    }

    #[instrument(skip(self, request))]
    async fn create_namespace(
        &self,
        request: Request<CreateNamespaceRequest>,
    ) -> Result<Response<CreateNamespaceResponse>, Status> {
        let created_by = request.metadata().get("x-user").map(|v| v.to_str().unwrap_or("grpc-user")).unwrap_or("grpc-user").to_string();
        let req = request.into_inner();

        let namespace_type = match req.namespace_type {
            1 => secreton_core::namespace::NamespaceType::Pusat,
            2 => secreton_core::namespace::NamespaceType::Wilayah,
            3 => secreton_core::namespace::NamespaceType::Satker,
            _ => return Err(Status::invalid_argument("Invalid namespace type")),
        };

        // Note: The logic here assumes `add_wilayah` and `add_satker` return `secreton_core::namespace::Namespace`
        // We will execute them inside the update loop if needed
        let mut hierarchy = self.services.namespace.hierarchy();

        let namespace = match namespace_type {
            secreton_core::namespace::NamespaceType::Pusat => {
                return Err(Status::invalid_argument("Cannot create Pusat namespace via gRPC endpoint"));
            }
            secreton_core::namespace::NamespaceType::Wilayah => {
                hierarchy
                    .add_wilayah(req.id.clone(), req.name.clone(), created_by)
                    .map_err(|e| Status::internal(e.to_string()))?
            }
            secreton_core::namespace::NamespaceType::Satker => {
                let parent = req
                    .parent
                    .ok_or_else(|| Status::invalid_argument("Satker requires parent"))?;
                hierarchy
                    .add_satker(
                        req.id.clone(),
                        req.name.clone(),
                        parent,
                        created_by,
                    )
                    .map_err(|e| Status::internal(e.to_string()))?
            }
        };

        // Update hierarchy mutably to add policies and quotas
        if let Some(ns) = hierarchy.get_namespace_mut(&namespace.id) {
            for policy in req.policies {
                ns.add_policy(policy);
            }

            if let Some(quotas) = req.quotas {
                ns.update_quotas(self.grpc_quotas_to_core(quotas));
            }

            for (key, value) in req.metadata {
                ns.metadata.insert(key, value);
            }
        }

        self.services.namespace.update_hierarchy(hierarchy.clone());

        // We fetch the updated namespace info
        let updated_namespace = hierarchy.get_namespace(&namespace.id)
            .ok_or_else(|| Status::internal("Failed to get updated namespace"))?;

        let namespace_info = self.namespace_to_grpc_info(updated_namespace, &hierarchy);

        Ok(Response::new(CreateNamespaceResponse {
            namespace: Some(namespace_info),
        }))
    }

    #[instrument(skip(self, request))]
    async fn get_namespace(
        &self,
        request: Request<GetNamespaceRequest>,
    ) -> Result<Response<GetNamespaceResponse>, Status> {
        let req = request.into_inner();
        let hierarchy = self.services.namespace.hierarchy();

        let namespace = hierarchy
            .get_namespace(&req.id)
            .ok_or_else(|| Status::not_found(format!("Namespace {} not found", req.id)))?;

        Ok(Response::new(GetNamespaceResponse {
            namespace: Some(self.namespace_to_grpc_info(namespace, &hierarchy)),
        }))
    }

    #[instrument(skip(self, request))]
    async fn update_namespace(
        &self,
        request: Request<UpdateNamespaceRequest>,
    ) -> Result<Response<UpdateNamespaceResponse>, Status> {
        let req = request.into_inner();
        let mut hierarchy = self.services.namespace.hierarchy();

        if let Some(ns) = hierarchy.get_namespace_mut(&req.id) {
            if let Some(name) = req.name {
                ns.name = name;
            }
            if let Some(is_active) = req.is_active {
                ns.is_active = is_active;
            }
            if !req.policies.is_empty() {
                ns.policies = req.policies;
            }
            if let Some(quotas) = req.quotas {
                ns.update_quotas(self.grpc_quotas_to_core(quotas));
            }
            for (key, value) in req.metadata {
                ns.metadata.insert(key, value);
            }
            ns.updated_at = chrono::Utc::now();
        } else {
            return Err(Status::not_found(format!("Namespace {} not found", req.id)));
        }

        self.services.namespace.update_hierarchy(hierarchy.clone());

        let namespace = hierarchy.get_namespace(&req.id).unwrap();

        Ok(Response::new(UpdateNamespaceResponse {
            namespace: Some(self.namespace_to_grpc_info(namespace, &hierarchy)),
        }))
    }

    #[instrument(skip(self, request))]
    async fn delete_namespace(
        &self,
        request: Request<DeleteNamespaceRequest>,
    ) -> Result<Response<DeleteNamespaceResponse>, Status> {
        let req = request.into_inner();

        let hierarchy = self.services.namespace.hierarchy();

        if hierarchy.get_namespace(&req.id).is_none() {
            return Err(Status::not_found(format!("Namespace {} not found", req.id)));
        }

        // `NamespaceHierarchy` doesn't seem to have a `remove_namespace` method yet,
        // but we can try to remove it or return unimplemented if the core service doesn't support deletion.
        return Err(Status::unimplemented("Namespace deletion not yet fully implemented in core hierarchy"));
    }

    #[instrument(skip(self, request))]
    async fn get_namespace_stats(
        &self,
        request: Request<GetNamespaceStatsRequest>,
    ) -> Result<Response<GetNamespaceStatsResponse>, Status> {
        let req = request.into_inner();
        let hierarchy = self.services.namespace.hierarchy();

        let namespace = hierarchy
            .get_namespace(&req.id)
            .ok_or_else(|| Status::not_found(format!("Namespace {} not found", req.id)))?;

        let children_count = hierarchy.get_descendants(&req.id).len() as i32;

        let usage_percentage = if let Some(max_secrets) = namespace.quotas.max_secrets {
            if max_secrets > 0 {
                (namespace.quotas.current_usage.secrets_count as f64 / max_secrets as f64) * 100.0
            } else {
                0.0
            }
        } else {
            0.0
        };

        let is_quota_exceeded = {
            let q = &namespace.quotas;
            let usage = &q.current_usage;
            let mut exceeded = false;

            if let Some(max) = q.max_secrets && usage.secrets_count >= max { exceeded = true; }
            if let Some(max) = q.max_storage_bytes && usage.storage_bytes >= max { exceeded = true; }
            if let Some(max) = q.max_leases && usage.leases_count >= max { exceeded = true; }
            if let Some(max) = q.max_policies && usage.policies_count >= max { exceeded = true; }

            exceeded
        };

        Ok(Response::new(GetNamespaceStatsResponse {
            namespace_id: req.id.clone(),
            quota_usage: Some(QuotaUsage {
                secrets_count: namespace.quotas.current_usage.secrets_count as i64,
                storage_bytes: namespace.quotas.current_usage.storage_bytes as i64,
                leases_count: namespace.quotas.current_usage.leases_count as i64,
                policies_count: namespace.quotas.current_usage.policies_count as i64,
            }),
            quota_limits: Some(self.core_quotas_to_grpc(&namespace.quotas)),
            usage_percentage,
            is_quota_exceeded,
            children_count,
            total_descendants: children_count, // Using descendants length as total descendants
            active_leases: namespace.quotas.current_usage.leases_count as i64,
            active_policies: namespace.quotas.current_usage.policies_count as i32,
        }))
    }

    #[instrument(skip(self, request))]
    async fn renew_lease(
        &self,
        request: Request<RenewLeaseRequest>,
    ) -> Result<Response<RenewLeaseResponse>, Status> {
        let req = request.into_inner();

        let lease = self
            .services
            .lease_manager
            .renew_lease(&req.lease_id, req.increment.unwrap_or(0))
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        let lease_duration = (lease.expired_at - chrono::Utc::now()).num_seconds().max(0);

        Ok(Response::new(RenewLeaseResponse {
            lease_id: lease.id,
            expired_at: lease.expired_at.timestamp(),
            lease_duration,
            renewable: lease.renewable,
            renew_count: lease.renew_count,
            max_renewals: lease.max_renewals,
        }))
    }

    #[instrument(skip(self, request))]
    async fn revoke_lease(
        &self,
        request: Request<RevokeLeaseRequest>,
    ) -> Result<Response<RevokeLeaseResponse>, Status> {
        let req = request.into_inner();

        let revoked_ids = self.services
            .lease_manager
            .revoke_lease(&req.lease_id)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(RevokeLeaseResponse {
            lease_id: req.lease_id,
            revoked_count: revoked_ids.len() as i32,
            revoked_ids,
        }))
    }

    #[instrument(skip(self, request))]
    async fn revoke_lease_prefix(
        &self,
        request: Request<RevokeLeasePrefixRequest>,
    ) -> Result<Response<RevokeLeasePrefixResponse>, Status> {
        let _req = request.into_inner();

        // `revoke_prefix` does not exist on `LeaseManager`. We might have to fetch and revoke or just return unimplemented.
        Err(Status::unimplemented("revoke_lease_prefix not yet supported by lease manager"))
    }

    #[instrument(skip(self, request))]
    async fn lookup_lease(
        &self,
        request: Request<LookupLeaseRequest>,
    ) -> Result<Response<LookupLeaseResponse>, Status> {
        let req = request.into_inner();

        info!("Looking up lease: {}", req.lease_id);

        let lease = self
            .services
            .lease_manager
            .lookup_lease(&req.lease_id)
            .await
            .map_err(|e| Status::not_found(format!("Lease not found: {}", e)))?;

        let ttl = (lease.expired_at - chrono::Utc::now()).num_seconds().max(0);

        let response = LookupLeaseResponse {
            lease_id: lease.id,
            user: lease.user,
            resource: lease.resource,
            resource_type: lease.resource_type,
            namespace: lease.namespace,
            issued_at: lease.issued_at.timestamp(),
            expired_at: lease.expired_at.timestamp(),
            ttl,
            status: lease.status,
            renewable: lease.renewable,
            max_ttl: lease.max_ttl,
            renew_count: lease.renew_count,
            max_renewals: lease.max_renewals,
            last_renewed_at: lease.last_renewed_at.map(|t| t.timestamp()),
            parent_id: lease.parent_id,
            child_ids: lease.child_ids,
            metadata: lease.metadata,
        };

        Ok(Response::new(response))
    }

    #[instrument(skip(self, request))]
    async fn list_leases(
        &self,
        request: Request<ListLeasesRequest>,
    ) -> Result<Response<ListLeasesResponse>, Status> {
        let req = request.into_inner();

        info!("Listing leases");

        let limit = req.limit.unwrap_or(50) as i64;
        let offset = req.offset.unwrap_or(0) as i64;

        let total = self
            .services
            .lease_manager
            .count_leases(
                req.user_id.clone(),
                req.namespace.clone(),
                req.resource_type.clone(),
                req.status.clone(),
            )
            .await
            .map_err(|e| Status::internal(format!("Failed to count leases: {}", e)))?;

        let leases = self
            .services
            .lease_manager
            .list_leases(
                req.user_id.clone(),
                req.namespace.clone(),
                req.resource_type.clone(),
                req.status.clone(),
                Some(limit),
                Some(offset),
            )
            .await
            .map_err(|e| Status::internal(format!("Failed to list leases: {}", e)))?;

        let lease_responses: Vec<LookupLeaseResponse> = leases
            .into_iter()
            .map(|lease| {
                let ttl = (lease.expired_at - chrono::Utc::now()).num_seconds().max(0);

                LookupLeaseResponse {
                    lease_id: lease.id,
                    user: lease.user,
                    resource: lease.resource,
                    resource_type: lease.resource_type,
                    namespace: lease.namespace,
                    issued_at: lease.issued_at.timestamp(),
                    expired_at: lease.expired_at.timestamp(),
                    ttl,
                    status: lease.status,
                    renewable: lease.renewable,
                    max_ttl: lease.max_ttl,
                    renew_count: lease.renew_count,
                    max_renewals: lease.max_renewals,
                    last_renewed_at: lease.last_renewed_at.map(|t| t.timestamp()),
                    parent_id: lease.parent_id,
                    child_ids: lease.child_ids,
                    metadata: lease.metadata,
                }
            })
            .collect();

        let total_i64 = total as i64;
        let has_next = offset + limit < total_i64;
        let has_previous = offset > 0;

        let response = ListLeasesResponse {
            leases: lease_responses,
            total: total_i64,
            limit: limit as i32,
            offset: offset as i32,
            has_next,
            has_previous,
        };

        Ok(Response::new(response))
    }

    #[instrument(skip(self, _request))]
    async fn get_lease_stats(
        &self,
        _request: Request<GetLeaseStatsRequest>,
    ) -> Result<Response<GetLeaseStatsResponse>, Status> {
        info!("Getting lease statistics");

        let stats = self
            .services
            .lease_manager
            .get_stats()
            .await
            .map_err(|e| Status::internal(format!("Failed to get lease stats: {}", e)))?;

        let response = GetLeaseStatsResponse {
            active_count: stats.active_count as i64,
            revoked_count: stats.revoked_count as i64,
            expired_count: stats.expired_count as i64,
            expiring_soon_count: stats.expiring_soon_count as i64,
            unique_users: stats.unique_users as i64,
            unique_namespaces: stats.unique_namespaces as i64,
            by_resource_type: self.get_stats_by_resource_type().await,
            by_namespace: self.get_stats_by_namespace().await
        };

        Ok(Response::new(response))
    }

    // ============================================================================
    // Database Secrets Engine Methods
    // ============================================================================

    #[instrument(skip(self, request))]
    async fn generate_database_credentials(
        &self,
        request: Request<GenerateDatabaseCredentialsRequest>,
    ) -> Result<Response<GenerateDatabaseCredentialsResponse>, Status> {
        let user = request.metadata().get("x-user").map(|v| v.to_str().unwrap_or("grpc_user")).unwrap_or("grpc_user").to_string();
        let req = request.into_inner();

        let (credentials, lease) = self
            .services
            .database_engine
            .generate_credentials_ensure_lease(
                &req.role_name,
                req.ttl_seconds,
                &self.services.lease_manager,
                &user,
            )
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        let grpc_credentials = DatabaseCredentials {
            username: credentials.username,
            password: credentials.password,
            connection_url: credentials.connection_url,
            database: String::new(), // We don't have db name here unless we fetch the role
            role: credentials.role_name,
        };

        Ok(Response::new(GenerateDatabaseCredentialsResponse {
            lease_id: lease.id,
            lease_duration: (lease.expired_at - chrono::Utc::now()).num_seconds().max(0),
            renewable: lease.renewable,
            credentials: Some(grpc_credentials),
        }))
    }

    #[instrument(skip(self, request))]
    async fn create_database_role(
        &self,
        request: Request<CreateDatabaseRoleRequest>,
    ) -> Result<Response<CreateDatabaseRoleResponse>, Status> {
        let req = request.into_inner();

        let role = secreton_core::services::secrets::database::DatabaseRole {
            name: req.role_name.clone(),
            db_name: req.db_name.clone(),
            default_ttl: req.default_ttl,
            max_ttl: req.max_ttl,
            creation_statements: req.creation_statements,
            revocation_statements: req.revocation_statements,
            rotation_statements: req.rotation_statements,
            renew_statements: req.renew_statements,
        };

        self.services
            .database_engine
            .create_role(role)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(CreateDatabaseRoleResponse {
            name: req.role_name,
            db_name: req.db_name,
            default_ttl: req.default_ttl,
            max_ttl: req.max_ttl,
        }))
    }

    #[instrument(skip(self, request))]
    async fn get_database_role(
        &self,
        request: Request<GetDatabaseRoleRequest>,
    ) -> Result<Response<GetDatabaseRoleResponse>, Status> {
        let req = request.into_inner();

        let role = self
            .services
            .database_engine
            .get_role(&req.role_name)
            .await
            .ok_or_else(|| Status::not_found("Role not found"))?;

        Ok(Response::new(GetDatabaseRoleResponse {
            role: Some(DatabaseRoleInfo {
                name: role.name,
                db_name: role.db_name,
                default_ttl: role.default_ttl,
                max_ttl: role.max_ttl,
                creation_statements: role.creation_statements,
                revocation_statements: role.revocation_statements,
                rotation_statements: role.rotation_statements,
                renew_statements: role.renew_statements,
            }),
        }))
    }

    #[instrument(skip(self, request))]
    async fn list_database_roles(
        &self,
        request: Request<ListDatabaseRolesRequest>,
    ) -> Result<Response<ListDatabaseRolesResponse>, Status> {
        let req = request.into_inner();

        let roles = self.services.database_engine.list_roles().await;

        let total = roles.len() as i32;

        // Apply basic pagination
        let offset = req.offset.unwrap_or(0) as usize;
        let limit = req.limit.unwrap_or(50) as usize;

        let paginated_roles = roles
            .into_iter()
            .skip(offset)
            .take(limit)
            .collect();

        Ok(Response::new(ListDatabaseRolesResponse {
            role_names: paginated_roles,
            total,
        }))
    }

    #[instrument(skip(self, request))]
    async fn update_database_role(
        &self,
        request: Request<UpdateDatabaseRoleRequest>,
    ) -> Result<Response<UpdateDatabaseRoleResponse>, Status> {
        let req = request.into_inner();

        let role = secreton_core::services::secrets::database::DatabaseRole {
            name: req.role_name.clone(),
            db_name: req.db_name.clone(),
            default_ttl: req.default_ttl,
            max_ttl: req.max_ttl,
            creation_statements: req.creation_statements,
            revocation_statements: req.revocation_statements,
            rotation_statements: req.rotation_statements,
            renew_statements: req.renew_statements,
        };

        // For now, create_role functions as update_role in engine if it exists
        self.services
            .database_engine
            .create_role(role)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(UpdateDatabaseRoleResponse {
            name: req.role_name,
            db_name: req.db_name,
            default_ttl: req.default_ttl,
            max_ttl: req.max_ttl,
        }))
    }

    #[instrument(skip(self, request))]
    async fn delete_database_role(
        &self,
        request: Request<DeleteDatabaseRoleRequest>,
    ) -> Result<Response<DeleteDatabaseRoleResponse>, Status> {
        let req = request.into_inner();

        let success = self
            .services
            .database_engine
            .delete_role(&req.role_name)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(DeleteDatabaseRoleResponse {
            success,
            message: if success {
                "Role deleted successfully".to_string()
            } else {
                "Role not found".to_string()
            },
        }))
    }

    #[instrument(skip(self, request))]
    async fn configure_database_connection(
        &self,
        request: Request<ConfigureDatabaseConnectionRequest>,
    ) -> Result<Response<ConfigureDatabaseConnectionResponse>, Status> {
        let req = request.into_inner();

        let db_type = match DatabaseType::try_from(req.db_type) {
            Ok(DatabaseType::Postgresql) => secreton_core::services::secrets::database::DatabaseType::PostgreSQL,
            Ok(DatabaseType::Mysql) => secreton_core::services::secrets::database::DatabaseType::MySQL,
            Ok(DatabaseType::Mongodb) => secreton_core::services::secrets::database::DatabaseType::MongoDB,
            Ok(DatabaseType::Redis) => secreton_core::services::secrets::database::DatabaseType::Redis,
            Ok(DatabaseType::Cassandra) => secreton_core::services::secrets::database::DatabaseType::Cassandra,
            Ok(DatabaseType::Mssql) => secreton_core::services::secrets::database::DatabaseType::MSSQL,
            _ => return Err(Status::invalid_argument("Unsupported database type")),
        };

        let config = secreton_core::services::secrets::database::DatabaseConnection {
            name: req.name.clone(),
            db_type,
            connection_url: req.connection_url,
            max_open_connections: req.max_open_connections,
            max_idle_connections: req.max_idle_connections,
            max_connection_lifetime: req.max_connection_lifetime,
            verify_connection: req.verify_connection,
            root_rotation_statements: req.root_rotation_statements,
            username: None, // Can be parsed from connection_url if needed
            password: None,
        };

        self.services
            .database_engine
            .configure_connection(config)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(ConfigureDatabaseConnectionResponse {
            name: req.name,
            db_type: req.db_type,
            verified: req.verify_connection,
        }))
    }

    // ============================================================================
    // Policy Management Methods
    // ============================================================================

    #[instrument(skip(self, request))]
    async fn list_policies(
        &self,
        request: Request<ListPoliciesRequest>,
    ) -> Result<Response<ListPoliciesResponse>, Status> {
        let req = request.into_inner();

        let limit = req.limit.map(|v| v as u32);
        let offset = req.offset.map(|v| v as u32);

        let (policies, total) = self
            .services
            .policy_service
            .list_policies(req.namespace, limit, offset)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        let grpc_policies = policies
            .into_iter()
            .map(|p| self.policy_def_to_grpc(p))
            .collect();

        Ok(Response::new(ListPoliciesResponse {
            policies: grpc_policies,
            total: total.try_into().unwrap_or(i64::MAX),
            limit: req.limit.unwrap_or(50),
            offset: req.offset.unwrap_or(0),
        }))
    }

    #[instrument(skip(self, request))]
    async fn create_policy(
        &self,
        request: Request<CreatePolicyRequest>,
    ) -> Result<Response<CreatePolicyResponse>, Status> {
        let user = request.metadata().get("x-user").map(|v| v.to_str().unwrap_or("grpc_user")).unwrap_or("grpc_user").to_string();
        let req = request.into_inner();

        let rules: Vec<secreton_core::models::PolicyRule> = req
            .rules
            .into_iter()
            .map(|r| self.grpc_rule_to_core(r))
            .collect();

        // Convert empty string namespace to default namespace
        let namespace = if req.namespace.is_empty() {
            "default".to_string()
        } else {
            req.namespace
        };

        let policy = self
            .services
            .policy_service
            .create_policy(req.name, namespace, req.description, rules, user)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(CreatePolicyResponse {
            policy: Some(self.policy_def_to_grpc(policy)),
        }))
    }

    #[instrument(skip(self, request))]
    async fn get_policy(
        &self,
        request: Request<GetPolicyRequest>,
    ) -> Result<Response<GetPolicyResponse>, Status> {
        let namespace = request.metadata().get("x-namespace").map(|v| v.to_str().unwrap_or("default")).unwrap_or("default").to_string();
        let req = request.into_inner();

        let policy = self
            .services
            .policy_service
            .get_policy(&req.name, &namespace)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(GetPolicyResponse {
            policy: Some(self.policy_def_to_grpc(policy)),
        }))
    }

    #[instrument(skip(self, request))]
    async fn update_policy(
        &self,
        request: Request<UpdatePolicyRequest>,
    ) -> Result<Response<UpdatePolicyResponse>, Status> {
        let namespace = request.metadata().get("x-namespace").map(|v| v.to_str().unwrap_or("default")).unwrap_or("default").to_string();
        let user = request.metadata().get("x-user").map(|v| v.to_str().unwrap_or("grpc_user")).unwrap_or("grpc_user").to_string();
        let req = request.into_inner();

        let rules: Vec<secreton_core::models::PolicyRule> = req
            .rules
            .into_iter()
            .map(|r| self.grpc_rule_to_core(r))
            .collect();

        let rules_opt = if rules.is_empty() { None } else { Some(rules) };

        let policy = self
            .services
            .policy_service
            .update_policy(
                &req.name,
                &namespace,
                req.description,
                rules_opt,
                req.is_active,
                user,
            )
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(UpdatePolicyResponse {
            policy: Some(self.policy_def_to_grpc(policy)),
        }))
    }

    #[instrument(skip(self, request))]
    async fn delete_policy(
        &self,
        request: Request<DeletePolicyRequest>,
    ) -> Result<Response<DeletePolicyResponse>, Status> {
        let namespace = request.metadata().get("x-namespace").map(|v| v.to_str().unwrap_or("default")).unwrap_or("default").to_string();
        let req = request.into_inner();

        self.services
            .policy_service
            .delete_policy(&req.name, &namespace)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(DeletePolicyResponse { success: true, message: "Policy deleted".to_string() }))
    }

    #[instrument(skip(self, _request))]
    async fn test_policy(
        &self,
        _request: Request<TestPolicyRequest>,
    ) -> Result<Response<TestPolicyResponse>, Status> {
        // Currently missing implementation for `test_policy` in `secreton_core::services::policy_service`
        // We will return unimplemented or implement it later.
        Err(Status::unimplemented(
            "Test policy not yet fully implemented in core services",
        ))
    }

    // ============================================================================
    // Response Wrapping Methods
    // ============================================================================

    #[instrument(skip(self, request))]
    async fn wrap_data(
        &self,
        request: Request<WrapDataRequest>,
    ) -> Result<Response<WrapDataResponse>, Status> {
        let req = request.into_inner();

        let wrap_req = secreton_core::services::wrapping::WrapRequest {
            data: serde_json::from_str(&req.data_json)
                .map_err(|_| Status::invalid_argument("Invalid JSON data"))?,
            ttl: std::time::Duration::from_secs(req.ttl),
            namespace: req.namespace,
        };

        let response = self
            .services
            .wrapping_service
            .wrap(wrap_req)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(WrapDataResponse {
            token: response.token,
            created_at: response.created_at.timestamp(),
            expires_at: response.expires_at.timestamp(),
            ttl: response.ttl,
            accessor: String::new(), // Not present in WrapResponse
        }))
    }

    #[instrument(skip(self, request))]
    async fn unwrap_token(
        &self,
        request: Request<UnwrapTokenRequest>,
    ) -> Result<Response<UnwrapTokenResponse>, Status> {
        let req = request.into_inner();

        let (data, info) = self
            .services
            .wrapping_service
            .unwrap_with_info(&req.token, &req.namespace)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(UnwrapTokenResponse {
            data_json: data.to_string(),
            created_at: info.created_at.timestamp(),
            expired_at: info.expires_at.timestamp(),
        }))
    }

    #[instrument(skip(self, request))]
    async fn lookup_wrapping_token(
        &self,
        request: Request<LookupWrappingTokenRequest>,
    ) -> Result<Response<LookupWrappingTokenResponse>, Status> {
        let req = request.into_inner();

        let info = self
            .services
            .wrapping_service
            .lookup(&req.token, &req.namespace)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        let status_str = match info.status {
            secreton_core::services::wrapping::TokenStatus::Active => "Active",
            secreton_core::services::wrapping::TokenStatus::Unwrapped => "Unwrapped",
            secreton_core::services::wrapping::TokenStatus::Expired => "Expired",
        };

        Ok(Response::new(LookupWrappingTokenResponse {
            token: info.token,
            created_at: info.created_at.timestamp(),
            expires_at: info.expires_at.timestamp(),
            ttl_remaining: info.ttl_remaining,
            namespace: info.namespace,
            status: status_str.to_string(),
            data_size: info.data_size as i64,
        }))
    }

    #[instrument(skip(self, _request))]
    async fn rewrap_token(
        &self,
        _request: Request<RewrapTokenRequest>,
    ) -> Result<Response<RewrapTokenResponse>, Status> {
        // rewrap not currently supported directly in core WrappingService
        Err(Status::unimplemented(
            "Response token rewrapping not yet supported",
        ))
    }
}

// ================================================================================
// Helper and Snapshot Methods (not part of gRPC trait)
// ================================================================================
impl SecretonGrpcService {
    // ============================================================================
    // Helper Methods for Policy Conversion
    // ============================================================================

    fn policy_def_to_grpc(&self, p: secreton_core::services::policy_service::PolicyDefinition) -> PolicyInfo {
        PolicyInfo {
            id: 0, // No ID in PolicyDefinition
            name: p.name,
            namespace: p.namespace,
            description: p.description,
            rules: p.rules.into_iter().map(|r| self.core_rule_to_grpc(r)).collect(),
            version: p.version as i32,
            is_active: p.is_active,
            created_at: p.created_at.timestamp(),
            updated_at: p.updated_at.timestamp(),
            created_by: p.created_by,
            updated_by: p.updated_by,
            stats: None,
        }
    }

    fn core_rule_to_grpc(&self, r: secreton_core::models::PolicyRule) -> PolicyRule {
        PolicyRule {
            effect: r.effect,
            action: r.action,
            path: r.path,
            condition_json: r.condition.map(|c| c.to_string()),
            control_group: r.control_group.map(|cg| ControlGroup {
                required_approvals: cg.required_approvals,
                approved_by: cg.approved_by,
            }),
            mfa: r.mfa,
        }
    }

    fn grpc_rule_to_core(&self, r: PolicyRule) -> secreton_core::models::PolicyRule {
        secreton_core::models::PolicyRule {
            effect: r.effect,
            action: r.action,
            path: r.path,
            condition: r.condition_json.and_then(|c| serde_json::from_str(&c).ok()),
            control_group: r.control_group.map(|cg| secreton_core::models::ControlGroup {
                required_approvals: cg.required_approvals,
                approved_by: cg.approved_by,
            }),
            mfa: r.mfa,
        }
    }

    // ============================================================================
    // Helper Methods for Namespace Conversion
    // ============================================================================

    #[allow(dead_code)]
    fn namespace_to_grpc_info(
        &self,
        namespace: &secreton_core::namespace::Namespace,
        hierarchy: &secreton_core::namespace::NamespaceHierarchy,
    ) -> NamespaceInfo {
        let children_count = hierarchy.get_descendants(&namespace.id).len() as i32;
        let ancestors: Vec<String> = hierarchy
            .get_ancestors(&namespace.id)
            .iter()
            .map(|ns| ns.id.clone())
            .collect();

        let namespace_type = match namespace.namespace_type {
            secreton_core::namespace::NamespaceType::Pusat => 1,
            secreton_core::namespace::NamespaceType::Wilayah => 2,
            secreton_core::namespace::NamespaceType::Satker => 3,
        };

        NamespaceInfo {
            id: namespace.id.clone(),
            path: namespace.path.clone(),
            parent: namespace.parent.clone(),
            name: namespace.name.clone(),
            namespace_type,
            policies: namespace.policies.clone(),
            quotas: Some(self.core_quotas_to_grpc(&namespace.quotas)),
            metadata: namespace.metadata.clone(),
            created_at: namespace.created_at.timestamp(),
            updated_at: namespace.updated_at.timestamp(),
            created_by: namespace.created_by.clone(),
            is_active: namespace.is_active,
            children_count,
            ancestors,
        }
    }

    #[allow(dead_code)]
    fn core_quotas_to_grpc(
        &self,
        quotas: &secreton_core::namespace::NamespaceQuotas,
    ) -> NamespaceQuotas {
        NamespaceQuotas {
            max_secrets: quotas.max_secrets.map(|v| v as i64),
            max_storage_bytes: quotas.max_storage_bytes.map(|v| v as i64),
            max_leases: quotas.max_leases.map(|v| v as i64),
            max_policies: quotas.max_policies.map(|v| v as i64),
            current_usage: Some(QuotaUsage {
                secrets_count: quotas.current_usage.secrets_count as i64,
                storage_bytes: quotas.current_usage.storage_bytes as i64,
                leases_count: quotas.current_usage.leases_count as i64,
                policies_count: quotas.current_usage.policies_count as i64,
            }),
        }
    }

    #[allow(dead_code)]
    fn grpc_quotas_to_core(
        &self,
        quotas: NamespaceQuotas,
    ) -> secreton_core::namespace::NamespaceQuotas {
        secreton_core::namespace::NamespaceQuotas {
            max_secrets: quotas.max_secrets.map(|v| v as u64),
            max_storage_bytes: quotas.max_storage_bytes.map(|v| v as u64),
            max_leases: quotas.max_leases.map(|v| v as u64),
            max_policies: quotas.max_policies.map(|v| v as u64),
            current_usage: quotas
                .current_usage
                .map(|u| secreton_core::namespace::QuotaUsage {
                    secrets_count: u.secrets_count as u64,
                    storage_bytes: u.storage_bytes as u64,
                    leases_count: u.leases_count as u64,
                    policies_count: u.policies_count as u64,
                })
                .unwrap_or_default(),
        }
    }

    // ============================================================================
    // Snapshot Management Methods
    // ============================================================================

    #[allow(dead_code)]
    #[instrument(skip(self, _request))]
    async fn create_snapshot(
        &self,
        _request: Request<CreateSnapshotRequest>,
    ) -> Result<Response<CreateSnapshotResponse>, Status> {
        #[cfg(not(feature = "raft-consensus"))]
        {
            return Err(Status::unimplemented("Raft consensus feature not enabled"));
        }

        #[cfg(feature = "raft-consensus")]
        {
            info!("Creating Raft snapshot via gRPC");

            // Get Raft storage backend
            let raft_storage = self
                .storage
                .as_any()
                .downcast_ref::<secreton_storage::raft::RaftCluster>()
                .ok_or_else(|| {
                    error!("Storage backend is not a Raft cluster");
                    Status::failed_precondition("Raft cluster not configured")
                })?;

            // Check if this node is the leader
            if !raft_storage.is_leader().await {
                warn!("Snapshot creation attempted on non-leader node");
                return Err(Status::failed_precondition(
                    "Snapshots can only be created on the leader node",
                ));
            }

            // Get current cluster status for metadata
            let status = raft_storage.status().await.map_err(|e| {
                error!("Failed to get cluster status: {}", e);
                Status::internal(format!("Failed to get cluster status: {}", e))
            })?;

            // Generate snapshot ID
            let snapshot_id = format!(
                "snapshot-{}-{}",
                chrono::Utc::now().timestamp(),
                uuid::Uuid::new_v4().to_string().split('-').next().unwrap()
            );

            // Simulate snapshot data
            let snapshot_data = format!(
                "{{\"node_id\":{},\"term\":{},\"index\":{},\"timestamp\":{}}}",
                status.node_id,
                status.current_term,
                status.last_applied.unwrap_or(0),
                chrono::Utc::now().timestamp()
            );
            let snapshot_bytes = snapshot_data.as_bytes();
            let original_size = snapshot_bytes.len() as u64;

            // Compress snapshot data
            #[cfg(feature = "flate2")]
            let (compressed_data, compressed_size) = {
                use flate2::Compression;
                use flate2::write::GzEncoder;
                use std::io::Write;

                let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
                encoder
                    .write_all(snapshot_bytes)
                    .map_err(|e| Status::internal(format!("Failed to compress snapshot: {}", e)))?;
                let compressed_data = encoder.finish().map_err(|e| {
                    Status::internal(format!("Failed to finish compression: {}", e))
                })?;
                let compressed_size = compressed_data.len() as u64;
                (compressed_data, compressed_size)
            };

            #[cfg(not(feature = "flate2"))]
            let (compressed_data, compressed_size) = { (snapshot_bytes.to_vec(), original_size) };

            // Calculate checksum
            use sha2::{Digest, Sha256};
            let mut hasher = Sha256::new();
            hasher.update(&compressed_data);
            let checksum = format!("{:x}", hasher.finalize());

            // Record metrics (only if metrics feature is enabled)
            #[cfg(feature = "metrics")]
            {
                counter!("secreton_grpc_create_snapshot_requests").increment(1);
                counter!("secreton_raft_snapshots_created").increment(1);
                gauge!("secreton_raft_snapshot_size_bytes").set(original_size as f64);
            }

            let response = CreateSnapshotResponse {
                snapshot_id: snapshot_id.clone(),
                size_bytes: original_size,
                compressed_size_bytes: compressed_size,
                checksum,
                created_at: chrono::Utc::now().timestamp(),
            };

            info!("Snapshot {} created successfully via gRPC", snapshot_id);
            Ok(Response::new(response))
        }
    }

    #[allow(dead_code)]
    #[instrument(skip(self, _request))]
    async fn list_snapshots(
        &self,
        _request: Request<ListSnapshotsRequest>,
    ) -> Result<Response<ListSnapshotsResponse>, Status> {
        #[cfg(not(feature = "raft-consensus"))]
        {
            return Err(Status::unimplemented("Raft consensus feature not enabled"));
        }

        #[cfg(feature = "raft-consensus")]
        {
            info!("Listing snapshots via gRPC");

            // Get Raft storage backend
            let _raft_storage = self
                .storage
                .as_any()
                .downcast_ref::<secreton_storage::raft::RaftCluster>()
                .ok_or_else(|| {
                    error!("Storage backend is not a Raft cluster");
                    Status::failed_precondition("Raft cluster not configured")
                })?;

            // List snapshots from raft storage (stub implementation)
            let snapshots = Vec::new();

            // Record metrics (only if metrics feature is enabled)
            #[cfg(feature = "metrics")]
            {
                counter!("secreton_grpc_list_snapshots_requests").increment(1);
            }

            let response = ListSnapshotsResponse {
                snapshots,
                total: 0,
            };

            info!("Listed {} snapshots via gRPC", response.total);
            Ok(Response::new(response))
        }
    }

    #[allow(dead_code)]
    #[instrument(skip(self, request))]
    async fn restore_snapshot(
        &self,
        request: Request<RestoreSnapshotRequest>,
    ) -> Result<Response<RestoreSnapshotResponse>, Status> {
        #[cfg(not(feature = "raft-consensus"))]
        {
            let _ = request;
            return Err(Status::unimplemented("Raft consensus feature not enabled"));
        }

        #[cfg(feature = "raft-consensus")]
        {
            let req = request.into_inner();

            info!("Restoring from snapshot {} via gRPC", req.snapshot_id);

            // Validate snapshot ID
            if req.snapshot_id.is_empty() {
                return Err(Status::invalid_argument("Snapshot ID cannot be empty"));
            }

            if !req.snapshot_id.starts_with("snapshot-") {
                return Err(Status::invalid_argument(
                    "Invalid snapshot ID format. Must start with 'snapshot-'",
                ));
            }

            // Get Raft storage backend
            let raft_storage = self
                .storage
                .as_any()
                .downcast_ref::<secreton_storage::raft::RaftCluster>()
                .ok_or_else(|| {
                    error!("Storage backend is not a Raft cluster");
                    Status::failed_precondition("Raft cluster not configured")
                })?;

            // Check if this node is the leader
            if !raft_storage.is_leader().await {
                warn!("Snapshot restore attempted on non-leader node");
                return Err(Status::failed_precondition(
                    "Snapshots can only be restored on the leader node",
                ));
            }

            // Record metrics (only if metrics feature is enabled)
            #[cfg(feature = "metrics")]
            {
                counter!("secreton_grpc_restore_snapshot_requests").increment(1);
                counter!("secreton_raft_restores_attempted").increment(1);
            }

            warn!("Snapshot restore not yet fully implemented");
            Err(Status::unimplemented(
                "Snapshot restore not yet fully implemented",
            ))
        }
    }

    // Helper methods
    fn get_uptime_seconds(&self) -> i64 {
        use std::sync::OnceLock;
        static START_TIME: OnceLock<std::time::Instant> = OnceLock::new();
        let start = START_TIME.get_or_init(std::time::Instant::now);
        start.elapsed().as_secs() as i64
    }

    #[allow(dead_code)]
    async fn get_stats_by_resource_type(&self) -> std::collections::HashMap<String, i64> {
        // TODO: Implement count methods in StorageBackend trait
        // Get statistics grouped by resource type
        // let stats = std::collections::HashMap::new();
        // stats.insert("secret".to_string(), self.storage.count_secrets().await.unwrap_or(0) as i64);
        // stats.insert("key".to_string(), self.storage.count_keys().await.unwrap_or(0) as i64);
        // stats.insert("policy".to_string(), self.storage.count_policies().await.unwrap_or(0) as i64);
        std::collections::HashMap::new()
    }

    #[allow(dead_code)]
    async fn get_stats_by_namespace(&self) -> std::collections::HashMap<String, i64> {
        // TODO: Implement get_namespace_stats in StorageBackend trait
        // Get statistics grouped by namespace
        // self.storage.get_namespace_stats().await.unwrap_or_default()
        std::collections::HashMap::new()
    }
}
