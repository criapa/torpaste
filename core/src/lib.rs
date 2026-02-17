//! TorChat-Paste Core Library
//! Anonymous P2P Messenger with Tor integration

// Declaração dos módulos filhos
// O compilador procurará por arquivos com esses nomes na pasta src/
pub mod tor_manager;
pub mod crypto;
pub mod storage;
// Estes módulos são opcionais dependendo se você já criou os arquivos.
// Se ainda não criou 'protocol.rs' ou 'config.rs', comente as linhas abaixo.
pub mod protocol;
pub mod config;

// Re-exportações (Torna mais fácil importar no main.rs)
// Ex: 'use torchat_paste_core::TorManager' ao invés de '...::tor_manager::TorManager'
pub use tor_manager::{TorManager, TorStatus, TorError};

// Assumindo que você tem essas structs definidas em crypto.rs e storage.rs.
// Se não tiver, o código não compilará até que elas existam.
pub use crypto::{Crypto, IdentityKeyPair, Fingerprint}; 
pub use storage::{SecureStorage, StoredContact};

// Imports do sistema
use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};

/// AppState: O estado global da aplicação.
///
/// Usamos Arc (Atomic Reference Counting) para permitir que múltiplos threads/tasks
/// acessem esses dados simultaneamente com segurança.
///
/// Usamos RwLock (Read-Write Lock) para permitir:
/// - Múltiplas leituras simultâneas (ex: listar contatos enquanto envia msg)
/// - Apenas uma escrita por vez (ex: inicializar Tor ou adicionar contato)
pub struct AppState {
    /// Gerenciador da conexão Tor (Mutable porque muda de estado: Start -> Ready)
    pub tor_manager: Arc<RwLock<TorManager>>,
    
    /// Utilitários de Criptografia (Geralmente imutável após setup)
    pub crypto: Arc<Crypto>,
    
    /// Armazenamento persistente (Banco de dados/Arquivo)
    pub storage: Arc<SecureStorage>,
    
    /// Lista de contatos em memória (Mutable)
    pub contacts: Arc<RwLock<HashMap<String, Contact>>>,
}

impl AppState {
    /// Cria uma nova instância do estado da aplicação
    pub fn new() -> Self {
        // Inicializa o TorManager (definido no tor_manager.rs)
        let tor_manager = TorManager::new();
        
        // Inicializa Crypto (definido no crypto.rs)
        // Se Crypto::new() não existir, ajuste conforme sua implementação
        let crypto = Crypto::new(); 

        // Inicializa Storage (definido no storage.rs)
        let storage = SecureStorage::new();

        Self {
            tor_manager: Arc::new(RwLock::new(tor_manager)),
            crypto: Arc::new(crypto),
            storage: Arc::new(storage),
            contacts: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

// Permite chamar AppState::default()
impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

/// Representação de um Contato
/// Implementa Serialize/Deserialize para poder ser salvo em disco ou enviado via rede
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contact {
    /// O endereço .onion do contato (chave primária)
    pub address: String,
    
    /// Apelido local para exibição
    pub nickname: String,
    
    /// Status online (não persistido em disco, apenas runtime)
    #[serde(skip)] // Não salva o status 'online' no banco de dados
    pub online: bool,
    
    /// Carimbo de data/hora da última vez visto
    pub last_seen: Option<i64>,
}

impl Contact {
    pub fn new(address: String, nickname: String) -> Self {
        Self {
            address,
            nickname,
            online: false,
            last_seen: None,
        }
    }
}

// --- Testes Unitários ---
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_contact_creation() {
        let onion = "abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890.onion".to_string();
        let contact = Contact::new(onion.clone(), "Test User".to_string());
        
        assert_eq!(contact.address, onion);
        assert_eq!(contact.nickname, "Test User");
        assert!(!contact.online); // Deve começar offline
    }

    #[tokio::test]
    async fn test_app_state_initialization() {
        let state = AppState::new();
        
        // Testa se conseguimos adquirir o lock de leitura do TorManager
        let manager = state.tor_manager.read().await;
        assert_eq!(*manager.get_status(), TorStatus::NotStarted);
    }
}
