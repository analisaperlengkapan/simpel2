//! Secreton Engine Agent
//!
//! Auto-authentication, token renewal, and template rendering agent for Secreton engine.

pub mod auth;
pub mod config;
pub mod health;
pub mod sink;
pub mod template;

pub use config::AgentConfig;

use anyhow::Result;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{error, info, warn};

/// Main agent structure
pub struct SecretonAgent {
    config: AgentConfig,
    token: Arc<RwLock<Option<String>>>,
    http_client: reqwest::Client,
    shutdown_tx: Option<tokio::sync::broadcast::Sender<()>>,
}

impl SecretonAgent {
    /// Create new agent
    pub fn new(config: AgentConfig) -> Result<Self> {
        let http_client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()?;

        Ok(Self {
            config,
            token: Arc::new(RwLock::new(None)),
            http_client,
            shutdown_tx: None,
        })
    }

    /// Start the agent
    pub async fn start(&mut self) -> Result<()> {
        info!("Starting Secreton Agent");

        let (shutdown_tx, mut shutdown_rx) = tokio::sync::broadcast::channel(1);
        self.shutdown_tx = Some(shutdown_tx);

        // Initial authentication
        self.authenticate().await?;

        // Start token renewal task
        let token_renewal = self.start_token_renewal(shutdown_rx.resubscribe());

        // Start template rendering task
        let template_rendering = self.start_template_rendering(shutdown_rx.resubscribe());

        // Start health server if configured
        let health_server = self
            .config
            .health_port
            .map(|port| self.start_health_server(port, shutdown_rx.resubscribe()));

        // Wait for shutdown
        tokio::select! {
            _ = token_renewal => warn!("Token renewal stopped"),
            _ = template_rendering => warn!("Template rendering stopped"),
            _ = async {
                if let Some(server) = health_server {
                    server.await
                } else {
                    std::future::pending().await
                }
            } => warn!("Health server stopped"),
            _ = shutdown_rx.recv() => info!("Shutdown signal received"),
            _ = tokio::signal::ctrl_c() => info!("Ctrl+C received"),
        }

        self.shutdown().await
    }

    /// Authenticate with engine
    async fn authenticate(&self) -> Result<()> {
        info!("Authenticating with Secreton engine");

        let token = auth::authenticate(
            &self.http_client,
            &self.config.server_url,
            &self.config.auth_method,
            &self.config.auth_config,
        )
        .await?;

        *self.token.write().await = Some(token.clone());

        // Write token to sink if configured
        if let Some(ref sink_config) = self.config.sink {
            sink::write_token(&token, sink_config).await?;
        }

        info!("Authentication successful");
        Ok(())
    }

    /// Start token renewal task
    async fn start_token_renewal(
        &self,
        mut shutdown_rx: tokio::sync::broadcast::Receiver<()>,
    ) -> Result<()> {
        let token = Arc::clone(&self.token);
        let http_client = self.http_client.clone();
        let server_url = self.config.server_url.clone();
        let renewal_interval = self.config.token_renewal_interval();

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(renewal_interval);

            loop {
                tokio::select! {
                    _ = interval.tick() => {
                        if let Some(ref current_token) = *token.read().await {
                            match auth::renew_token(&http_client, &server_url, current_token).await {
                                Ok(new_token) => {
                                    *token.write().await = Some(new_token);
                                    info!("Token renewed successfully");
                                }
                                Err(e) => {
                                    error!("Failed to renew token: {}", e);
                                }
                            }
                        }
                    }
                    _ = shutdown_rx.recv() => {
                        info!("Token renewal task shutting down");
                        break;
                    }
                }
            }
        });

        Ok(())
    }

    /// Start template rendering task
    async fn start_template_rendering(
        &self,
        mut shutdown_rx: tokio::sync::broadcast::Receiver<()>,
    ) -> Result<()> {
        let token = Arc::clone(&self.token);
        let http_client = self.http_client.clone();
        let server_url = self.config.server_url.clone();
        let templates = self.config.templates.clone();
        let render_interval = self.config.template_interval();

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(render_interval);

            loop {
                tokio::select! {
                    _ = interval.tick() => {
                        if let Some(ref current_token) = *token.read().await {
                            for template_config in &templates {
                                match template::render_template(
                                    &http_client,
                                    &server_url,
                                    current_token,
                                    template_config,
                                )
                                .await
                                {
                                    Ok(_) => {
                                        info!("Template rendered: {}", template_config.dest);
                                    }
                                    Err(e) => {
                                        error!("Failed to render template {}: {}", template_config.dest, e);
                                    }
                                }
                            }
                        }
                    }
                    _ = shutdown_rx.recv() => {
                        info!("Template rendering task shutting down");
                        break;
                    }
                }
            }
        });

        Ok(())
    }

    /// Start health server
    async fn start_health_server(
        &self,
        port: u16,
        shutdown_rx: tokio::sync::broadcast::Receiver<()>,
    ) -> Result<()> {
        let token = Arc::clone(&self.token);

        tokio::spawn(async move {
            if let Err(e) = health::start_server(port, token, shutdown_rx).await {
                error!("Health server error: {}", e);
            }
        });

        Ok(())
    }

    /// Shutdown agent
    async fn shutdown(&self) -> Result<()> {
        info!("Shutting down Secreton Agent");

        if let Some(tx) = &self.shutdown_tx {
            let _ = tx.send(());
        }

        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        info!("Secreton Agent stopped");
        Ok(())
    }
}
