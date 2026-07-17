# Networking

## Visión general

El módulo `sm-net` proporciona conectividad SSH, SFTP y herramientas de diagnóstico de red.

## SSH Client

Cliente SSH asíncrono basado en **Russh 0.44** (implementación nativa en Rust).

### Autenticación soportada

- Password
- Clave privada (Ed25519, RSA, ECDSA)
- Clave con passphrase
- Agente SSH (futuro)

### Ejemplo

```rust
use sm_net::SshClient;

let client = SshClient::new();
client.connect("192.168.1.10", 22, "admin", &credential).await?;
let output = client.execute("uptime").await?;
println!("{}", output.stdout);
client.disconnect().await;
```

## Network Discovery

Herramientas de escaneo y diagnóstico.

### Funciones

- **Ping** — verificar conectividad TCP
- **Port scan** — escaneo de puertos comunes (paralelo)
- **Subnet expansion** — expandir notación CIDR

```rust
let discovery = NetworkDiscovery::new(5000);

// Ping
let alive = discovery.ping("192.168.1.1").await?;

// Scan common ports
let ports = discovery.scan_ports("192.168.1.1", &[22, 80, 443, 3389]).await;

// Expand subnet
let hosts = NetworkDiscovery::expand_subnet("192.168.1.0/24")?;
```

## Integraciones

El módulo está diseñado para integrar herramientas del sistema:

- Nmap (escaneo avanzado)
- tcpdump / Wireshark (captura de tráfico)
- iperf3 (pruebas de ancho de banda)
- mtr (traceroute con estadísticas)
- dig / nslookup (DNS)
- OpenSSL (certificados)

## Protocolos

| Protocolo | Puerto | Estado |
|---|---|---|
| SSH | 22 | Implementado |
| SFTP | 22 | Pendiente |
| FTP | 21 | Pendiente |
| RDP | 3389 | Pendiente |
| VNC | 5900 | Pendiente |
| HTTP/HTTPS | 80/443 | Pendiente |
| WebSocket | 80/443 | Pendiente |
| Serial | — | Pendiente |
