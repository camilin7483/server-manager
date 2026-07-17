mod ansi;
mod grid;
mod widget;

pub use widget::TerminalWidget;

use std::collections::VecDeque;

const MAX_HISTORY: usize = 500;

pub struct TerminalSession {
    pub id: String,
    pub title: String,
    pub widget: TerminalWidget,
    pub command_input: String,
    pub history: VecDeque<String>,
    pub history_index: usize,
    pub connected: bool,
}

impl TerminalSession {
    pub fn new(id: impl Into<String>, title: impl Into<String>) -> Self {
        let mut widget = TerminalWidget::new(24, 80);
        widget.writeln("\x1b[1;32mServer Manager Terminal\x1b[0m");
        widget.writeln("");

        Self {
            id: id.into(),
            title: title.into(),
            widget,
            command_input: String::new(),
            history: VecDeque::new(),
            history_index: 0,
            connected: false,
        }
    }

    pub fn add_history(&mut self, cmd: &str) {
        let cmd = cmd.trim().to_string();
        if !cmd.is_empty() {
            if self.history.len() >= MAX_HISTORY {
                self.history.pop_front();
            }
            self.history.push_back(cmd);
        }
        self.history_index = self.history.len();
    }

    pub fn history_up(&mut self) -> Option<&str> {
        if self.history.is_empty() {
            return None;
        }
        if self.history_index > 0 {
            self.history_index -= 1;
        }
        self.history.get(self.history_index).map(|s| s.as_str())
    }

    pub fn history_down(&mut self) -> Option<&str> {
        if self.history_index < self.history.len() {
            self.history_index += 1;
            if self.history_index == self.history.len() {
                return Some(""); // empty at end
            }
        }
        self.history.get(self.history_index).map(|s| s.as_str())
    }
}

pub struct TerminalManager {
    pub sessions: Vec<TerminalSession>,
    pub active: usize,
    pub next_id: usize,
}

impl TerminalManager {
    pub fn new() -> Self {
        Self {
            sessions: vec![TerminalSession::new("local", "Local")],
            active: 0,
            next_id: 1,
        }
    }

    pub fn open_tab(&mut self, title: &str) -> usize {
        let id = self.next_id;
        self.next_id += 1;
        let session = TerminalSession::new(id.to_string(), title);
        self.sessions.push(session);
        let idx = self.sessions.len() - 1;
        self.active = idx;
        idx
    }

    pub fn close_tab(&mut self, idx: usize) {
        if self.sessions.len() <= 1 {
            return;
        }
        self.sessions.remove(idx);
        if self.active >= self.sessions.len() {
            self.active = self.sessions.len() - 1;
        }
    }

    pub fn active_session(&self) -> &TerminalSession {
        &self.sessions[self.active]
    }

    pub fn active_session_mut(&mut self) -> &mut TerminalSession {
        &mut self.sessions[self.active]
    }
}
