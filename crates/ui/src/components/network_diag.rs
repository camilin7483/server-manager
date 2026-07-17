use egui::{self, Color32, RichText, Ui};
use super::super::theme::Theme;

pub struct NetworkDiag {
    pub target: String,
    pub results: Vec<DiagResult>,
    pub scanning: bool,
    pub scan_type: DiagType,
    pub ports: String,
    pub dns_record: String,
}

#[derive(Debug, Clone)]
pub struct DiagResult {
    pub line: String,
    pub color: Color32,
    pub timestamp: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagType {
    Ping,
    Traceroute,
    PortScan,
    DnsLookup,
    Whois,
    Latency,
}

impl DiagType {
    pub fn all() -> &'static [DiagType] {
        &[
            Self::Ping, Self::Traceroute, Self::PortScan,
            Self::DnsLookup, Self::Whois, Self::Latency,
        ]
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::Ping => "Ping",
            Self::Traceroute => "Traceroute",
            Self::PortScan => "Port Scan",
            Self::DnsLookup => "DNS Lookup",
            Self::Whois => "WHOIS",
            Self::Latency => "Latency",
        }
    }
}

impl Default for NetworkDiag {
    fn default() -> Self {
        Self {
            target: String::new(),
            results: Vec::new(),
            scanning: false,
            scan_type: DiagType::Ping,
            ports: "22,80,443,3389,8080".into(),
            dns_record: "A".into(),
        }
    }
}

impl NetworkDiag {
    pub fn show(&mut self, ui: &mut Ui, theme: &Theme) {
        ui.horizontal(|ui| {
            ui.label("Target:");
            ui.text_edit_singleline(&mut self.target)
                .on_hover_text("IP, hostname o dominio");
        });

        ui.horizontal(|ui| {
            ui.label("Tipo:");
            for kind in DiagType::all() {
                let selected = self.scan_type == *kind;
                if ui.selectable_label(selected, kind.name()).clicked() {
                    self.scan_type = *kind;
                }
            }
        });

        // Type-specific options
        match self.scan_type {
            DiagType::PortScan => {
                ui.horizontal(|ui| {
                    ui.label("Puertos:");
                    ui.text_edit_singleline(&mut self.ports);
                });
            }
            DiagType::DnsLookup => {
                ui.horizontal(|ui| {
                    ui.label("Registro:");
                    for record in ["A", "AAAA", "MX", "TXT", "NS", "CNAME"] {
                        let selected = self.dns_record == record;
                        if ui.selectable_label(selected, record).clicked() {
                            self.dns_record = record.to_string();
                        }
                    }
                });
            }
            _ => {}
        }

        ui.separator();

        // Run button
        let run_label = if self.scanning { "Escaneando..." } else { "Ejecutar" };
        if ui.button(RichText::new(run_label).color(if self.scanning {
            theme.colors.accent_warning
        } else {
            theme.colors.accent_success
        })).clicked() && !self.target.is_empty()
        {
            self.run_diag();
        }

        ui.separator();

        // Results area
        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .stick_to_bottom(true)
            .show(ui, |ui| {
                for result in &self.results {
                    ui.label(
                        RichText::new(format!("[{}] {}", result.timestamp, result.line))
                            .color(result.color)
                            .monospace()
                            .small(),
                    );
                }
            });

        if ui.button("Limpiar").clicked() {
            self.results.clear();
        }
    }

    fn run_diag(&mut self) {
        self.scanning = true;
        let target = self.target.clone();
        let scan_type = self.scan_type;
        let ports = self.ports.clone();

        // Simulated diagnostics (in production would call system tools)
        let now = chrono::Utc::now().format("%H:%M:%S").to_string();
        let green = Color32::from_rgb(80, 200, 110);
        let blue = Color32::from_rgb(120, 200, 255);
        let yellow = Color32::from_rgb(230, 190, 70);

        match scan_type {
            DiagType::Ping => {
                self.results.push(DiagResult {
                    line: format!("PING {} ({}): 56 data bytes", target, target),
                    color: blue, timestamp: now.clone(),
                });
                for i in 1..=4 {
                    let rtt = 10.0 + (i as f64 * 5.0);
                    self.results.push(DiagResult {
                        line: format!("64 bytes from {}: icmp_seq={} ttl=64 time={:.1} ms", target, i, rtt),
                        color: green, timestamp: now.clone(),
                    });
                }
                self.results.push(DiagResult {
                    line: format!("4 packets transmitted, 4 received, 0% loss"),
                    color: blue, timestamp: now,
                });
            }
            DiagType::Traceroute => {
                for hop in 1..=8 {
                    self.results.push(DiagResult {
                        line: format!("{}  gateway-{}.isp.net (10.0.{}.{})  12.{} ms  11.{} ms  13.{} ms",
                            hop, hop, hop, hop, hop, hop, hop),
                        color: if hop < 5 { green } else { yellow },
                        timestamp: now.clone(),
                    });
                }
            }
            DiagType::PortScan => {
                for port_str in ports.split(',') {
                    if let Ok(port) = port_str.trim().parse::<u16>() {
                        let open = port == 22 || port == 80 || port == 443;
                        self.results.push(DiagResult {
                            line: format!("Port {}/tcp — {}", port, if open { "OPEN" } else { "CLOSED" }),
                            color: if open { green } else { Color32::from_rgb(180, 80, 80) },
                            timestamp: now.clone(),
                        });
                    }
                }
            }
            DiagType::DnsLookup => {
                self.results.push(DiagResult {
                    line: format!("DNS {} record for {}: 93.184.216.34", self.dns_record, target),
                    color: blue, timestamp: now.clone(),
                });
                self.results.push(DiagResult {
                    line: format!("TTL: 300 | Authority: ns1.example.net"),
                    color: green, timestamp: now,
                });
            }
            DiagType::Whois => {
                self.results.push(DiagResult {
                    line: format!("Domain: {}\nRegistrar: Example Registrar Inc.\nCreated: 2020-01-15\nExpires: 2027-01-15",
                        target),
                    color: blue, timestamp: now,
                });
            }
            DiagType::Latency => {
                self.results.push(DiagResult {
                    line: format!("Latency to {}: min=12.3ms avg=15.7ms max=22.1ms jitter=3.2ms", target),
                    color: green, timestamp: now,
                });
            }
        }
        self.scanning = false;
    }
}
