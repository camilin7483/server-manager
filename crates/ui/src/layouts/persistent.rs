use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutState {
    pub sidebar_width: f32,
    pub bottom_panel_height: f32,
    pub active_tab: String,
    pub tabs: Vec<TabState>,
    pub panel_sizes: HashMap<String, f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TabState {
    pub id: String,
    pub title: String,
    pub icon: String,
    pub active: bool,
}

impl Default for LayoutState {
    fn default() -> Self {
        Self {
            sidebar_width: 260.0,
            bottom_panel_height: 200.0,
            active_tab: "console".into(),
            tabs: vec![
                TabState {
                    id: "console".into(),
                    title: "Consola".into(),
                    icon: "\u{2328}".into(),
                    active: true,
                },
                TabState {
                    id: "dashboard".into(),
                    title: "Dashboard".into(),
                    icon: "\u{2b50}".into(),
                    active: false,
                },
            ],
            panel_sizes: HashMap::new(),
        }
    }
}

pub struct LayoutManager {
    state: LayoutState,
}

impl LayoutManager {
    pub fn new() -> Self {
        Self {
            state: LayoutState::default(),
        }
    }

    pub fn load(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(path)?;
        let state: LayoutState = serde_json::from_str(&content)?;
        Ok(Self { state })
    }

    pub fn save(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let json = serde_json::to_string_pretty(&self.state)?;
        if let Some(parent) = std::path::Path::new(path).parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(path, json)?;
        Ok(())
    }

    pub fn state(&self) -> &LayoutState {
        &self.state
    }

    pub fn state_mut(&mut self) -> &mut LayoutState {
        &mut self.state
    }

    pub fn set_sidebar_width(&mut self, width: f32) {
        self.state.sidebar_width = width.clamp(200.0, 500.0);
    }

    pub fn select_tab(&mut self, tab_id: &str) {
        for tab in &mut self.state.tabs {
            tab.active = tab.id == tab_id;
        }
        self.state.active_tab = tab_id.to_string();
    }

    pub fn add_tab(&mut self, id: &str, title: &str, icon: &str) {
        if !self.state.tabs.iter().any(|t| t.id == id) {
            self.state.tabs.push(TabState {
                id: id.to_string(),
                title: title.to_string(),
                icon: icon.to_string(),
                active: false,
            });
        }
    }
}
