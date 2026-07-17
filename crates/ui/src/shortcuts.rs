use egui::{self, Key, KeyboardShortcut, Modifiers};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Shortcut {
    NewTab,
    CloseTab,
    NextTab,
    PrevTab,
    Save,
    SaveAll,
    Find,
    Replace,
    ToggleSidebar,
    ToggleTerminal,
    FocusCommandInput,
    ClearTerminal,
    RefreshConnection,
    Connect,
    Disconnect,
    Quit,
    OpenSettings,
    Copy,
    Paste,
    Cut,
    Undo,
    Redo,
    ZoomIn,
    ZoomOut,
    ZoomReset,
}

pub struct ShortcutManager {
    shortcuts: HashMap<Shortcut, KeyboardShortcut>,
    enabled: bool,
}

impl Default for ShortcutManager {
    fn default() -> Self {
        let mut shortcuts = HashMap::new();

        let ctrl = Modifiers::CTRL;
        let ctrl_shift = Modifiers::CTRL | Modifiers::SHIFT;

        shortcuts.insert(Shortcut::NewTab,     KeyboardShortcut::new(ctrl, Key::T));
        shortcuts.insert(Shortcut::CloseTab,   KeyboardShortcut::new(ctrl, Key::W));
        shortcuts.insert(Shortcut::NextTab,    KeyboardShortcut::new(ctrl, Key::Tab));
        shortcuts.insert(Shortcut::PrevTab,    KeyboardShortcut::new(ctrl_shift, Key::Tab));
        shortcuts.insert(Shortcut::Save,       KeyboardShortcut::new(ctrl, Key::S));
        shortcuts.insert(Shortcut::SaveAll,    KeyboardShortcut::new(ctrl_shift, Key::S));
        shortcuts.insert(Shortcut::Find,       KeyboardShortcut::new(ctrl, Key::F));
        shortcuts.insert(Shortcut::Replace,    KeyboardShortcut::new(ctrl, Key::H));
        shortcuts.insert(Shortcut::ToggleSidebar, KeyboardShortcut::new(ctrl, Key::B));
        shortcuts.insert(Shortcut::FocusCommandInput, KeyboardShortcut::new(ctrl, Key::L));
        shortcuts.insert(Shortcut::ClearTerminal, KeyboardShortcut::new(ctrl, Key::K));
        shortcuts.insert(Shortcut::RefreshConnection, KeyboardShortcut::new(Modifiers::NONE, Key::F5));
        shortcuts.insert(Shortcut::Connect,    KeyboardShortcut::new(ctrl, Key::Enter));
        shortcuts.insert(Shortcut::Disconnect, KeyboardShortcut::new(ctrl, Key::D));
        shortcuts.insert(Shortcut::Quit,       KeyboardShortcut::new(ctrl, Key::Q));
        shortcuts.insert(Shortcut::Copy,       KeyboardShortcut::new(ctrl, Key::C));
        shortcuts.insert(Shortcut::Paste,      KeyboardShortcut::new(ctrl, Key::V));
        shortcuts.insert(Shortcut::Cut,        KeyboardShortcut::new(ctrl, Key::X));
        shortcuts.insert(Shortcut::Undo,       KeyboardShortcut::new(ctrl, Key::Z));
        shortcuts.insert(Shortcut::Redo,       KeyboardShortcut::new(ctrl_shift, Key::Z));
        shortcuts.insert(Shortcut::ZoomIn,     KeyboardShortcut::new(ctrl, Key::Plus));
        shortcuts.insert(Shortcut::ZoomOut,    KeyboardShortcut::new(ctrl, Key::Minus));
        shortcuts.insert(Shortcut::ZoomReset,  KeyboardShortcut::new(ctrl, Key::Num0));

        Self { shortcuts, enabled: true }
    }
}

impl ShortcutManager {
    pub fn is_pressed(&self, ctx: &egui::Context, shortcut: Shortcut) -> bool {
        if !self.enabled { return false; }
        if let Some(kb) = self.shortcuts.get(&shortcut) {
            ctx.input_mut(|i| i.consume_shortcut(kb))
        } else {
            false
        }
    }

    pub fn enable(&mut self) { self.enabled = true; }
    pub fn disable(&mut self) { self.enabled = false; }
    pub fn toggle(&mut self) { self.enabled = !self.enabled; }

    pub fn shortcut_help(&self) -> Vec<(Shortcut, String)> {
        let mut list = Vec::new();
        let descriptions = [
            (Shortcut::NewTab, "New tab"),
            (Shortcut::CloseTab, "Close tab"),
            (Shortcut::NextTab, "Next tab"),
            (Shortcut::PrevTab, "Previous tab"),
            (Shortcut::Save, "Save"),
            (Shortcut::Find, "Find"),
            (Shortcut::ToggleSidebar, "Toggle sidebar"),
            (Shortcut::Connect, "Connect"),
            (Shortcut::Disconnect, "Disconnect"),
            (Shortcut::Quit, "Quit"),
        ];
        for (shortcut, desc) in descriptions {
            if let Some(kb) = self.shortcuts.get(&shortcut) {
                let key_str = format!("{:?}", kb);
                list.push((shortcut, format!("{} — {}", desc, key_str)))
            }
        }
        list
    }
}
