//! Tor Manager - agora com suporte a serviços onion efêmeros para compartilhamento

use std::net::SocketAddr;
use std::sync::Arc;
use log::{info, error};
use thiserror::Error;
use serde::{Deserialize, Serialize};
use futures::stream::StreamExt;

use arti_client::{TorClient, TorClientConfig};
use arti_client::onion::service::{OnionServiceConfig, RunningOnionService};
use tor_rtcompat::PreferredRuntime;
use tokio_socks::tcp::Socks5Stream;

use axum::{Router, response::IntoResponse};
use axum::routing::get;
use hyper::server::conn::Http;

#[derive(Error, Debug)]
pub enum TorError {
    #[error("Tor client creation failed: {0}")]
    ClientCreation(String),
    #[error("Tor bootstrap failed: {0}")]
    BootstrapFailed(String),
    #[error("Failed to create hidden service: {0}")]
    HiddenServiceCreation(String),
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),
    #[error("Invalid onion address: {0}")]
    InvalidOnionAddress(String),
    #[error("SOCKS5 proxy error: {0}")]
    SocksError(String),
    #[error("Tor not initialized")]
    NotInitialized,
}

/// Tor connection status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TorStatus {
    /// Tor is not started yet
    NotStarted,
    /// Tor is currently bootstrapping (percentage)
    Bootstrapping(u8),
    /// Tor is fully operational
    Ready,
    /// Tor encountered an error
    Error(String),
}

impl Default for TorStatus {
    fn default() -> Self {
        Self::NotStarted
    }
}

/// Tor Manager - Manages Tor client and hidden service using Arti
pub struct TorManager {
    status: TorStatus,
    /// Our hidden service address (.onion)
    onion_address: Option<String>,
    /// SOCKS5 proxy port (always localhost:9050 with Arti)
    socks_addr: Option<SocketAddr>,
    /// The Tor client instance
    tor_client: Option<Arc<TorClient<PreferredRuntime>>>,
    /// The onion service runner (kept alive)
    _onion_service: Option<Arc<RunningOnionService>>,
}

impl TorManager {
    pub fn new() -> Self {
        Self {
            status: TorStatus::NotStarted,
            onion_address: None,
            socks_addr: Some(SocketAddr::from(([127, 0, 0, 1], 9050))),
            tor_client: None,
            _onion_service: None,
        }
    }

    /// Get current status
    pub fn get_status(&self) -> &TorStatus {
        &self.status
    }

    /// Get our onion address
    pub fn get_onion_address(&self) -> Option<&String> {
        self.onion_address.as_ref()
    }

    /// Get SOCKS5 proxy address
    pub fn get_socks_addr(&self) -> Option<SocketAddr> {
        self.socks_addr
    }

    /// Set status
    pub fn set_status(&mut self, status: TorStatus) {
        self.status = status;
    }

    /// Validate onion address format (v3)
    pub fn validate_onion_address(address: &str) -> Result<(), TorError> {
        let addr = address.trim_end_matches(".onion");
        if addr.len() != 56 {
            return Err(TorError::InvalidOnionAddress(
                format!("Address must be 56 characters, got {}", addr.len())
            ));
        }
        for c in addr.chars() {
            if !c.is_ascii_lowercase() && !('2'..='7').contains(&c) {
                return Err(TorError::InvalidOnionAddress(
                    format!("Invalid character: {}", c)
                ));
            }
        }
        Ok(())
    }

    /// Bootstrap Tor (cria o cliente e aguarda prontidão)
    pub async fn bootstrap(&mut self) -> Result<(), TorError> {
        info!("Criando cliente Tor com Arti...");
        let config = TorClientConfig::default();
        let tor_client = TorClient::<PreferredRuntime>::create_bootstrapped(config).await
            .map_err(|e| TorError::BootstrapFailed(e.to_string()))?;

        self.tor_client = Some(Arc::new(tor_client));
        self.status = TorStatus::Ready;
        info!("Tor pronto para uso.");
        Ok(())
    }

    /// Create an ephemeral hidden service (the main chat service)
    pub async fn create_hidden_service(&mut self) -> Result<String, TorError> {
        let tor_client = self.tor_client.as_ref()
            .ok_or(TorError::NotInitialized)?;

        use rand::Rng;
        let mut rng = rand::thread_rng();
        let local_port = rng.gen_range(5000..9000);

        info!("Criando serviço onion principal na porta local {}", local_port);

        let config = OnionServiceConfig::builder()
            .ports(vec![local_port])
            .build()
            .map_err(|e| TorError::HiddenServiceCreation(e.to_string()))?;

        // launch_onion_service é async, retorna um future
        let (running, mut requests) = tor_client.launch_onion_service(config).await
            .map_err(|e| TorError::HiddenServiceCreation(e.to_string()))?;

        let onion_address = running.onion_address().to_string();
        self.onion_address = Some(onion_address.clone());
        self._onion_service = Some(Arc::new(running));

        // Processa requisições em background
        let client_clone = tor_client.clone();
        tokio::spawn(async move {
            while let Some(request) = requests.next().await {
                info!("Nova conexão recebida no serviço principal");
                if let Ok(mut stream) = request.accept().await {
                    // Aqui você passaria a stream para o protocolo de chat
                    drop(stream);
                }
            }
        });

        info!("Serviço oculto principal criado: {}", onion_address);
        Ok(onion_address)
    }

