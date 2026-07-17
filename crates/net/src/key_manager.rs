use russh_keys::{load_secret_key, PublicKeyBase64};
use sm_core::types::{SshKeyPair, SshKeyType};
use std::fs;
use std::path::{Path, PathBuf};
use tracing::info;

pub struct SshKeyManager {
    keys_dir: PathBuf,
}

impl SshKeyManager {
    pub fn new(keys_dir: impl Into<PathBuf>) -> Self {
        let dir = keys_dir.into();
        fs::create_dir_all(&dir).ok();
        Self { keys_dir: dir }
    }

    pub fn generate(
        &self,
        name: &str,
        key_type: SshKeyType,
        bits: u32,
        comment: &str,
    ) -> Result<SshKeyPair, String> {
        let safe_name = sanitize_name(name);
        let private_path = self.keys_dir.join(format!("{}_key", safe_name));
        let public_path = self.keys_dir.join(format!("{}_key.pub", safe_name));

        if private_path.exists() {
            return Err(format!("la clave '{}' ya existe", name));
        }

        // Use ssh-keygen for reliable generation
        let key_type_flag = match key_type {
            SshKeyType::Ed25519 => "ed25519",
            SshKeyType::Rsa => "rsa",
            SshKeyType::Ecdsa => "ecdsa",
        };

        let output = std::process::Command::new("ssh-keygen")
            .args([
                "-t", key_type_flag,
                "-b", &bits.to_string(),
                "-C", comment,
                "-f", private_path.to_str().unwrap_or("key"),
                "-N", "",
                "-q",
            ])
            .output()
            .map_err(|e| format!("error ejecutando ssh-keygen: {}", e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("ssh-keygen falló: {}", stderr));
        }

        let public_key = fs::read_to_string(&public_path).unwrap_or_default();

        info!("Clave SSH generada: {}", name);
        Ok(SshKeyPair {
            name: name.to_string(),
            key_type,
            public_key,
            private_key_path: private_path.to_string_lossy().into_owned(),
            bits,
            comment: comment.to_string(),
            created_at: chrono::Utc::now(),
        })
    }

    pub fn import(
        &self,
        name: &str,
        source_path: &Path,
    ) -> Result<SshKeyPair, String> {
        let safe_name = sanitize_name(name);
        let dest_path = self.keys_dir.join(format!("{}_key", safe_name));

        if dest_path.exists() {
            return Err(format!("la clave '{}' ya existe", name));
        }

        let key_pair = load_secret_key(source_path, None)
            .map_err(|e| format!("formato de clave inválido: {}", e))?;

        let public_key = key_pair
            .clone_public_key()
            .map_err(|e| format!("error leyendo clave pública: {}", e))?;

        let public_str = format!(
            "ssh-{} {}\n",
            public_key.name(),
            public_key.public_key_base64()
        );

        let private_bytes = fs::read(source_path)
            .map_err(|e| format!("no se pudo leer la clave: {}", e))?;

        fs::write(&dest_path, &private_bytes)
            .map_err(|e| format!("error guardando clave: {}", e))?;

        let pub_path = self.keys_dir.join(format!("{}_key.pub", safe_name));
        fs::write(&pub_path, &public_str).ok();

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = fs::set_permissions(&dest_path, fs::Permissions::from_mode(0o600));
        }

        info!("Clave SSH importada: {}", name);
        Ok(SshKeyPair {
            name: name.to_string(),
            key_type: SshKeyType::Ed25519,
            public_key: public_str,
            private_key_path: dest_path.to_string_lossy().into_owned(),
            bits: 256,
            comment: String::new(),
            created_at: chrono::Utc::now(),
        })
    }

    pub fn list(&self) -> Result<Vec<SshKeyPair>, String> {
        let mut keys = Vec::new();
        let entries = fs::read_dir(&self.keys_dir)
            .map_err(|e| format!("error leyendo directorio de claves: {}", e))?;

        for entry in entries {
            let entry = entry.map_err(|e| format!("error: {}", e))?;
            let path = entry.path();
            if path.extension().map_or(true, |e| e == "pub") {
                continue;
            }
            if !path.is_file() {
                continue;
            }
            let filename = path
                .file_stem()
                .unwrap_or_default()
                .to_string_lossy()
                .replace("_key", "");
            let raw_stem = path.file_stem().unwrap_or_default().to_string_lossy();
            let pub_path = self.keys_dir.join(format!("{}_key.pub", raw_stem));
            let public_key = fs::read_to_string(&pub_path).unwrap_or_default();

            keys.push(SshKeyPair {
                name: filename,
                key_type: SshKeyType::Ed25519,
                public_key,
                private_key_path: path.to_string_lossy().into_owned(),
                bits: 0,
                comment: String::new(),
                created_at: chrono::Utc::now(),
            });
        }
        Ok(keys)
    }

    pub fn delete(&self, name: &str) -> Result<(), String> {
        let safe_name = sanitize_name(name);
        let private_path = self.keys_dir.join(format!("{}_key", safe_name));
        let public_path = self.keys_dir.join(format!("{}_key.pub", safe_name));
        if private_path.exists() {
            fs::remove_file(&private_path)
                .map_err(|e| format!("error eliminando: {}", e))?;
        }
        if public_path.exists() {
            fs::remove_file(&public_path).ok();
        }
        Ok(())
    }

    pub fn get_public_key(&self, name: &str) -> Result<String, String> {
        let safe_name = sanitize_name(name);
        let pub_path = self.keys_dir.join(format!("{}_key.pub", safe_name));
        fs::read_to_string(&pub_path)
            .map_err(|e| format!("clave pública no encontrada: {}", e))
    }
}

fn sanitize_name(name: &str) -> String {
    name.chars()
        .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' { c } else { '_' })
        .collect()
}
