use std::fs;
use std::path::{Path, PathBuf};

pub struct LocalFileManager;

impl LocalFileManager {
    pub fn list(path: impl AsRef<Path>) -> Result<Vec<FileEntry>, std::io::Error> {
        let mut entries = Vec::new();
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let metadata = entry.metadata()?;
            entries.push(FileEntry {
                name: entry.file_name().to_string_lossy().into_owned(),
                path: entry.path(),
                is_dir: metadata.is_dir(),
                size: metadata.len(),
                modified: metadata
                    .modified()
                    .ok()
                    .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                    .map(|d| d.as_secs())
                    .unwrap_or(0),
                permissions: format!("{:?}", metadata.permissions()),
            });
        }
        Ok(entries)
    }

    pub fn read(path: impl AsRef<Path>) -> Result<Vec<u8>, std::io::Error> {
        fs::read(path)
    }

    pub fn write(path: impl AsRef<Path>, data: &[u8]) -> Result<(), std::io::Error> {
        if let Some(parent) = path.as_ref().parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, data)
    }

    pub fn delete(path: impl AsRef<Path>) -> Result<(), std::io::Error> {
        if path.as_ref().is_dir() {
            fs::remove_dir_all(path)
        } else {
            fs::remove_file(path)
        }
    }

    pub fn mkdir(path: impl AsRef<Path>) -> Result<(), std::io::Error> {
        fs::create_dir_all(path)
    }

    pub fn rename(from: impl AsRef<Path>, to: impl AsRef<Path>) -> Result<(), std::io::Error> {
        fs::rename(from, to)
    }

    pub fn exists(path: impl AsRef<Path>) -> bool {
        path.as_ref().exists()
    }
}

#[derive(Debug, Clone)]
pub struct FileEntry {
    pub name: String,
    pub path: PathBuf,
    pub is_dir: bool,
    pub size: u64,
    pub modified: u64,
    pub permissions: String,
}