    /// Cria um serviço onion efêmero que serve o endereço permanente do usuário.
    /// Retorna o endereço onion temporário e um handle que pode ser usado para encerrar o serviço.
    pub async fn create_ephemeral_sharing_service(
        &self,
        permanent_address: &str,
    ) -> Result<(String, Arc<RunningOnionService>), TorError> {
        let tor_client = self.tor_client.as_ref()
            .ok_or(TorError::NotInitialized)?;

        use rand::Rng;
        let mut rng = rand::thread_rng();
        let local_port = rng.gen_range(5000..9000);

        info!("Criando serviço onion efêmero na porta local {}", local_port);

        let config = OnionServiceConfig::builder()
            .ports(vec![local_port])
            .build()
            .map_err(|e| TorError::HiddenServiceCreation(e.to_string()))?;

        let (running, mut requests) = tor_client.launch_onion_service(config).await
            .map_err(|e| TorError::HiddenServiceCreation(e.to_string()))?;

        let onion_address = running.onion_address().to_string();

        // Cria o servidor HTTP com axum
        let app = Router::new().route("/", get(|| async move {
            permanent_address.to_string()
        }));

        // Inicia o servidor na porta local usando hyper
        let server_addr = SocketAddr::from(([127, 0, 0, 1], local_port));
        tokio::spawn(async move {
            if let Err(e) = hyper::Server::bind(&server_addr)
                .serve(app.into_make_service())
                .await
            {
                eprintln!("Erro no servidor hyper: {}", e);
            }
        });

        // Processa requisições onion em background
        let running_clone = running.clone();
        tokio::spawn(async move {
            while let Some(request) = requests.next().await {
                info!("Nova requisição recebida no serviço onion efêmero");
                if let Ok(mut stream) = request.accept().await {
                    // O hyper já está cuidando do HTTP, então não precisamos fazer nada aqui
                    drop(stream);
                }
            }
        });

        info!("Serviço onion efêmero criado em: {}", onion_address);
        Ok((onion_address, Arc::new(running)))
    }

    /// Connect to a remote onion service via SOCKS5
    pub async fn connect_to_onion(
        &self,
        address: &str,
        port: u16,
    ) -> Result<tokio::net::TcpStream, TorError> {
        if self.status != TorStatus::Ready {
            return Err(TorError::NotInitialized);
        }
        Self::validate_onion_address(address)?;

        let socks_addr = self.socks_addr.ok_or(TorError::NotInitialized)?;

        info!("Conectando a {}:{} via SOCKS5...", address, port);

        let stream = Socks5Stream::<tokio::net::TcpStream>::connect(socks_addr, (address, port)).await
            .map_err(|e| TorError::SocksError(e.to_string()))?;

        Ok(stream.into_inner())
    }

    /// Check if an address is reachable
    pub async fn is_reachable(&self, address: &str) -> bool {
        if self.status != TorStatus::Ready || Self::validate_onion_address(address).is_err() {
            return false;
        }
        let socks_addr = match self.socks_addr {
            Some(a) => a,
            None => return false,
        };
        matches!(
            Socks5Stream::<tokio::net::TcpStream>::connect(socks_addr, (address, 1)).await,
            Ok(_)
        )
    }

    /// Shutdown the hidden service (drop the runner)
    pub fn shutdown_hidden_service(&mut self) {
        self._onion_service = None;
        self.onion_address = None;
        info!("Serviço oculto encerrado.");
    }
}

impl Default for TorManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_onion_validation_valid() {
        let addr = "abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890";
        assert!(TorManager::validate_onion_address(addr).is_ok());
    }

    #[test]
    fn test_onion_validation_with_suffix() {
        let addr = "abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890.onion";
        assert!(TorManager::validate_onion_address(addr).is_ok());
    }

    #[test]
    fn test_onion_validation_invalid_length() {
        let addr = "abc.onion";
        assert!(TorManager::validate_onion_address(addr).is_err());
    }

    #[test]
    fn test_onion_validation_invalid_chars() {
        let addr = "abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567891";
        assert!(TorManager::validate_onion_address(addr).is_err());
    }
}
