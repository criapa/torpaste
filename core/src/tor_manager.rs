//! Tor Manager - Versão Correta (Tipagem Estrita)
//! Resolve o erro E0599 forçando a tipagem do DataStream.

use std::sync::{Arc, Mutex};
use log::{info, error};
use thiserror::Error;
use serde::{Deserialize, Serialize};

// Necessário para o método .next()
use futures::stream::StreamExt;

// Arti Imports
use arti_client::{TorClient, TorClientConfig, DataStream};
use arti_client::config::onion_service::OnionServiceConfigBuilder;
use tor_rtcompat::PreferredRuntime;
use tor_hsservice::RunningOnionService;

// Web Server Imports
use axum::{Router, routing::get};
use hyper::server::conn::Http;

// Importamos a Struct Compat diretamente para instanciar manualmente
use tokio_util::compat::Compat;

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
    #[error("Tor not initialized")]
    NotInitialized,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TorStatus {
    NotStarted,
    Bootstrapping(u8),
    Ready,
    Error(String),
}

impl Default for TorStatus {
    fn default() -> Self {
        Self::NotStarted
    }
}

pub struct TorManager {
    status: TorStatus,
    onion_address: Option<String>,
    tor_client: Option<TorClient<PreferredRuntime>>,
    // Mantém o serviço vivo.
    _onion_service: Arc<Mutex<Option<Arc<RunningOnionService>>>>,
}

impl TorManager {
    pub fn new() -> Self {
        Self {
            status: TorStatus::NotStarted,
            onion_address: None,
            tor_client: None,
            _onion_service: Arc::new(Mutex::new(None)),
        }
    }

    pub fn get_status(&self) -> &TorStatus {
        &self.status
    }

    pub fn get_onion_address(&self) -> Option<&String> {
        self.onion_address.as_ref()
    }

    pub fn set_status(&mut self, status: TorStatus) {
        self.status = status;
    }

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

    pub async fn bootstrap(&mut self) -> Result<(), TorError> {
        info!("Criando cliente Tor com Arti...");
        let config = TorClientConfig::default();
        
        let tor_client = TorClient::create_bootstrapped(config).await
            .map_err(|e| TorError::BootstrapFailed(e.to_string()))?;

        self.tor_client = Some(tor_client);
        self.status = TorStatus::Ready;
        info!("Tor pronto para uso.");
        Ok(())
    }

    pub async fn create_ephemeral_sharing_service(
        &mut self,
        permanent_address: String, 
    ) -> Result<String, TorError> {
        
        let tor_client = self.tor_client.as_ref()
            .ok_or(TorError::NotInitialized)?;

        let config = OnionServiceConfigBuilder::default()
            .nickname("ephemeral-share".parse().unwrap()) 
            .build()
            .map_err(|e| TorError::HiddenServiceCreation(e.to_string()))?;

        // Inicia o serviço
        let (running, mut request_stream) = tor_client.launch_onion_service(config)
            .map_err(|e| TorError::HiddenServiceCreation(e.to_string()))?;

        let onion_address = running.onion_name()
            .ok_or_else(|| TorError::HiddenServiceCreation("Endereço onion não disponível".into()))?
            .to_string();
            
        self.onion_address = Some(onion_address.clone());
        
        // Guarda o serviço para mantê-lo rodando
        *self._onion_service.lock().unwrap() = Some(running);

        let app = Router::new().route("/", get(move || async move {
            permanent_address
        }));

        info!("Serviço Onion iniciado em: http://{}", onion_address);

        // Loop de processamento
        tokio::spawn(async move {
            while let Some(stream_req) = request_stream.next().await {
                let app_clone = app.clone();
                
                tokio::spawn(async move {
                    // Tipagem Explícita: Garante que o compilador sabe que isso é um socket de dados
                    let accept_result: Result<DataStream, _> = stream_req.accept().await;

                    match accept_result {
                        Ok(arti_stream) => {
                            // CORREÇÃO DEFINITIVA:
                            // Em vez de usar .compat() (trait), usamos o construtor Compat::new().
                            // Isso força o wrapper sem depender de inferência de traits.
                            let tokio_stream = Compat::new(arti_stream);
                            
                            let service = tower::ServiceBuilder::new()
                                .service(app_clone);
                            
                            if let Err(err) = Http::new()
                                .serve_connection(tokio_stream, service)
                                .await 
                            {
                                error!("Erro HTTP/Tor: {}", err);
                            }
                        }
                        Err(e) => error!("Falha ao aceitar conexão Tor: {}", e),
                    }
                });
            }
            info!("Serviço onion encerrado.");
        });

        Ok(onion_address)
    }

    pub async fn connect_to_onion(
        &self,
        address: &str,
        port: u16,
    ) -> Result<DataStream, TorError> {
        let client = self.tor_client.as_ref().ok_or(TorError::NotInitialized)?;
        
        let addr_string = if address.ends_with(".onion") {
            address.to_string()
        } else {
            format!("{}.onion", address)
        };

        let stream = client.connect((addr_string.as_str(), port)).await
            .map_err(|e| TorError::ConnectionFailed(e.to_string()))?;

        Ok(stream)
    }

    pub fn shutdown_hidden_service(&mut self) {
        let mut service_guard = self._onion_service.lock().unwrap();
        *service_guard = None;
        self.onion_address = None;
        info!("Serviço oculto encerrado.");
    }
}
