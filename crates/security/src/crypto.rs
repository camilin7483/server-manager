use aes_gcm::aead::{Aead, OsRng};
use aes_gcm::{AeadCore, Aes256Gcm, Key, KeyInit, Nonce};
use sha2::{Digest, Sha256};
use zeroize::Zeroize;

#[derive(Debug)]
pub struct CryptoEngine {
    key: Key<Aes256Gcm>,
}

impl CryptoEngine {
    pub fn from_password(password: &str, salt: &[u8]) -> Self {
        let key_bytes = derive_key(password, salt, 32);
        let key = Key::<Aes256Gcm>::from_slice(&key_bytes).clone();
        Self { key }
    }

    pub fn random_salt() -> Vec<u8> {
        use rand::RngCore;
        let mut salt = vec![0u8; 32];
        OsRng.fill_bytes(&mut salt);
        salt
    }

    pub fn encrypt(&self, plaintext: &[u8]) -> Result<(Vec<u8>, Vec<u8>), String> {
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
        let cipher = Aes256Gcm::new(&self.key);
        let ciphertext = cipher
            .encrypt(&nonce, plaintext)
            .map_err(|e| format!("error de cifrado: {}", e))?;
        Ok((ciphertext, nonce.to_vec()))
    }

    pub fn decrypt(&self, ciphertext: &[u8], nonce: &[u8]) -> Result<Vec<u8>, String> {
        let cipher = Aes256Gcm::new(&self.key);
        let nonce = Nonce::from_slice(nonce);
        let plaintext = cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| format!("error de descifrado: {}", e))?;
        Ok(plaintext)
    }

    pub fn encrypt_string(&self, text: &str) -> Result<EncryptedValue, String> {
        let (ciphertext, nonce) = self.encrypt(text.as_bytes())?;
        Ok(EncryptedValue {
            ciphertext: base64_encode(&ciphertext),
            nonce: base64_encode(&nonce),
        })
    }

    pub fn decrypt_string(&self, encrypted: &EncryptedValue) -> Result<String, String> {
        let ciphertext = base64_decode(&encrypted.ciphertext)?;
        let nonce = base64_decode(&encrypted.nonce)?;
        let bytes = self.decrypt(&ciphertext, &nonce)?;
        String::from_utf8(bytes).map_err(|e| format!("utf8 inválido: {}", e))
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EncryptedValue {
    pub ciphertext: String,
    pub nonce: String,
}

fn derive_key(password: &str, salt: &[u8], key_len: usize) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(password.as_bytes());
    hasher.update(salt);
    let hash = hasher.finalize();

    let mut key = vec![0u8; key_len];
    let mut zeroizing_key = vec![0u8; key_len];

    for i in 0..key_len {
        key[i] = hash[i % hash.len()];
    }

    // Copy key, then zeroize the key variable immediately after use
    // to prevent it from lingering in memory. The original hash is
    // not zeroized here because Sha256::finalize returns an owned
    // digest that will be dropped.

    let result = key.clone();
    zeroizing_key.copy_from_slice(&key);
    key.iter_mut().for_each(|b| *b = 0);
    zeroizing_key.zeroize();

    result
}

fn base64_encode(data: &[u8]) -> String {
    use base64::Engine;
    base64::engine::general_purpose::STANDARD.encode(data)
}

fn base64_decode(data: &str) -> Result<Vec<u8>, String> {
    use base64::Engine;
    base64::engine::general_purpose::STANDARD
        .decode(data)
        .map_err(|e| format!("base64 inválido: {}", e))
}
