use thiserror::Error;

#[derive(Debug, Error)]
pub enum CoreError {
    #[error("Servidor no encontrado: {0}")]
    ServerNotFound(crate::id::ServerId),

    #[error("Conexión fallida: {0}")]
    ConnectionFailed(String),

    #[error("Error de autenticación: {0}")]
    AuthenticationFailed(String),

    #[error("Configuración inválida: {0}")]
    InvalidConfig(String),

    #[error("Error de base de datos: {0}")]
    Database(String),

    #[error("Error de red: {0}")]
    Network(String),

    #[error("Error SSH: {0}")]
    Ssh(String),

    #[error("Error de plugin: {0}")]
    Plugin(String),

    #[error("Error de serialización: {0}")]
    Serialization(String),

    #[error("Operación no soportada: {0}")]
    Unsupported(String),

    #[error("Timeout: {0}")]
    Timeout(String),

    #[error("Permiso denegado: {0}")]
    PermissionDenied(String),

    #[error("Error de cifrado: {0}")]
    Encryption(String),

    #[error("Error de almacenamiento: {0}")]
    Storage(String),

    #[error("{0}")]
    Generic(String),
}

impl From<serde_json::Error> for CoreError {
    fn from(e: serde_json::Error) -> Self {
        Self::Serialization(e.to_string())
    }
}

impl From<std::io::Error> for CoreError {
    fn from(e: std::io::Error) -> Self {
        Self::Generic(e.to_string())
    }
}

pub type CoreResult<T> = Result<T, CoreError>;
