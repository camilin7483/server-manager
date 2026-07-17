use std::time::Duration;
use tokio::net::TcpStream;

pub struct NetworkDiscovery {
    timeout: Duration,
}

impl NetworkDiscovery {
    pub fn new(timeout_ms: u64) -> Self {
        Self {
            timeout: Duration::from_millis(timeout_ms),
        }
    }

    pub async fn ping(&self, host: &str) -> Result<bool, String> {
        let addr = format!("{}:80", host);
        match tokio::time::timeout(self.timeout, TcpStream::connect(&addr)).await {
            Ok(Ok(_)) => Ok(true),
            Ok(Err(_)) => Ok(false),
            Err(_) => Ok(false),
        }
    }

    pub async fn scan_ports(&self, host: &str, ports: &[u16]) -> Vec<u16> {
        let mut results = Vec::new();
        let futures: Vec<_> = ports
            .iter()
            .map(|&port| {
                let host = host.to_string();
                let timeout = self.timeout;
                async move {
                    let addr = format!("{}:{}", host, port);
                    match tokio::time::timeout(timeout, TcpStream::connect(&addr)).await {
                        Ok(Ok(_)) => Some(port),
                        _ => None,
                    }
                }
            })
            .collect();

        let joined = futures::future::join_all(futures).await;
        for result in joined {
            if let Some(port) = result {
                results.push(port);
            }
        }
        results
    }

    pub fn expand_subnet(subnet: &str) -> Result<Vec<String>, String> {
        if subnet.contains('/') {
            let parts: Vec<&str> = subnet.split('/').collect();
            let base_ip = parts[0];
            let cidr: u8 = parts[1]
                .parse()
                .map_err(|_| format!("CIDR inválido: {}", parts[1]))?;

            if cidr > 32 {
                return Err(format!("CIDR fuera de rango: {}", cidr));
            }

            let ip: std::net::Ipv4Addr = base_ip
                .parse()
                .map_err(|_| format!("IP inválida: {}", base_ip))?;

            let ip_u32 = u32::from(ip);
            let mask = if cidr == 0 {
                0
            } else {
                !((1u64 << (32 - cidr)) - 1) as u32
            };
            let network = ip_u32 & mask;
            let broadcast = network | !mask;
            let range = broadcast - network;

            let mut hosts = Vec::new();
            for i in 1..range {
                let addr = network + i;
                let ip = std::net::Ipv4Addr::from(addr);
                hosts.push(ip.to_string());
            }
            Ok(hosts)
        } else {
            Err(format!("Formato CIDR inválido (falta /): {}", subnet))
        }
    }
}
