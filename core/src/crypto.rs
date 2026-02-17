//! Cryptographic operations using libsodium
//! Inclui: X25519, Argon2id, Padding ISO 7816-4

use sodiumoxide::crypto::{kx, secretbox, pwhash, hash::sha256};
use sodiumoxide::utils::memzero;
use sodiumoxide::randombytes::randombytes;
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use log::error;

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

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct IdentityKeyPair {
    pub public_key: String,   // base64
    pub secret_key: String,   // base64 (segredo)
}

#[derive(Clone, Debug)]
pub struct SessionKeys {
    pub rx: SessionKey,
    pub tx: SessionKey,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Fingerprint(String);

impl Fingerprint {
    pub fn from_public_key(pk: &PublicKey) -> Self {
        let hash = sha256::hash(pk.as_ref());
        let b64 = BASE64.encode(&hash[..8]); 
        Fingerprint(b64)
    }

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

    pub fn verify(&self, pk: &PublicKey) -> bool {
        *self == Fingerprint::from_public_key(pk)
    }

    pub fn new(inner: String) -> Self {
        Fingerprint(inner)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EncryptedMessage {
    pub nonce: String,       // base64
    pub ciphertext: String,  // base64
}

pub struct Crypto;

impl Crypto {
    pub fn init() {
        if sodiumoxide::init().is_err() {
            error!("Failed to initialize sodiumoxide");
        }
    }

    pub fn new() -> Self {
        Self::init();
        Self
    }

    pub fn generate_identity() -> IdentityKeyPair {
        let (pk, sk) = kx::gen_keypair();
        IdentityKeyPair {
            public_key: BASE64.encode(pk.as_ref()),
            secret_key: BASE64.encode(sk.as_ref()),
        }
    }

    pub fn client_session_keys(
        client_pk: &PublicKey,
        client_sk: &SecretKey,
        server_pk: &PublicKey,
    ) -> Result<SessionKeys, CryptoError> {
        kx::client_session_keys(client_pk, client_sk, server_pk)
            .map(|(rx, tx)| SessionKeys { rx, tx })
            .map_err(|_| CryptoError::KeyExchangeError)
    }

    pub fn server_session_keys(
        server_pk: &PublicKey,
        server_sk: &SecretKey,
        client_pk: &PublicKey,
    ) -> Result<SessionKeys, CryptoError> {
        kx::server_session_keys(server_pk, server_sk, client_pk)
            .map(|(rx, tx)| SessionKeys { rx, tx })
            .map_err(|_| CryptoError::KeyExchangeError)
    }

    pub fn encrypt(message: &[u8], key: &SessionKey) -> EncryptedMessage {
        let nonce = secretbox::gen_nonce();
        let key_box = SecretBoxKey::from_slice(key.as_ref()).expect("SessionKey valid");
        let ciphertext = secretbox::seal(message, &nonce, &key_box);
        EncryptedMessage {
            nonce: BASE64.encode(nonce.as_ref()),
            ciphertext: BASE64.encode(&ciphertext),
        }
    }

    pub fn decrypt(enc: &EncryptedMessage, key: &SessionKey) -> Result<Vec<u8>, CryptoError> {
        let nonce = SecretBoxNonce::from_slice(
            &BASE64.decode(&enc.nonce).map_err(|e| CryptoError::Base64Error(e.to_string()))?
        ).ok_or(CryptoError::InvalidKeyLength)?;

        let ciphertext = BASE64.decode(&enc.ciphertext)
            .map_err(|e| CryptoError::Base64Error(e.to_string()))?;

        let key_box = SecretBoxKey::from_slice(key.as_ref()).expect("SessionKey valid");
        secretbox::open(&ciphertext, &nonce, &key_box)
            .map_err(|_| CryptoError::DecryptionFailed)
    }

    pub fn random_bytes(len: usize) -> Vec<u8> {
        randombytes(len)
    }

    pub fn hash(data: &[u8]) -> String {
        BASE64.encode(sha256::hash(data).as_ref())
    }

    pub fn secure_wipe(data: &mut [u8]) {
        memzero(data);
    }

    /// Criptografa dados com senha usando Argon2id + SecretBox
    pub fn encrypt_with_password(data: &[u8], password: &str) -> Result<Vec<u8>, CryptoError> {
        let salt_bytes = randombytes(16);
        let salt = pwhash::argon2id13::Salt::from_slice(&salt_bytes)
            .ok_or(CryptoError::EncryptionFailed)?;

        let mut key = [0u8; secretbox::KEYBYTES];
        pwhash::argon2id13::derive_key(
            &mut key,
            password.as_bytes(),
            &salt,
            pwhash::argon2id13::OPSLIMIT_INTERACTIVE,
            pwhash::argon2id13::MEMLIMIT_INTERACTIVE,
        ).map_err(|_| CryptoError::EncryptionFailed)?;

        let key_box = secretbox::Key::from_slice(&key).unwrap();
        let nonce = secretbox::gen_nonce();
        let ciphertext = secretbox::seal(data, &nonce, &key_box);

        // Formato: [Salt(16) | Nonce(24) | Ciphertext(...)]
        let mut out = Vec::with_capacity(16 + 24 + ciphertext.len());
        out.extend_from_slice(&salt_bytes);
        out.extend_from_slice(nonce.as_ref());
        out.extend_from_slice(&ciphertext);
        
        memzero(&mut key); // Limpa chave da memória
        Ok(out)
    }

    pub fn decrypt_with_password(data: &[u8], password: &str) -> Result<Vec<u8>, CryptoError> {
        if data.len() < 40 { return Err(CryptoError::DecryptionFailed); }

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

        let result = secretbox::open(ciphertext, &nonce, &key_box)
            .map_err(|_| CryptoError::DecryptionFailed);
            
        memzero(&mut key); // Limpa chave da memória
        result
    }

    // --- Padding (Proteção contra Traffic Analysis) ---
    
    /// Adiciona padding ISO 7816-4 para uniformizar o tamanho das mensagens
    pub fn apply_padding(data: &mut Vec<u8>, block_size: usize) {
        data.push(0x80); // Delimitador
        while data.len() % block_size != 0 {
            data.push(0x00);
        }
    }

    /// Remove o padding
    pub fn remove_padding(data: &[u8]) -> Result<&[u8], CryptoError> {
        let len = data.len();
        if len == 0 { return Err(CryptoError::DecryptionFailed); }

        let mut i = len - 1;
        while i > 0 && data[i] == 0x00 {
            i -= 1;
        }

        if data[i] != 0x80 {
            return Err(CryptoError::DecryptionFailed); // Padding inválido
        }

        Ok(&data[..i])
    }
}

impl Default for Crypto {
    fn default() -> Self { Self::new() }
}
