use serde::{Deserialize, Serialize};
use tracing::info;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GithubRelease {
    pub tag_name: String,
    pub name: String,
    pub body: String,
    pub prerelease: bool,
    pub assets: Vec<GithubAsset>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GithubAsset {
    pub name: String,
    pub browser_download_url: String,
    pub size: u64,
}

pub struct Updater {
    current_version: String,
    repo: String,
    client: reqwest::Client,
}

#[derive(Debug, Clone)]
pub struct UpdateInfo {
    pub current: String,
    pub latest: String,
    pub download_url: Option<String>,
    pub changelog: String,
    pub is_prerelease: bool,
    pub file_size: u64,
}

impl Updater {
    pub fn new(current_version: impl Into<String>, repo: impl Into<String>) -> Self {
        Self {
            current_version: current_version.into(),
            repo: repo.into(),
            client: reqwest::Client::new(),
        }
    }

    pub fn current_version(&self) -> &str {
        &self.current_version
    }

    pub async fn check(&self) -> Result<UpdateInfo, String> {
        let url = format!(
            "https://api.github.com/repos/{}/releases/latest",
            self.repo
        );

        let response = self
            .client
            .get(&url)
            .header("User-Agent", "server-manager-updater")
            .send()
            .await
            .map_err(|e| format!("error de red: {}", e))?;

        if !response.status().is_success() {
            return Err(format!(
                "GitHub API error: {}",
                response.status()
            ));
        }

        let release: GithubRelease = response
            .json()
            .await
            .map_err(|e| format!("error parseando release: {}", e))?;

        let latest = release.tag_name.trim_start_matches('v').to_string();
        let current = self.current_version.trim_start_matches('v').to_string();

        let is_newer = compare_versions(&latest, &current) > 0;

        if !is_newer {
            info!("Sin actualizaciones: {} (actual: {})", latest, current);
            return Ok(UpdateInfo {
                current,
                latest,
                download_url: None,
                changelog: release.body,
                is_prerelease: release.prerelease,
                file_size: 0,
            });
        }

        // Find appropriate asset (linux binary)
        let asset = release.assets.iter().find(|a| {
            let name = a.name.to_lowercase();
            name.contains("linux") && !name.contains("windows") && !name.contains("mac")
        }).or_else(|| release.assets.first());

        info!(
            "Nueva versión disponible: {} → {}",
            current, latest
        );

        Ok(UpdateInfo {
            current,
            latest,
            download_url: asset.map(|a| a.browser_download_url.clone()),
            changelog: release.body,
            is_prerelease: release.prerelease,
            file_size: asset.map(|a| a.size).unwrap_or(0),
        })
    }
}

fn compare_versions(a: &str, b: &str) -> i32 {
    let a_parts: Vec<u32> = a.split('.').filter_map(|p| p.parse().ok()).collect();
    let b_parts: Vec<u32> = b.split('.').filter_map(|p| p.parse().ok()).collect();

    for i in 0..a_parts.len().max(b_parts.len()) {
        let a_v = a_parts.get(i).copied().unwrap_or(0);
        let b_v = b_parts.get(i).copied().unwrap_or(0);
        if a_v > b_v { return 1; }
        if a_v < b_v { return -1; }
    }
    0
}
