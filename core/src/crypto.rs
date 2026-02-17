//! Cryptographic operations using libsodium
//! Versão revisada com segurança aprimorada e tipos corretos.

use sodiumoxide::crypto::{
    kx, secretbox, pwhash, hash::sha256,
};
use sodiumoxide::utils::memzero;
use sodiumoxide::randombytes::randombytes;
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use log::error;

// Re-exportações para facilitar o uso
pub use sodiumoxide::crypto::kx::{PublicKey, SecretKey, SessionKey};
pub use sodiumoxide::crypto::secretbox::{Key as SecretBoxKey, Nonce as SecretBoxNonce};

#[derive(Error, Debug)]
pub enum CryptoError {
    #[error("Key generation failed")]
    KeyGenerationFailed,
    #[error("Encryption failed")]
    EncryptionFailed,
    #[error("Decryption failed")]
    DecryptionFailed,
    #[error("Invalid key length")]
    InvalidKeyLength,
    #[error("Base64 decode error: {0}")]
    Base64Error(String),
    #[error("Key exchange error")]
    KeyExchangeError,
    #[error("Invalid fingerprint")]
    InvalidFingerprint,
}

/// Par de chaves para identidade (X25519)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct IdentityKeyPair {
    pub public_key: String,   // base64
    pub secret_key: String,   // base64 (deve ser mantido em segredo)
}

/// Chaves de sessão derivadas do key exchange (rx e tx)
#[derive(Clone, Debug)]
pub struct SessionKeys {
    pub rx: SessionKey,
    pub tx: SessionKey,
}

/// Representação amigável da impressão digital (fingerprint) de uma chave pública
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Fingerprint(String);

impl Fingerprint {
    /// Gera fingerprint a partir de uma chave pública (hash SHA-256 + base64 truncado)
    pub fn from_public_key(pk: &PublicKey) -> Self {
        let hash = sha256::hash(pk.as_ref());
        let b64 = BASE64.encode(&hash[..8]); // primeiros 8 bytes (64 bits) são suficientes
        Fingerprint(b64)
    }

    /// Exibe o fingerprint em formato legível (ex: ABCD-EFGH-IJKL-MNOP)
    pub fn formatted(&self) -> String {
        let raw = &self.0;
        let mut result = String::new();
        for (i, ch) in raw.chars().enumerate() {
            if i > 0 && i % 4 == 0 {
                result.push('-');
            }
            result.push(ch);
        }
        result
    }

    /// Verifica se o fingerprint corresponde a uma chave pública
    pub fn verify(&self, pk: &PublicKey) -> bool {
        *self == Fingerprint::from_public_key(pk)
    }

    /// Cria um fingerprint a partir de uma string (já sem formatação)
    pub fn new(inner: String) -> Self {
        Fingerprint(inner)
    }
}

/// Mensagem assinada (apenas ciphertext + nonce)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EncryptedMessage {
    pub nonce: String,       // base64
    pub ciphertext: String,  // base64
}

/// Estrutura principal de operações criptográficas (stateless)
pub struct Crypto;

impl Crypto {
    /// Inicializa a biblioteca (deve ser chamada uma vez)
    pub fn init() {
        if sodiumoxide::init().is_err() {
            error!("Failed to initialize sodiumoxide");
        }
    }

    /// Cria uma nova instância (chama init implicitamente)
    pub fn new() -> Self {
        Self::init();
        Self
    }

    /// Gera um novo par de chaves de identidade (X25519)
    pub fn generate_identity() -> IdentityKeyPair {
        let (pk, sk) = kx::gen_keypair();
        IdentityKeyPair {
            public_key: BASE64.encode(pk.as_ref()),
            secret_key: BASE64.encode(sk.as_ref()),
        }
    }

