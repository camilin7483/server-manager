use sm_core::events::SystemMetrics;
use sm_core::id::ServerId;
use std::collections::VecDeque;

pub struct MetricsStore {
    history: VecDeque<SystemMetrics>,
    max_entries: usize,
}

impl MetricsStore {
    pub fn new(max_entries: usize) -> Self {
        Self {
            history: VecDeque::with_capacity(max_entries),
            max_entries,
        }
    }

    pub fn push(&mut self, metrics: SystemMetrics) {
        if self.history.len() >= self.max_entries {
            self.history.pop_front();
        }
        self.history.push_back(metrics);
    }

    pub fn latest(&self) -> Option<&SystemMetrics> {
        self.history.back()
    }

    pub fn for_server(&self, server_id: ServerId) -> Vec<&SystemMetrics> {
        self.history
            .iter()
            .filter(|m| m.server_id == server_id)
            .collect()
    }

    pub fn all(&self) -> &VecDeque<SystemMetrics> {
        &self.history
    }

    pub fn len(&self) -> usize {
        self.history.len()
    }

    pub fn clear(&mut self) {
        self.history.clear();
    }
}
