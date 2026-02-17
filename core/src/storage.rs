//! Secure Storage - Agora com identidade criptografada em repouso

use std::fs;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use log::{info, error};
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};

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

/// Identidade armazenada de forma segura (criptografada em disco)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecureIdentity {
    /// Fingerprint da chave pública (para verificação)
    pub fingerprint: Fingerprint,
    /// Dados criptografados: contém o IdentityKeyPair em formato JSON
    pub encrypted_data: String, // base64
}

/// Contact armazenado localmente
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredContact {
    pub address: String,
    pub nickname: String,
    pub fingerprint: Fingerprint,
    pub added_at: i64,
}

/// Gerenciador de armazenamento seguro
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

    /// Verifica se a identidade existe (o arquivo)
    pub fn has_identity(&self) -> bool {
        self.storage_dir.join("identity.enc").exists()
    }

    /// Cria uma nova identidade e a salva criptografada com a senha
    pub fn create_identity(&self, password: &str) -> Result<Fingerprint, StorageError> {
        let keypair = crypto::Crypto::generate_identity();
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

        let path = self.storage_dir.join("identity.enc");
        let content = serde_json::to_string_pretty(&secure_id)
            .map_err(|e| StorageError::SerializationError(e.to_string()))?;

        fs::write(&path, content)
            .map_err(|e| StorageError::IoError(e.to_string()))?;

        #[cfg(unix)]
        Self::set_permissions_unix(&path);

        info!("New identity created with fingerprint: {}", fingerprint.formatted());
        Ok(fingerprint)
    }

    /// Carrega a identidade (requer senha)
    pub fn load_identity(&self, password: &str) -> Result<IdentityKeyPair, StorageError> {
        let path = self.storage_dir.join("identity.enc");
        if !path.exists() {
            return Err(StorageError::IdentityNotFound);
        }

        let content = fs::read_to_string(&path)
            .map_err(|e| StorageError::IoError(e.to_string()))?;
        let secure_id: SecureIdentity = serde_json::from_str(&content)
            .map_err(|e| StorageError::SerializationError(e.to_string()))?;

        let encrypted = BASE64.decode(&secure_id.encrypted_data)
            .map_err(|e| StorageError::EncryptionError(e.to_string()))?;

        let plain = crypto::Crypto::decrypt_with_password(&encrypted, password)
            .map_err(|_| StorageError::InvalidPassword)?;

        let keypair: IdentityKeyPair = serde_json::from_slice(&plain)
            .map_err(|e| StorageError::SerializationError(e.to_string()))?;

        // Verifica se o fingerprint corresponde (integridade)
        let pk_bytes = BASE64.decode(&keypair.public_key)
            .map_err(|e| StorageError::EncryptionError(e.to_string()))?;
        let pk = crypto::PublicKey::from_slice(&pk_bytes)
            .ok_or(StorageError::EncryptionError("Invalid public key".to_string()))?;
        let fp = crypto::Fingerprint::from_public_key(&pk);
        if fp != secure_id.fingerprint {
            return Err(StorageError::FingerprintMismatch);
        }

        Ok(keypair)
    }

    /// Altera a senha da identidade
    pub fn change_password(&self, old_password: &str, new_password: &str) -> Result<(), StorageError> {
        let keypair = self.load_identity(old_password)?;
        self.create_identity_with_keypair(&keypair, new_password)?;
        Ok(())
    }

    /// Salva uma identidade existente com nova senha (usado internamente)
    fn create_identity_with_keypair(&self, keypair: &IdentityKeyPair, password: &str) -> Result<Fingerprint, StorageError> {
        let pk_bytes = BASE64.decode(&keypair.public_key)
            .map_err(|e| StorageError::EncryptionError(e.to_string()))?;
        let pk = crypto::PublicKey::from_slice(&pk_bytes)
            .ok_or(StorageError::EncryptionError("Invalid public key".to_string()))?;
        let fingerprint = crypto::Fingerprint::from_public_key(&pk);

        let plain = serde_json::to_string(keypair)
            .map_err(|e| StorageError::SerializationError(e.to_string()))?;

        let encrypted = crypto::Crypto::encrypt_with_password(plain.as_bytes(), password)
            .map_err(|e| StorageError::EncryptionError(e.to_string()))?;

        let secure_id = SecureIdentity {
            fingerprint: fingerprint.clone(),
            encrypted_data: BASE64.encode(&encrypted),
        };

        let path = self.storage_dir.join("identity.enc");
        let content = serde_json::to_string_pretty(&secure_id)
            .map_err(|e| StorageError::SerializationError(e.to_string()))?;

        fs::write(&path, content)
            .map_err(|e| StorageError::IoError(e.to_string()))?;

        #[cfg(unix)]
        Self::set_permissions_unix(&path);

        Ok(fingerprint)
    }

    /// Carrega a lista de contatos
    pub fn load_contacts(&self) -> Result<Vec<StoredContact>, StorageError> {
        let path = self.storage_dir.join("contacts.json");
        if !path.exists() {
            return Ok(Vec::new());
        }
        let content = fs::read_to_string(&path)
            .map_err(|e| StorageError::IoError(e.to_string()))?;
        let contacts: Vec<StoredContact> = serde_json::from_str(&content)
            .map_err(|e| StorageError::SerializationError(e.to_string()))?;
        Ok(contacts)
    }

    /// Salva a lista de contatos
    pub fn save_contacts(&self, contacts: &[StoredContact]) -> Result<(), StorageError> {
        let path = self.storage_dir.join("contacts.json");
        let content = serde_json::to_string_pretty(contacts)
            .map_err(|e| StorageError::SerializationError(e.to_string()))?;
        fs::write(&path, content)
            .map_err(|e| StorageError::IoError(e.to_string()))?;
        Ok(())
    }

    /// Adiciona um contato (com fingerprint)
    pub fn add_contact(&self, address: &str, nickname: &str, fingerprint: Fingerprint) -> Result<(), StorageError> {
        let mut contacts = self.load_contacts()?;
        if contacts.iter().any(|c| c.address == address) {
            return Ok(()); // já existe
        }
        let contact = StoredContact {
            address: address.to_string(),
            nickname: nickname.to_string(),
            fingerprint,
            added_at: chrono::Utc::now().timestamp(),
        };
        contacts.push(contact);
        self.save_contacts(&contacts)?;
        info!("Contact added: {} ({})", address, nickname);
        Ok(())
    }

    /// Remove um contato
    pub fn remove_contact(&self, address: &str) -> Result<(), StorageError> {
        let mut contacts = self.load_contacts()?;
        contacts.retain(|c| c.address != address);
        self.save_contacts(&contacts)?;
        info!("Contact removed: {}", address);
        Ok(())
    }

    /// Busca um contato pelo endereço
    pub fn find_contact(&self, address: &str) -> Option<StoredContact> {
        self.load_contacts().ok()?.into_iter().find(|c| c.address == address)
    }

    /// Wipe all data (secure delete)
    pub fn wipe_all(&self) -> Result<(), StorageError> {
        let identity_path = self.storage_dir.join("identity.enc");
        if identity_path.exists() {
            // Sobrescreve com zeros antes de deletar (tentativa)
            if let Ok(mut file) = fs::OpenOptions::new().write(true).open(&identity_path) {
                use std::io::Write;
                let _ = file.write_all(&[0u8; 4096]);
            }
            fs::remove_file(&identity_path)
                .map_err(|e| StorageError::IoError(e.to_string()))?;
        }

        let contacts_path = self.storage_dir.join("contacts.json");
        if contacts_path.exists() {
            fs::remove_file(&contacts_path)
                .map_err(|e| StorageError::IoError(e.to_string()))?;
        }

        info!("All secure data wiped");
        Ok(())
    }

    #[cfg(unix)]
    fn set_permissions_unix(path: &std::path::Path) {
        use std::fs;
        use std::os::unix::fs::PermissionsExt;
        if let Ok(metadata) = fs::metadata(path) {
            let mut perms = metadata.permissions();
            perms.set_mode(0o600);
            let _ = fs::set_permissions(path, perms);
        }
    }
}

impl Default for SecureStorage {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identity_creation_and_load() {
        let storage = SecureStorage::new();
        let password = "test123";

        // Cria identidade
        let fp = storage.create_identity(password).unwrap();
        assert!(storage.has_identity());

        // Carrega com senha correta
        let keypair = storage.load_identity(password).unwrap();
        assert_eq!(keypair.public_key.len(), 44); // base64 de 32 bytes

        // Carrega com senha errada
        assert!(storage.load_identity("wrong").is_err());

        // Verifica fingerprint
        let pk_bytes = BASE64.decode(&keypair.public_key).unwrap();
        let pk = crypto::PublicKey::from_slice(&pk_bytes).unwrap();
        assert!(fp.verify(&pk));

        // Cleanup
        storage.wipe_all().unwrap();
        assert!(!storage.has_identity());
    }
}
