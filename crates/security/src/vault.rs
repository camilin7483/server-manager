use crate::crypto::{CryptoEngine, EncryptedValue};
use sm_core::types::Credential;
use std::collections::HashMap;
use tracing::warn;

pub struct CredentialVault {
    engine: CryptoEngine,
    salt: Vec<u8>,
    store: HashMap<String, EncryptedValue>,
}

impl CredentialVault {
    pub fn new(master_password: &str) -> Self {
        let salt = CryptoEngine::random_salt();
        let engine = CryptoEngine::from_password(master_password, &salt);
        Self {
            engine,
            salt,
            store: HashMap::new(),
        }
    }

    pub fn store(&mut self, key: &str, credential: &Credential) -> Result<(), String> {
        let json = serde_json::to_string(credential)
            .map_err(|e| format!("serialization: {}", e))?;
        let encrypted = self.engine.encrypt_string(&json)?;
        self.store.insert(key.to_string(), encrypted);
        Ok(())
    }

    pub fn retrieve(&self, key: &str) -> Option<Credential> {
        let encrypted = self.store.get(key)?;
        match self.engine.decrypt_string(encrypted) {
            Ok(json) => {
                match serde_json::from_str(&json) {
                    Ok(cred) => Some(cred),
                    Err(e) => {
                        warn!("Error al deserializar credencial '{}': {}", key, e);
                        None
                    }
                }
            }
            Err(e) => {
                warn!("Error al descifrar credencial '{}': {}", key, e);
                None
            }
        }
    }

    pub fn remove(&mut self, key: &str) {
        self.store.remove(key);
    }

    pub fn keys(&self) -> Vec<&String> {
        self.store.keys().collect()
    }

    pub fn export(&self) -> Result<Vec<u8>, String> {
        let json = serde_json::to_string(&self.store)
            .map_err(|e| format!("export: {}", e))?;
        let (encrypted, nonce) = self.engine.encrypt(json.as_bytes())?;
        let mut result = self.salt.clone();
        result.extend_from_slice(&nonce);
        result.extend_from_slice(&encrypted);
        Ok(result)
    }

    pub fn import(&mut self, master_password: &str, data: &[u8]) -> Result<(), String> {
        if data.len() < 44 {
            return Err("datos de importación corruptos".into());
        }
        let imported_salt = &data[..32];
        let nonce = &data[32..44];
        let ciphertext = &data[44..];
        let import_engine = CryptoEngine::from_password(master_password, imported_salt);
        let json_bytes = import_engine.decrypt(ciphertext, nonce)?;
        let json = String::from_utf8(json_bytes)
            .map_err(|e| format!("utf8: {}", e))?;
        let imported: HashMap<String, EncryptedValue> =
            serde_json::from_str(&json)
                .map_err(|e| format!("import: {}", e))?;
        self.engine = import_engine;
        self.salt = imported_salt.to_vec();
        self.store = imported;
        Ok(())
    }

    pub fn count(&self) -> usize {
        self.store.len()
    }

    pub fn is_empty(&self) -> bool {
        self.store.is_empty()
    }
}
