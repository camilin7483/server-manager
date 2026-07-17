use crate::AppConfig;
use std::path::{Path, PathBuf};
use tracing::info;

pub struct ConfigManager {
    config: AppConfig,
    config_path: PathBuf,
    data_dir: PathBuf,
}

impl Default for ConfigManager {
    fn default() -> Self {
        Self::with_defaults()
    }
}

impl ConfigManager {
    pub fn with_defaults() -> Self {
        let config = AppConfig::default();
        let dirs = directories::ProjectDirs::from("com", "server-manager", "server-manager")
            .expect("no se pudo determinar el directorio de configuración");

        let config_dir = dirs.config_dir();
        let data_dir = dirs.data_dir();

        std::fs::create_dir_all(config_dir).ok();
        std::fs::create_dir_all(data_dir).ok();

        let config_path = config_dir.join("config.toml");

        Self {
            config,
            config_path,
            data_dir: data_dir.to_path_buf(),
        }
    }

    pub fn load(config_path: impl AsRef<Path>) -> Result<Self, Box<dyn std::error::Error>> {
        let path = config_path.as_ref();

        let config = if path.exists() {
            let contents = std::fs::read_to_string(path)?;
            let config: AppConfig = toml::from_str(&contents)?;
            info!("Configuración cargada desde {}", path.display());
            config
        } else {
            info!(
                "No se encontró archivo de configuración, usando defaults. Guardando en {}",
                path.display()
            );
            let config = AppConfig::default();
            let toml_str = toml::to_string_pretty(&config)?;
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::write(path, toml_str)?;
            config
        };

        let dirs = directories::ProjectDirs::from("com", "server-manager", "server-manager")
            .expect("no se pudo determinar el directorio de configuración");

        let data_dir = if config.general.data_dir.is_empty() {
            dirs.data_dir().to_path_buf()
        } else {
            PathBuf::from(&config.general.data_dir)
        };

        std::fs::create_dir_all(&data_dir).ok();

        Ok(Self {
            config,
            config_path: path.to_path_buf(),
            data_dir,
        })
    }

    pub fn config(&self) -> &AppConfig {
        &self.config
    }

    pub fn config_path(&self) -> &Path {
        &self.config_path
    }

    pub fn data_dir(&self) -> &Path {
        &self.data_dir
    }

    pub fn db_path(&self) -> PathBuf {
        self.data_dir.join("server-manager.db")
    }

    pub fn plugin_dir(&self) -> PathBuf {
        self.data_dir.join("plugins")
    }

    pub fn ssh_key_dir(&self) -> PathBuf {
        self.data_dir.join("ssh_keys")
    }

    pub fn backup_dir(&self) -> PathBuf {
        self.data_dir.join("backups")
    }

    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let toml_str = toml::to_string_pretty(&self.config)?;
        std::fs::write(&self.config_path, toml_str)?;
        info!(
            "Configuración guardada en {}",
            self.config_path.display()
        );
        Ok(())
    }

    pub fn update<F>(&mut self, f: F) -> Result<(), Box<dyn std::error::Error>>
    where
        F: FnOnce(&mut AppConfig),
    {
        f(&mut self.config);
        self.save()
    }
}
