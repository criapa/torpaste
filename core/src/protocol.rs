//! P2P Messaging Protocol
//! Responsável por definir as estruturas de dados e a serialização das mensagens.

use serde::{Deserialize, Serialize};
use thiserror::Error;
use rand::Rng;
use sodiumoxide::crypto::hash::sha256;
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use chrono;

// --- Erros do Protocolo ---

#[derive(Error, Debug)]
pub enum ProtocolError {
    #[error("Handshake failed: {0}")]
    HandshakeFailed(String),
    #[error("Invalid message format")]
    InvalidFormat,
    #[error("Encryption error: {0}")]
    EncryptionError(String),
}

// --- Estruturas de Dados ---

/// Tipos de mensagens suportadas pelo protocolo
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MessageType {
    /// Troca de chaves inicial
    Handshake,
    /// Mensagem de texto simples
    Text,
    /// Transferência de arquivo (metadados + conteúdo)
    File,
    /// Confirmação de recebimento
    Ack,
    /// Manter conexão viva (Heartbeat)
    KeepAlive,
    /// Notificação de desconexão
    Disconnect,
}

/// Estrutura principal da mensagem de chat
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    /// ID único da mensagem (UUID ou Hex aleatório)
    pub id: String,
    /// Tipo da mensagem
    pub msg_type: MessageType,
    /// Endereço .onion do remetente
    pub sender: String,
    /// Timestamp (UTC)
    pub timestamp: i64,
    /// Conteúdo da mensagem (Texto ou Payload Base64)
    pub content: String,
    /// Número de sequência para ordenação
    pub sequence: u64,
    /// Metadados opcionais se for um arquivo
    pub file_metadata: Option<FileMetadata>,
}

/// Metadados para transferência de arquivos
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileMetadata {
    pub name: String,
    pub size: u64,
    pub mime_type: String,
}

/// Mensagem de Handshake (Troca de Chaves)
/// Atualizado para suportar Forward Secrecy com chaves efêmeras.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandshakeMessage {
    /// Versão do protocolo
    pub version: u16,
    /// Chave Pública de Identidade (Longa duração, para autenticação)
    pub identity_pk: String,
    /// Chave Pública Efêmera (Sessão atual, para criptografia)
    pub ephemeral_pk: String,
    /// Nonce aleatório para evitar replay attacks
    pub nonce: String,
}

/// Envelope de transporte (Frame)
/// Envolve a mensagem serializada e adiciona um checksum para integridade básica.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolFrame {
    /// Payload (JSON da Message ou Handshake)
    pub payload: String,
    /// Checksum SHA256 do payload (Base64)
    pub checksum: String,
}

// --- Lógica do Protocolo ---

pub struct ChatProtocol {
    /// Versão atual do protocolo usada nesta instância
    version: u16,
    /// Contador local de sequência de mensagens enviadas
    sequence: u64,
}

impl ChatProtocol {
    pub fn new() -> Self {
        Self {
            version: 1,
            sequence: 0,
        }
    }

    /// Incrementa e retorna o próximo número de sequência
    pub fn next_sequence(&mut self) -> u64 {
        let seq = self.sequence;
        self.sequence += 1;
        seq
    }

    /// Cria a estrutura de handshake
    pub fn create_handshake(&self, identity_pk: &str, ephemeral_pk: &str, nonce: &str) -> HandshakeMessage {
        HandshakeMessage {
            version: self.version,
            identity_pk: identity_pk.to_string(),
            ephemeral_pk: ephemeral_pk.to_string(),
            nonce: nonce.to_string(),
        }
    }

    /// Serializa uma mensagem completa para JSON
    pub fn serialize_message(&self, msg: &Message) -> Result<String, ProtocolError> {
        serde_json::to_string(msg)
            .map_err(|_| ProtocolError::InvalidFormat)
    }

    /// Deserializa uma string JSON para uma mensagem
    pub fn deserialize_message(&self, data: &str) -> Result<Message, ProtocolError> {
        serde_json::from_str(data)
            .map_err(|_| ProtocolError::InvalidFormat)
    }

    /// Serializa o handshake para JSON
    pub fn serialize_handshake(&self, handshake: &HandshakeMessage) -> Result<String, ProtocolError> {
        serde_json::to_string(handshake)
            .map_err(|_| ProtocolError::InvalidFormat)
    }

    /// Deserializa o handshake de JSON
    pub fn deserialize_handshake(&self, data: &str) -> Result<HandshakeMessage, ProtocolError> {
        serde_json::from_str(data)
            .map_err(|_| ProtocolError::InvalidFormat)
    }

    /// Cria um Frame de protocolo (adiciona checksum)
    pub fn create_frame(&self, payload: String) -> ProtocolFrame {
        let checksum = BASE64.encode(sha256::hash(payload.as_bytes()).as_ref());
        ProtocolFrame {
            payload,
            checksum,
        }
    }

    /// Serializa o Frame para envio na rede
    pub fn serialize_frame(&self, frame: &ProtocolFrame) -> Result<String, ProtocolError> {
        serde_json::to_string(frame)
            .map_err(|_| ProtocolError::InvalidFormat)
    }

    /// Deserializa o Frame recebido da rede
    pub fn deserialize_frame(&self, data: &str) -> Result<ProtocolFrame, ProtocolError> {
        serde_json::from_str(data)
            .map_err(|_| ProtocolError::InvalidFormat)
    }

    /// Cria uma nova mensagem de texto pronta para envio
    pub fn create_text_message(
        &mut self,
        sender: &str,
        content: String,
    ) -> Message {
        let timestamp = chrono::Utc::now().timestamp();
        let sequence = self.next_sequence();
        let id = Self::generate_message_id();

        Message {
            id,
            msg_type: MessageType::Text,
            sender: sender.to_string(),
            timestamp,
            content,
            sequence,
            file_metadata: None,
        }
    }

    /// Gera um ID aleatório de 16 bytes (hex) para a mensagem
    fn generate_message_id() -> String {
        let mut rng = rand::thread_rng();
        let bytes: Vec<u8> = (0..16).map(|_| rng.gen()).collect();
        hex::encode(bytes)
    }
}

impl Default for ChatProtocol {
    fn default() -> Self {
        Self::new()
    }
}
