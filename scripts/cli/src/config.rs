use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub type SimplConfig = Config;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub workspace: WorkspaceConfig,
    pub services: Vec<ServiceConfig>,
    pub build: BuildConfig,
    pub deploy: DeployConfig,
    pub docker: DockerConfig,
    pub k8s: K8sConfig,
    pub database: DatabaseConfig,
    pub monitoring: MonitoringConfig,
    pub security: SecurityConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceConfig {
    pub root: PathBuf,
    pub rust_version: String,
    pub edition: String,
    pub workspace_members: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceConfig {
    pub name: String,
    pub service_type: ServiceType,
    pub path: PathBuf,
    pub port: u16,
    pub dependencies: Vec<String>,
    pub environment: HashMap<String, String>,
    pub build: ServiceBuildConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceBuildConfig {
    pub dockerfile: Option<String>,
    pub registry: Option<String>,
    pub image_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ServiceType {
    Backend,
    Frontend,
    Shared,
    Infrastructure,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildConfig {
    pub target: String,
    pub profile: String,
    pub features: Vec<String>,
    pub parallel_jobs: usize,
    pub cache_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeployConfig {
    pub environments: HashMap<String, EnvironmentConfig>,
    pub default_environment: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentConfig {
    pub name: String,
    pub namespace: String,
    pub replicas: u32,
    pub resources: ResourceConfig,
    pub secrets: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceConfig {
    pub cpu_request: String,
    pub cpu_limit: String,
    pub memory_request: String,
    pub memory_limit: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DockerConfig {
    pub registry: String,
    pub base_images: HashMap<String, String>,
    pub build_args: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct K8sConfig {
    pub cluster: String,
    pub namespace: String,
    pub ingress_class: String,
    pub storage_class: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub postgres: DatabaseInstanceConfig,
    pub redis: DatabaseInstanceConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseInstanceConfig {
    pub host: String,
    pub port: u16,
    pub database: String,
    pub username: String,
    pub password_env: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringConfig {
    pub prometheus: bool,
    pub grafana: bool,
    pub loki: bool,
    pub tempo: bool,
    pub metrics_port: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    pub vault_enabled: bool,
    pub tls_enabled: bool,
    pub secrets_encryption: bool,
    pub rbac_enabled: bool,
}

impl Config {
    pub async fn load(config_path: Option<&str>) -> Result<Self> {
        let config_file = config_path
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("simpel.toml"));

        if config_file.exists() {
            let content = tokio::fs::read_to_string(&config_file).await?;
            let config: Config = toml::from_str(&content)?;
            Ok(config)
        } else {
            Ok(Self::default())
        }
    }

    pub async fn save(&self, config_path: Option<&str>) -> Result<()> {
        let config_file = config_path
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("simpel.toml"));

        let content = toml::to_string_pretty(self)?;
        tokio::fs::write(&config_file, content).await?;
        Ok(())
    }

    pub fn get_service(&self, name: &str) -> Option<&ServiceConfig> {
        self.services.iter().find(|s| s.name == name)
    }

    pub fn get_services_by_type(&self, service_type: ServiceType) -> Vec<&ServiceConfig> {
        self.services
            .iter()
            .filter(|s| std::mem::discriminant(&s.service_type) == std::mem::discriminant(&service_type))
            .collect()
    }

    pub fn workspace_root(&self) -> &Path {
        &self.workspace.root
    }
}

impl Default for Config {
    fn default() -> Self {
        let current_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        
        Self {
            workspace: WorkspaceConfig {
                root: current_dir,
                rust_version: "1.75".to_string(),
                edition: "2021".to_string(),
                workspace_members: vec![],
            },
            services: vec![],
            build: BuildConfig {
                target: "x86_64-unknown-linux-gnu".to_string(),
                profile: "dev".to_string(),
                features: vec![],
                parallel_jobs: std::thread::available_parallelism().map(|p| p.get()).unwrap_or(4),
                cache_enabled: true,
            },
            deploy: DeployConfig {
                environments: {
                    let mut envs = HashMap::new();
                    envs.insert("dev".to_string(), EnvironmentConfig {
                        name: "development".to_string(),
                        namespace: "simpelv2-dev".to_string(),
                        replicas: 1,
                        resources: ResourceConfig {
                            cpu_request: "100m".to_string(),
                            cpu_limit: "500m".to_string(),
                            memory_request: "128Mi".to_string(),
                            memory_limit: "512Mi".to_string(),
                        },
                        secrets: vec![],
                    });
                    envs.insert("prod".to_string(), EnvironmentConfig {
                        name: "production".to_string(),
                        namespace: "simpelv2".to_string(),
                        replicas: 3,
                        resources: ResourceConfig {
                            cpu_request: "200m".to_string(),
                            cpu_limit: "1000m".to_string(),
                            memory_request: "256Mi".to_string(),
                            memory_limit: "1Gi".to_string(),
                        },
                        secrets: vec![],
                    });
                    envs
                },
                default_environment: "dev".to_string(),
            },
            docker: DockerConfig {
                registry: "localhost:32000".to_string(),
                base_images: {
                    let mut images = HashMap::new();
                    images.insert("rust".to_string(), "rust:1.75-slim".to_string());
                    images.insert("runtime".to_string(), "debian:bookworm-slim".to_string());
                    images.insert("postgres".to_string(), "postgres:15-alpine".to_string());
                    images.insert("redis".to_string(), "redis:7-alpine".to_string());
                    images
                },
                build_args: HashMap::new(),
            },
            k8s: K8sConfig {
                cluster: "microk8s".to_string(),
                namespace: "simpelv2".to_string(),
                ingress_class: "nginx".to_string(),
                storage_class: "microk8s-hostpath".to_string(),
            },
            database: DatabaseConfig {
                postgres: DatabaseInstanceConfig {
                    host: "postgres".to_string(),
                    port: 5432,
                    database: "simpelv2".to_string(),
                    username: "simpelv2_user".to_string(),
                    password_env: "POSTGRES_PASSWORD".to_string(),
                },
                redis: DatabaseInstanceConfig {
                    host: "redis".to_string(),
                    port: 6379,
                    database: "0".to_string(),
                    username: "".to_string(),
                    password_env: "REDIS_PASSWORD".to_string(),
                },
            },
            monitoring: MonitoringConfig {
                prometheus: true,
                grafana: true,
                loki: true,
                tempo: true,
                metrics_port: 9090,
            },
            security: SecurityConfig {
                vault_enabled: true,
                tls_enabled: true,
                secrets_encryption: true,
                rbac_enabled: true,
            },
        }
    }
}
