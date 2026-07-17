use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sm_core::events::SystemMetrics;
use sm_core::id::ServerId;
use sm_core::types::{LogEntry, Server};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReportFormat {
    Html,
    Json,
    Pdf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportConfig {
    pub title: String,
    pub format: ReportFormat,
    pub include_metrics: bool,
    pub include_logs: bool,
    pub include_servers: bool,
    pub date_from: Option<String>,
    pub date_to: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ReportData {
    pub title: String,
    pub generated_at: String,
    pub version: String,
    pub server_count: usize,
    pub online_count: usize,
    pub servers: Vec<ServerSummary>,
    pub metrics: Vec<MetricSummary>,
    pub logs: Vec<LogSummary>,
    pub alerts: Vec<AlertSummary>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ServerSummary {
    pub name: String,
    pub host: String,
    pub os: String,
    pub status: String,
    pub tags: Vec<String>,
    pub last_connected: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct MetricSummary {
    pub server_name: String,
    pub cpu_avg: f64,
    pub cpu_max: f64,
    pub memory_avg_gb: f64,
    pub memory_max_gb: f64,
    pub disk_used_gb: f64,
    pub network_rx_total_gb: f64,
    pub network_tx_total_gb: f64,
    pub samples: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct LogSummary {
    pub timestamp: String,
    pub level: String,
    pub message: String,
    pub source: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct AlertSummary {
    pub timestamp: String,
    pub message: String,
    pub severity: String,
}

pub struct ReportEngine;

impl ReportEngine {
    pub fn new() -> Self {
        Self {}
    }

    pub fn generate_json(&self, data: &ReportData) -> Result<String, String> {
        serde_json::to_string_pretty(data)
            .map_err(|e| format!("error generando JSON: {}", e))
    }

    pub fn generate_html(&self, data: &ReportData) -> Result<String, String> {
        let mut handlebars = handlebars::Handlebars::new();

        let template = r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <title>{{title}}</title>
    <style>
        body { font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
               max-width: 900px; margin: 0 auto; padding: 2rem; background: #1a1a2e;
               color: #e0e0eb; }
        h1 { color: #5096ff; border-bottom: 2px solid #32324a; padding-bottom: 0.5rem; }
        h2 { color: #8c64ff; margin-top: 2rem; }
        table { width: 100%; border-collapse: collapse; margin: 1rem 0; }
        th, td { padding: 0.5rem 0.75rem; text-align: left; border-bottom: 1px solid #32324a; }
        th { background: #242436; color: #a0a0b4; font-weight: 600; }
        tr:hover { background: #28283c; }
        .status-online { color: #50c86e; }
        .status-offline { color: #b45050; }
        .tag { background: #32324a; border-radius: 4px; padding: 2px 8px; margin: 0 2px;
               font-size: 0.85em; }
        .metric-card { background: #242436; border-radius: 8px; padding: 1rem; margin: 0.5rem 0;
                       border-left: 4px solid #5096ff; }
        .severity-critical { color: #f05050; }
        .severity-warning { color: #f0be46; }
        .footer { margin-top: 2rem; padding-top: 1rem; border-top: 1px solid #32324a;
                  font-size: 0.85em; color: #646478; }
    </style>
</head>
<body>
    <h1>{{title}}</h1>
    <p>Generado: {{generated_at}} | Server Manager {{version}}</p>

    <h2>Resumen</h2>
    <p>{{server_count}} servidores ({{online_count}} online)</p>

    <h2>Servidores</h2>
    <table>
        <tr><th>Nombre</th><th>Host</th><th>OS</th><th>Estado</th><th>Tags</th><th>Última conexión</th></tr>
        {{#each servers}}
        <tr>
            <td>{{name}}</td>
            <td>{{host}}</td>
            <td>{{os}}</td>
            <td class="status-{{status}}">{{status}}</td>
            <td>{{#each tags}}<span class="tag">{{this}}</span>{{/each}}</td>
            <td>{{last_connected}}</td>
        </tr>
        {{/each}}
    </table>

    {{#if metrics}}
    <h2>Métricas</h2>
    {{#each metrics}}
    <div class="metric-card">
        <strong>{{server_name}}</strong><br>
        CPU: {{cpu_avg}}% avg / {{cpu_max}}% max |
        RAM: {{memory_avg_gb}} GB avg |
        Disco: {{disk_used_gb}} GB |
        Red: ↓{{network_rx_total_gb}} GB ↑{{network_tx_total_gb}} GB |
        {{samples}} muestras
    </div>
    {{/each}}
    {{/if}}

    {{#if alerts}}
    <h2>Alertas</h2>
    {{#each alerts}}
    <p class="severity-{{severity}}">[{{timestamp}}] {{severity}}: {{message}}</p>
    {{/each}}
    {{/if}}

    <h2>Logs</h2>
    <table>
        <tr><th>Timestamp</th><th>Level</th><th>Message</th><th>Source</th></tr>
        {{#each logs}}
        <tr>
            <td>{{timestamp}}</td>
            <td>{{level}}</td>
            <td>{{message}}</td>
            <td>{{source}}</td>
        </tr>
        {{/each}}
    </table>

    <div class="footer">
        Server Manager — Infrastructure Management Platform. Reporte generado automáticamente.
    </div>
</body>
</html>"#;

        handlebars
            .register_template_string("report", template)
            .map_err(|e| format!("error de template: {}", e))?;

        let html = handlebars
            .render("report", data)
            .map_err(|e| format!("error renderizando HTML: {}", e))?;

        Ok(html)
    }

    pub fn generate_pdf(&self, data: &ReportData) -> Result<Vec<u8>, String> {
        // PDF generation stub — returns HTML wrapped for now
        let html = self.generate_html(data)?;
        Ok(html.into_bytes())
    }

    pub fn generate(&self, data: &ReportData, format: &ReportFormat) -> Result<Vec<u8>, String> {
        match format {
            ReportFormat::Json => {
                let json = self.generate_json(data)?;
                Ok(json.into_bytes())
            }
            ReportFormat::Html => {
                let html = self.generate_html(data)?;
                Ok(html.into_bytes())
            }
            ReportFormat::Pdf => self.generate_pdf(data),
        }
    }

    pub fn build_inventory_report(
        servers: &[Server],
        metrics: &[MetricSummary],
        logs: &[LogSummary],
    ) -> ReportData {
        let online = servers.iter().filter(|s| {
            matches!(s.status, sm_core::types::ServerStatus::Online)
        }).count();

        let server_summaries: Vec<ServerSummary> = servers
            .iter()
            .map(|s| ServerSummary {
                name: s.name.clone(),
                host: s.connection.host.clone(),
                os: "Linux".into(), // TODO: detect from profile
                status: s.status.label().to_string(),
                tags: s.tags.clone(),
                last_connected: Some(s.last_connected_at
                    .map(|t| t.to_rfc3339())
                    .unwrap_or_else(|| "Nunca".into())),
            })
            .collect();

        ReportData {
            title: "Server Manager — Inventory Report".into(),
            generated_at: Utc::now().to_rfc3339(),
            version: env!("CARGO_PKG_VERSION").into(),
            server_count: servers.len(),
            online_count: online,
            servers: server_summaries,
            metrics: metrics.to_vec(),
            logs: logs.to_vec(),
            alerts: vec![],
        }
    }
}
