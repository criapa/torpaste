//! Secure Storage
//! Contatos e Identidade agora são 100% criptografados em disco.

use std::fs;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use log::{info, error};
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

use crate::crypto::{self, Fingerprint, IdentityKeyPair};

#[derive(Error, Debug)]
pub enum StorageError {
    #[error("IO error: {0}")]
    IoError(String),
    #[error("Serialization error: {0}")]
    SerializationError(String),
    #[error("Encryption error: {0}")]
    EncryptionError(String),
    #[error("Identity not found")]
    IdentityNotFound,
    #[error("Invalid password")]
    InvalidPassword,
    #[error("Fingerprint mismatch")]
    FingerprintMismatch,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecureIdentity {
    pub fingerprint: Fingerprint,
    pub encrypted_data: String, 
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredContact {
    pub address: String,
    pub nickname: String,
    pub fingerprint: Fingerprint,
    pub added_at: i64,
}

/// Estrutura auxiliar para salvar o arquivo de contatos criptografado
#[derive(Serialize, Deserialize)]
struct EncryptedContactsFile {
    /// O ciphertext aqui contém [Salt + Nonce + Ciphertext]
    blob: String, 
}

pub struct SecureStorage {
    storage_dir: PathBuf,
}

impl SecureStorage {
    pub fn new() -> Self {
        let storage_dir = dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("torchat_paste");

        if let Err(e) = fs::create_dir_all(&storage_dir) {
            error!("Failed to create storage directory: {}", e);
        }

        Self { storage_dir }
    }

    pub fn has_identity(&self) -> bool {
        self.storage_dir.join("identity.enc").exists()
    }

    pub fn create_identity(&self, password: &str) -> Result<Fingerprint, StorageError> {
        let keypair = crypto::Crypto::generate_identity();
        
        // Fingerprint é público, pode ficar visível no arquivo para verificação rápida
        let fingerprint = {
            let pk_bytes = BASE64.decode(&keypair.public_key)
                .map_err(|e| StorageError::EncryptionError(e.to_string()))?;
            let pk = crypto::PublicKey::from_slice(&pk_bytes)
                .ok_or(StorageError::EncryptionError("Invalid public key".to_string()))?;
            crypto::Fingerprint::from_public_key(&pk)
        };

        let plain = serde_json::to_string(&keypair)
            .map_err(|e| StorageError::SerializationError(e.to_string()))?;

        let encrypted = crypto::Crypto::encrypt_with_password(plain.as_bytes(), password)
            .map_err(|e| StorageError::EncryptionError(e.to_string()))?;

        let secure_id = SecureIdentity {
            fingerprint: fingerprint.clone(),
            encrypted_data: BASE64.encode(&encrypted),
        };

        self.write_secure_file("identity.enc", &secure_id)?;
        info!("New identity created: {}", fingerprint.formatted());
        Ok(fingerprint)
    }

    pub fn load_identity(&self, password: &str) -> Result<IdentityKeyPair, StorageError> {
        let secure_id: SecureIdentity = self.read_secure_file("identity.enc")?;

        let encrypted = BASE64.decode(&secure_id.encrypted_data)
            .map_err(|e| StorageError::EncryptionError(e.to_string()))?;

        let plain = crypto::Crypto::decrypt_with_password(&encrypted, password)
            .map_err(|_| StorageError::InvalidPassword)?;

        let keypair: IdentityKeyPair = serde_json::from_slice(&plain)
            .map_err(|e| StorageError::SerializationError(e.to_string()))?;

        // Integridade check
        let pk_bytes = BASE64.decode(&keypair.public_key).unwrap();
        let pk = crypto::PublicKey::from_slice(&pk_bytes).unwrap();
        if crypto::Fingerprint::from_public_key(&pk) != secure_id.fingerprint {
            return Err(StorageError::FingerprintMismatch);
        }

        Ok(keypair)
    }

    /// Carrega contatos descriptografando com a senha
    pub fn load_contacts(&self, password: &str) -> Result<Vec<StoredContact>, StorageError> {
        let path = self.storage_dir.join("contacts.enc");
        if !path.exists() {
            return Ok(Vec::new());
        }

        let content = fs::read_to_string(&path)
            .map_err(|e| StorageError::IoError(e.to_string()))?;
        
        let enc_file: EncryptedContactsFile = serde_json::from_str(&content)
            .map_err(|e| StorageError::SerializationError(e.to_string()))?;

        let encrypted_blob = BASE64.decode(&enc_file.blob)
            .map_err(|e| StorageError::EncryptionError(e.to_string()))?;

        let plain = crypto::Crypto::decrypt_with_password(&encrypted_blob, password)
            .map_err(|_| StorageError::InvalidPassword)?;

        let contacts: Vec<StoredContact> = serde_json::from_slice(&plain)
            .map_err(|e| StorageError::SerializationError(e.to_string()))?;

        Ok(contacts)
    }

    /// Salva contatos criptografados
    pub fn save_contacts(&self, contacts: &[StoredContact], password: &str) -> Result<(), StorageError> {
        let json_plain = serde_json::to_string(contacts)
            .map_err(|e| StorageError::SerializationError(e.to_string()))?;

        let encrypted_blob = crypto::Crypto::encrypt_with_password(json_plain.as_bytes(), password)
            .map_err(|e| StorageError::EncryptionError(e.to_string()))?;

        let enc_file = EncryptedContactsFile {
            blob: BASE64.encode(&encrypted_blob),
        };

        let path = self.storage_dir.join("contacts.enc");
        let content = serde_json::to_string_pretty(&enc_file)
            .map_err(|e| StorageError::SerializationError(e.to_string()))?;

        fs::write(&path, content).map_err(|e| StorageError::IoError(e.to_string()))?;
        #[cfg(unix)] Self::set_permissions_unix(&path);
        
        Ok(())
    }

    pub fn add_contact(&self, address: &str, nickname: &str, fingerprint: Fingerprint, password: &str) -> Result<(), StorageError> {
        let mut contacts = self.load_contacts(password)?;
        
        if contacts.iter().any(|c| c.address == address) {
            return Ok(());
        }
        
        contacts.push(StoredContact {
            address: address.to_string(),
            nickname: nickname.to_string(),
            fingerprint,
            added_at: chrono::Utc::now().timestamp(),
        });
        
        self.save_contacts(&contacts, password)?;
        info!("Contact added securely.");
        Ok(())
    }

    pub fn wipe_all(&self) -> Result<(), StorageError> {
        let files = ["identity.enc", "contacts.enc"];
        for f in files {
            let path = self.storage_dir.join(f);
            if path.exists() {
                // Overwrite secure wipe attempt
                if let Ok(mut file) = fs::OpenOptions::new().write(true).open(&path) {
                    use std::io::Write;
                    let _ = file.write_all(&[0u8; 1024]);
                }
                fs::remove_file(&path).map_err(|e| StorageError::IoError(e.to_string()))?;
            }
        }
        info!("Data wiped.");
        Ok(())
    }

    // --- Helpers ---
    fn write_secure_file<T: Serialize>(&self, filename: &str, data: &T) -> Result<(), StorageError> {
        let path = self.storage_dir.join(filename);
        let content = serde_json::to_string_pretty(data)
            .map_err(|e| StorageError::SerializationError(e.to_string()))?;
        fs::write(&path, content).map_err(|e| StorageError::IoError(e.to_string()))?;
        #[cfg(unix)] Self::set_permissions_unix(&path);
        Ok(())
    }

    fn read_secure_file<T: serde::de::DeserializeOwned>(&self, filename: &str) -> Result<T, StorageError> {
        let path = self.storage_dir.join(filename);
        if !path.exists() { return Err(StorageError::IdentityNotFound); }
        let content = fs::read_to_string(&path).map_err(|e| StorageError::IoError(e.to_string()))?;
        serde_json::from_str(&content).map_err(|e| StorageError::SerializationError(e.to_string()))
    }

    #[cfg(unix)]
    fn set_permissions_unix(path: &std::path::Path) {
        if let Ok(metadata) = fs::metadata(path) {
            let mut perms = metadata.permissions();
            perms.set_mode(0o600); 
            let _ = fs::set_permissions(path, perms);
        }
    }
}

impl Default for SecureStorage { fn default() -> Self { Self::new() } }