    /// Deriva as chaves de sessão do lado do cliente (initiator)
    pub fn client_session_keys(
        client_pk: &PublicKey,
        client_sk: &SecretKey,
        server_pk: &PublicKey,
    ) -> Result<SessionKeys, CryptoError> {
        kx::client_session_keys(client_pk, client_sk, server_pk)
            .map(|(rx, tx)| SessionKeys { rx, tx })
            .map_err(|_| CryptoError::KeyExchangeError)
    }

    /// Deriva as chaves de sessão do lado do servidor (responder)
    pub fn server_session_keys(
        server_pk: &PublicKey,
        server_sk: &SecretKey,
        client_pk: &PublicKey,
    ) -> Result<SessionKeys, CryptoError> {
        kx::server_session_keys(server_pk, server_sk, client_pk)
            .map(|(rx, tx)| SessionKeys { rx, tx })
            .map_err(|_| CryptoError::KeyExchangeError)
    }

    /// Criptografa uma mensagem usando a chave de sessão (rx ou tx) e um nonce aleatório
    pub fn encrypt(message: &[u8], key: &SessionKey) -> EncryptedMessage {
        let nonce = secretbox::gen_nonce();
        // Converte SessionKey para secretbox::Key
        let key_box = SecretBoxKey::from_slice(key.as_ref()).expect("SessionKey has correct length");
        let ciphertext = secretbox::seal(message, &nonce, &key_box);
        EncryptedMessage {
            nonce: BASE64.encode(nonce.as_ref()),
            ciphertext: BASE64.encode(&ciphertext),
        }
    }

    /// Descriptografa uma mensagem usando a chave de sessão correspondente
    pub fn decrypt(enc: &EncryptedMessage, key: &SessionKey) -> Result<Vec<u8>, CryptoError> {
        let nonce = SecretBoxNonce::from_slice(
            &BASE64.decode(&enc.nonce).map_err(|e| CryptoError::Base64Error(e.to_string()))?
        ).ok_or(CryptoError::InvalidKeyLength)?;

        let ciphertext = BASE64.decode(&enc.ciphertext)
            .map_err(|e| CryptoError::Base64Error(e.to_string()))?;

        let key_box = SecretBoxKey::from_slice(key.as_ref()).expect("SessionKey has correct length");
        secretbox::open(&ciphertext, &nonce, &key_box)
            .map_err(|_| CryptoError::DecryptionFailed)
    }

    /// Gera bytes aleatórios
    pub fn random_bytes(len: usize) -> Vec<u8> {
        randombytes(len)
    }

    /// Hash SHA-256 de dados (retorna base64)
    pub fn hash(data: &[u8]) -> String {
        BASE64.encode(sha256::hash(data).as_ref())
    }

    /// Limpa memória de forma segura
    pub fn secure_wipe(data: &mut [u8]) {
        memzero(data);
    }

    /// Criptografa dados com uma chave derivada de senha (usado para armazenar identidade)
    pub fn encrypt_with_password(data: &[u8], password: &str) -> Result<Vec<u8>, CryptoError> {
        // Gera salt aleatório de 16 bytes
        let salt_bytes = randombytes(16);
        let salt = pwhash::argon2id13::Salt::from_slice(&salt_bytes)
            .ok_or(CryptoError::EncryptionFailed)?;

        // Deriva chave de 32 bytes usando Argon2id (parâmetros interativos)
        let mut key = [0u8; secretbox::KEYBYTES];
        pwhash::argon2id13::derive_key(
            &mut key,
            password.as_bytes(),
            &salt,
            pwhash::argon2id13::OPSLIMIT_INTERACTIVE,
            pwhash::argon2id13::MEMLIMIT_INTERACTIVE,
        ).map_err(|_| CryptoError::EncryptionFailed)?;

        let key_box = secretbox::Key::from_slice(&key).unwrap();

        // Gera nonce e cifra
        let nonce = secretbox::gen_nonce();
        let ciphertext = secretbox::seal(data, &nonce, &key_box);

        // Empacota: salt (16) + nonce (24) + ciphertext
        let mut out = Vec::with_capacity(16 + 24 + ciphertext.len());
        out.extend_from_slice(&salt_bytes);
        out.extend_from_slice(nonce.as_ref());
        out.extend_from_slice(&ciphertext);
        Ok(out)
    }

