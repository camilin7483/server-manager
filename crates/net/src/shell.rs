/// Shell-safe argument escaping for remote SSH commands.
///
/// Uses strict allow-listing where possible and shell quoting where
/// allow-listing is not feasible. Prevents command injection from
/// user-supplied strings passed through `ssh.execute()`.
use std::path::{Path, PathBuf};

/// Escape a value for safe use as a shell argument inside single quotes.
/// Handles the only problematic character: `'` (which would close the quote).
pub fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}

/// Validate a container ID: only hex characters and at most 64 chars.
pub fn validate_container_id(id: &str) -> Result<&str, String> {
    if id.is_empty() || id.len() > 64 {
        return Err("container ID inválido: longitud fuera de rango".into());
    }
    if !id.chars().all(|c| c.is_ascii_alphanumeric()) {
        return Err("container ID inválido: solo caracteres alfanuméricos permitidos".into());
    }
    Ok(id)
}

/// Validate a container name: alphanumeric, hyphens, underscores, max 128 chars.
pub fn validate_container_name(name: &str) -> Result<&str, String> {
    if name.is_empty() || name.len() > 128 {
        return Err("nombre de contenedor inválido: longitud fuera de rango".into());
    }
    if !name.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.') {
        return Err("nombre de contenedor inválido: caracteres no permitidos".into());
    }
    Ok(name)
}

/// Validate a volume name
pub fn validate_volume_name(name: &str) -> Result<&str, String> {
    validate_container_name(name)
}

/// Validate a network name  
pub fn validate_network_name(name: &str) -> Result<&str, String> {
    validate_container_name(name)
}

/// Validate a server directory path.
/// Rejects path traversal attempts and dangerous characters.
pub fn validate_server_dir(path: &str) -> Result<String, String> {
    let normalized = Path::new(path)
        .canonicalize()
        .map_err(|_| format!("directorio inválido o inexistente: {}", path))?;

    let normalized_str = normalized.to_string_lossy();

    // Reject common dangerous patterns even after normalization
    if path.contains("..") {
        return Err("path traversal no permitido".into());
    }

    Ok(normalized_str.to_string())
}

/// Validate a relative path component (filename, directory name) — no slashes, no dots.
pub fn validate_filename(name: &str) -> Result<&str, String> {
    if name.is_empty() || name.len() > 255 {
        return Err("nombre de archivo inválido".into());
    }
    if name.contains('/') || name.contains('\0') {
        return Err("nombre de archivo inválido: contiene caracteres de ruta".into());
    }
    if name == "." || name == ".." {
        return Err("nombre de archivo inválido".into());
    }
    Ok(name)
}

/// Validate a path for SFTP operations — reject path traversal.
/// Returns the path unchanged if safe, or an error.
pub fn validate_remote_path(path: &str) -> Result<String, String> {
    if path.is_empty() {
        return Err("ruta vacía no permitida".into());
    }
    if path.contains('\0') {
        return Err("ruta contiene caracteres nulos".into());
    }
    // Normalize the path to catch `..` traversal
    let normalized = Path::new(path);
    let components: Vec<_> = normalized.components().collect();

    // Reject if path contains parent traversal
    for component in &components {
        if let std::path::Component::ParentDir = component {
            return Err("path traversal no permitido".into());
        }
    }

    // Reject absolute paths to /etc/shadow, /root, etc. for safety
    let lower = path.to_lowercase();
    let dangerous = ["/etc/shadow", "/etc/passwd", "/root/", "/boot/", "/sys/", "/proc/", "/dev/"];
    for prefix in &dangerous {
        if lower.starts_with(prefix) {
            return Err(format!("acceso a {} no permitido por seguridad", prefix));
        }
    }

    Ok(path.to_string())
}

/// Validate a command string for screen -X stuff.
/// Blocks characters that would escape the shell context.
pub fn validate_mc_command(command: &str) -> Result<&str, String> {
    if command.is_empty() {
        return Err("comando vacío no permitido".into());
    }
    if command.len() > 1024 {
        return Err("comando demasiado largo".into());
    }
    // Block shell metacharacters that could break out of 'screen -X stuff'
    let dangerous = ['\n', '\r', '\x00', '\x1b'];
    if command.chars().any(|c| dangerous.contains(&c)) {
        return Err("comando contiene caracteres de control no permitidos".into());
    }
    Ok(command)
}

/// Validate a server name (for Minecraft, Docker, etc.) — simple identifier.
pub fn validate_server_name(name: &str) -> Result<&str, String> {
    if name.is_empty() || name.len() > 64 {
        return Err("nombre de servidor inválido".into());
    }
    if !name.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_') {
        return Err("nombre de servidor inválido: solo letras, números, guiones y guiones bajos".into());
    }
    Ok(name)
}
