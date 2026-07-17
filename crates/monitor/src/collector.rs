use sm_core::events::SystemMetrics;
use sm_core::id::ServerId;
use sysinfo::{CpuRefreshKind, Disks, Networks, System};

pub struct SystemMonitor {
    sys: System,
}

impl SystemMonitor {
    pub fn new() -> Self {
        let mut sys = System::new_all();
        sys.refresh_all();
        Self { sys }
    }

    pub fn collect(&mut self, server_id: ServerId) -> SystemMetrics {
        self.sys.refresh_cpu_specifics(CpuRefreshKind::everything());
        self.sys.refresh_memory();

        let cpu = self.sys.global_cpu_usage() as f64;
        let total_memory = self.sys.total_memory();
        let used_memory = self.sys.used_memory();
        let total_swap = self.sys.total_swap();
        let used_swap = self.sys.used_swap();
        let uptime = System::uptime();
        let processes = self.sys.processes().len() as u32;

        let mut disk_used: u64 = 0;
        let mut disk_total: u64 = 0;
        let disks = Disks::new_with_refreshed_list();
        for disk in disks.iter() {
            disk_used += disk.total_space() - disk.available_space();
            disk_total += disk.total_space();
        }

        let mut rx: u64 = 0;
        let mut tx: u64 = 0;
        let networks = Networks::new_with_refreshed_list();
        for (_, data) in networks.iter() {
            rx += data.total_received();
            tx += data.total_transmitted();
        }

        SystemMetrics {
            server_id,
            timestamp: chrono::Utc::now(),
            cpu_percent: cpu,
            memory_used_bytes: used_memory,
            memory_total_bytes: total_memory,
            disk_used_bytes: disk_used,
            disk_total_bytes: disk_total,
            network_rx_bytes: rx,
            network_tx_bytes: tx,
            swap_used_bytes: used_swap,
            swap_total_bytes: total_swap,
            load_average: [0.0; 3],
            uptime_seconds: uptime,
            process_count: processes,
        }
    }

    pub fn get_processes(&self) -> Vec<ProcessInfo> {
        self.sys
            .processes()
            .iter()
            .map(|(pid, p)| ProcessInfo {
                pid: pid.as_u32(),
                name: p.name().to_string_lossy().into_owned(),
                cpu_usage: p.cpu_usage(),
                memory_bytes: p.memory(),
                status: p.status().to_string(),
            })
            .collect()
    }
}

#[derive(Debug, Clone)]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
    pub cpu_usage: f32,
    pub memory_bytes: u64,
    pub status: String,
}