    /// Descriptografa dados com senha (formato gerado por encrypt_with_password)
    pub fn decrypt_with_password(data: &[u8], password: &str) -> Result<Vec<u8>, CryptoError> {
        if data.len() < 16 + 24 {
            return Err(CryptoError::DecryptionFailed);
        }

        let salt_bytes = &data[..16];
        let nonce_bytes = &data[16..40];
        let ciphertext = &data[40..];

        let salt = pwhash::argon2id13::Salt::from_slice(salt_bytes)
            .ok_or(CryptoError::DecryptionFailed)?;

        let mut key = [0u8; secretbox::KEYBYTES];
        pwhash::argon2id13::derive_key(
            &mut key,
            password.as_bytes(),
            &salt,
            pwhash::argon2id13::OPSLIMIT_INTERACTIVE,
            pwhash::argon2id13::MEMLIMIT_INTERACTIVE,
        ).map_err(|_| CryptoError::DecryptionFailed)?;

        let key_box = secretbox::Key::from_slice(&key).unwrap();

        let nonce = SecretBoxNonce::from_slice(nonce_bytes)
            .ok_or(CryptoError::DecryptionFailed)?;

        secretbox::open(ciphertext, &nonce, &key_box)
            .map_err(|_| CryptoError::DecryptionFailed)
    }
}

impl Default for Crypto {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identity_generation() {
        Crypto::init();
        let id = Crypto::generate_identity();
        assert!(!id.public_key.is_empty());
        assert!(!id.secret_key.is_empty());
    }

    #[test]
    fn test_key_exchange() {
        Crypto::init();
        let alice = Crypto::generate_identity();
        let bob = Crypto::generate_identity();

        let alice_pk = PublicKey::from_slice(&BASE64.decode(&alice.public_key).unwrap()).unwrap();
        let alice_sk = SecretKey::from_slice(&BASE64.decode(&alice.secret_key).unwrap()).unwrap();
        let bob_pk = PublicKey::from_slice(&BASE64.decode(&bob.public_key).unwrap()).unwrap();
        let bob_sk = SecretKey::from_slice(&BASE64.decode(&bob.secret_key).unwrap()).unwrap();

        let alice_session = Crypto::client_session_keys(&alice_pk, &alice_sk, &bob_pk).unwrap();
        let bob_session = Crypto::server_session_keys(&bob_pk, &bob_sk, &alice_pk).unwrap();

        let msg = b"Hello Bob!";
        let encrypted = Crypto::encrypt(msg, &alice_session.tx);
        let decrypted = Crypto::decrypt(&encrypted, &bob_session.rx).unwrap();
        assert_eq!(msg, decrypted.as_slice());

        let reply = b"Hi Alice!";
        let encrypted_reply = Crypto::encrypt(reply, &bob_session.tx);
        let decrypted_reply = Crypto::decrypt(&encrypted_reply, &alice_session.rx).unwrap();
        assert_eq!(reply, decrypted_reply.as_slice());
    }

    #[test]
    fn test_fingerprint() {
        Crypto::init();
        let id = Crypto::generate_identity();
        let pk = PublicKey::from_slice(&BASE64.decode(&id.public_key).unwrap()).unwrap();
        let fp = Fingerprint::from_public_key(&pk);
        assert!(fp.verify(&pk));
        assert_eq!(fp.formatted().len(), 19); // 16 caracteres + 3 hífens
    }

    #[test]
    fn test_password_encryption() {
        Crypto::init();
        let data = b"secrete data";
        let password = "strong-password";

        let encrypted = Crypto::encrypt_with_password(data, password).unwrap();
        let decrypted = Crypto::decrypt_with_password(&encrypted, password).unwrap();
        assert_eq!(data, decrypted.as_slice());

        assert!(Crypto::decrypt_with_password(&encrypted, "wrong").is_err());
    }
}
