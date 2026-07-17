use sm_core::types::PluginManifest;

pub struct PluginRegistry {
    plugins: Vec<PluginManifest>,
}

impl PluginRegistry {
    pub fn new() -> Self {
        Self {
            plugins: Vec::new(),
        }
    }

    pub fn register(&mut self, manifest: PluginManifest) {
        self.plugins.push(manifest);
    }

    pub fn list(&self) -> &[PluginManifest] {
        &self.plugins
    }

    pub fn get(&self, id: &str) -> Option<&PluginManifest> {
        self.plugins.iter().find(|p| p.id == id)
    }

    pub fn remove(&mut self, id: &str) -> Option<PluginManifest> {
        if let Some(pos) = self.plugins.iter().position(|p| p.id == id) {
            Some(self.plugins.remove(pos))
        } else {
            None
        }
    }

    pub fn count(&self) -> usize {
        self.plugins.len()
    }
}
