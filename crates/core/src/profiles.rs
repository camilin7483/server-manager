use serde::{Deserialize, Serialize};
use super::types::{OperatingSystem, OsFlavor, ProfileType};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerProfileTemplate {
    pub name: String,
    pub os: OperatingSystem,
    pub profile_type: ProfileType,
    pub description: String,
    pub setup_commands: Vec<String>,
    pub default_packages: Vec<String>,
    pub services: Vec<String>,
    pub ports: Vec<u16>,
    pub firewall_rules: Vec<String>,
    pub monitor_checks: Vec<String>,
    pub init_script: Option<String>,
}

impl ServerProfileTemplate {
    pub fn all_presets() -> Vec<ServerProfileTemplate> {
        vec![
            Self::ubuntu_server(),
            Self::debian_server(),
            Self::arch_server(),
            Self::docker_host(),
            Self::nginx_web(),
            Self::minecraft_java(),
            Self::minecraft_bedrock(),
            Self::postgres_db(),
            Self::nodejs_app(),
            Self::python_app(),
            Self::wireguard_vpn(),
            Self::proxmox_host(),
            Self::openmediavault_nas(),
            Self::raspberry_pi(),
        ]
    }

    fn ubuntu_server() -> Self {
        Self {
            name: "Ubuntu Server".into(),
            os: OperatingSystem::Linux(OsFlavor::Ubuntu),
            profile_type: ProfileType::Generic,
            description: "Servidor Ubuntu LTS genérico con actualizaciones automáticas".into(),
            setup_commands: vec![
                "apt-get update -y".into(),
                "apt-get upgrade -y".into(),
                "apt-get install -y curl wget vim htop net-tools ufw".into(),
                "timedatectl set-timezone UTC".into(),
                "ufw allow 22/tcp".into(),
                "ufw --force enable".into(),
            ],
            default_packages: vec![
                "curl".into(), "wget".into(), "vim".into(), "htop".into(),
                "net-tools".into(), "ufw".into(), "git".into(), "unzip".into(),
            ],
            services: vec!["sshd".into()],
            ports: vec![22],
            firewall_rules: vec!["ufw allow 22/tcp".into()],
            monitor_checks: vec!["systemctl is-active sshd".into()],
            init_script: None,
        }
    }

    fn debian_server() -> Self {
        Self {
            name: "Debian Server".into(),
            os: OperatingSystem::Linux(OsFlavor::Debian),
            profile_type: ProfileType::Generic,
            description: "Servidor Debian estable con herramientas básicas".into(),
            setup_commands: vec![
                "apt-get update -y".into(),
                "apt-get upgrade -y".into(),
                "apt-get install -y curl wget vim htop sudo ufw".into(),
            ],
            default_packages: vec![
                "curl".into(), "wget".into(), "vim".into(), "htop".into(), "sudo".into(),
            ],
            services: vec!["sshd".into()],
            ports: vec![22],
            firewall_rules: vec!["ufw allow 22/tcp".into()],
            monitor_checks: vec!["systemctl is-active sshd".into()],
            init_script: None,
        }
    }

    fn arch_server() -> Self {
        Self {
            name: "Arch Linux".into(),
            os: OperatingSystem::Linux(OsFlavor::Arch),
            profile_type: ProfileType::Generic,
            description: "Servidor Arch Linux rolling release".into(),
            setup_commands: vec![
                "pacman -Syu --noconfirm".into(),
                "pacman -S --noconfirm curl wget vim htop git base-devel".into(),
            ],
            default_packages: vec![
                "curl".into(), "wget".into(), "vim".into(), "htop".into(), "git".into(),
                "base-devel".into(),
            ],
            services: vec!["sshd".into()],
            ports: vec![22],
            firewall_rules: vec![],
            monitor_checks: vec!["systemctl is-active sshd".into()],
            init_script: None,
        }
    }

    fn docker_host() -> Self {
        Self {
            name: "Docker Host".into(),
            os: OperatingSystem::Linux(OsFlavor::Ubuntu),
            profile_type: ProfileType::DockerHost,
            description: "Host Docker con Docker Compose y monitoreo de contenedores".into(),
            setup_commands: vec![
                "apt-get update -y".into(),
                "apt-get install -y ca-certificates curl".into(),
                "install -m 0755 -d /etc/apt/keyrings".into(),
                "curl -fsSL https://download.docker.com/linux/ubuntu/gpg -o /etc/apt/keyrings/docker.asc".into(),
                "chmod a+r /etc/apt/keyrings/docker.asc".into(),
                "echo 'deb [arch=$(dpkg --print-architecture) signed-by=/etc/apt/keyrings/docker.asc] https://download.docker.com/linux/ubuntu $(. /etc/os-release && echo \"$VERSION_CODENAME\") stable' | tee /etc/apt/sources.list.d/docker.list > /dev/null".into(),
                "apt-get update -y".into(),
                "apt-get install -y docker-ce docker-ce-cli containerd.io docker-buildx-plugin docker-compose-plugin".into(),
                "systemctl enable docker".into(),
                "systemctl start docker".into(),
            ],
            default_packages: vec![
                "docker-ce".into(), "docker-compose-plugin".into(), "curl".into(),
            ],
            services: vec!["docker".into()],
            ports: vec![22, 2375, 2376],
            firewall_rules: vec![
                "ufw allow 22/tcp".into(),
                "ufw allow 2375/tcp".into(),
            ],
            monitor_checks: vec![
                "systemctl is-active docker".into(),
                "docker ps -q | wc -l".into(),
            ],
            init_script: None,
        }
    }

    fn nginx_web() -> Self {
        Self {
            name: "Nginx Web Server".into(),
            os: OperatingSystem::Linux(OsFlavor::Ubuntu),
            profile_type: ProfileType::WebServer,
            description: "Nginx + PHP-FPM + Let's Encrypt (Certbot)".into(),
            setup_commands: vec![
                "apt-get update -y".into(),
                "apt-get install -y nginx certbot python3-certbot-nginx".into(),
                "systemctl enable nginx".into(),
                "systemctl start nginx".into(),
                "ufw allow 80/tcp".into(),
                "ufw allow 443/tcp".into(),
            ],
            default_packages: vec![
                "nginx".into(), "certbot".into(), "python3-certbot-nginx".into(),
            ],
            services: vec!["nginx".into(), "certbot.timer".into()],
            ports: vec![22, 80, 443],
            firewall_rules: vec![
                "ufw allow 80/tcp".into(),
                "ufw allow 443/tcp".into(),
            ],
            monitor_checks: vec![
                "systemctl is-active nginx".into(),
                "nginx -t 2>&1".into(),
            ],
            init_script: None,
        }
    }

    fn minecraft_java() -> Self {
        Self {
            name: "Minecraft Java Server".into(),
            os: OperatingSystem::Linux(OsFlavor::Ubuntu),
            profile_type: ProfileType::MinecraftJava,
            description: "Servidor Minecraft Java con PaperMC".into(),
            setup_commands: vec![
                "apt-get update -y".into(),
                "apt-get install -y openjdk-21-jre-headless screen wget jq".into(),
                "mkdir -p /opt/minecraft".into(),
                "useradd -r -s /bin/false minecraft || true".into(),
                "chown -R minecraft:minecraft /opt/minecraft".into(),
                "ufw allow 25565/tcp".into(),
            ],
            default_packages: vec![
                "openjdk-21-jre-headless".into(), "screen".into(), "wget".into(),
            ],
            services: vec!["minecraft@.service".into()],
            ports: vec![22, 25565],
            firewall_rules: vec!["ufw allow 25565/tcp".into()],
            monitor_checks: vec![
                "screen -ls | grep minecraft || true".into(),
                "ss -tlnp | grep 25565".into(),
            ],
            init_script: None,
        }
    }

    fn minecraft_bedrock() -> Self {
        Self {
            name: "Minecraft Bedrock Server".into(),
            os: OperatingSystem::Linux(OsFlavor::Ubuntu),
            profile_type: ProfileType::MinecraftBedrock,
            description: "Servidor Minecraft Bedrock Edition".into(),
            setup_commands: vec![
                "apt-get update -y".into(),
                "apt-get install -y screen wget unzip libcurl4".into(),
                "mkdir -p /opt/minecraft-bedrock".into(),
                "ufw allow 19132/udp".into(),
            ],
            default_packages: vec!["screen".into(), "wget".into(), "unzip".into()],
            services: vec!["minecraft-bedrock@.service".into()],
            ports: vec![22, 19132],
            firewall_rules: vec!["ufw allow 19132/udp".into()],
            monitor_checks: vec!["ss -ulnp | grep 19132".into()],
            init_script: None,
        }
    }

    fn postgres_db() -> Self {
        Self {
            name: "PostgreSQL Database".into(),
            os: OperatingSystem::Linux(OsFlavor::Ubuntu),
            profile_type: ProfileType::Database,
            description: "PostgreSQL con backups automáticos".into(),
            setup_commands: vec![
                "apt-get update -y".into(),
                "apt-get install -y postgresql postgresql-contrib".into(),
                "systemctl enable postgresql".into(),
                "systemctl start postgresql".into(),
                "ufw allow 5432/tcp".into(),
            ],
            default_packages: vec!["postgresql".into(), "postgresql-contrib".into()],
            services: vec!["postgresql".into()],
            ports: vec![22, 5432],
            firewall_rules: vec!["ufw allow 5432/tcp".into()],
            monitor_checks: vec![
                "systemctl is-active postgresql".into(),
                "pg_isready".into(),
            ],
            init_script: None,
        }
    }

    fn nodejs_app() -> Self {
        Self {
            name: "Node.js Application".into(),
            os: OperatingSystem::Linux(OsFlavor::Ubuntu),
            profile_type: ProfileType::NodeJs,
            description: "Node.js + PM2 para aplicaciones JavaScript/TypeScript".into(),
            setup_commands: vec![
                "apt-get update -y".into(),
                "apt-get install -y nodejs npm".into(),
                "npm install -g pm2".into(),
                "pm2 startup systemd".into(),
                "ufw allow 3000/tcp".into(),
            ],
            default_packages: vec!["nodejs".into(), "npm".into()],
            services: vec!["pm2-root.service".into()],
            ports: vec![22, 3000],
            firewall_rules: vec!["ufw allow 3000/tcp".into()],
            monitor_checks: vec!["pm2 list".into()],
            init_script: None,
        }
    }

    fn python_app() -> Self {
        Self {
            name: "Python Application".into(),
            os: OperatingSystem::Linux(OsFlavor::Ubuntu),
            profile_type: ProfileType::Python,
            description: "Python 3 + pip + venv para aplicaciones".into(),
            setup_commands: vec![
                "apt-get update -y".into(),
                "apt-get install -y python3 python3-pip python3-venv".into(),
                "pip3 install --upgrade pip".into(),
            ],
            default_packages: vec!["python3".into(), "python3-pip".into(), "python3-venv".into()],
            services: vec![],
            ports: vec![22, 8000],
            firewall_rules: vec!["ufw allow 8000/tcp".into()],
            monitor_checks: vec!["python3 --version".into()],
            init_script: None,
        }
    }

    fn wireguard_vpn() -> Self {
        Self {
            name: "WireGuard VPN".into(),
            os: OperatingSystem::Linux(OsFlavor::Ubuntu),
            profile_type: ProfileType::Vpn,
            description: "Servidor VPN WireGuard".into(),
            setup_commands: vec![
                "apt-get update -y".into(),
                "apt-get install -y wireguard".into(),
                "systemctl enable wg-quick@wg0".into(),
                "ufw allow 51820/udp".into(),
            ],
            default_packages: vec!["wireguard".into()],
            services: vec!["wg-quick@wg0".into()],
            ports: vec![22, 51820],
            firewall_rules: vec!["ufw allow 51820/udp".into()],
            monitor_checks: vec!["wg show".into()],
            init_script: None,
        }
    }

    fn proxmox_host() -> Self {
        Self {
            name: "Proxmox VE Host".into(),
            os: OperatingSystem::Linux(OsFlavor::Proxmox),
            profile_type: ProfileType::Generic,
            description: "Host de virtualización Proxmox VE".into(),
            setup_commands: vec![
                "apt-get update -y".into(),
                "apt-get install -y qemu-guest-agent".into(),
            ],
            default_packages: vec!["qemu-guest-agent".into()],
            services: vec!["pve-cluster".into(), "pvedaemon".into(), "pveproxy".into()],
            ports: vec![22, 8006, 3128, 5900, 111],
            firewall_rules: vec![
                "ufw allow 8006/tcp".into(),
                "ufw allow 5900:5999/tcp".into(),
            ],
            monitor_checks: vec![
                "systemctl is-active pve-cluster".into(),
                "pvesh get /cluster/resources --output-format json | jq length".into(),
            ],
            init_script: None,
        }
    }

    fn openmediavault_nas() -> Self {
        Self {
            name: "OpenMediaVault NAS".into(),
            os: OperatingSystem::Linux(OsFlavor::OpenMediaVault),
            profile_type: ProfileType::Nas,
            description: "Servidor NAS con OpenMediaVault".into(),
            setup_commands: vec![
                "apt-get update -y".into(),
                "apt-get install -y smartmontools hdparm".into(),
            ],
            default_packages: vec!["smartmontools".into(), "hdparm".into()],
            services: vec!["smbd".into(), "nfs-kernel-server".into(), "smartd".into()],
            ports: vec![22, 80, 443, 445, 139, 111, 2049],
            firewall_rules: vec![
                "ufw allow 445/tcp".into(),
                "ufw allow 2049/tcp".into(),
            ],
            monitor_checks: vec![
                "systemctl is-active smbd".into(),
                "df -h /".into(),
            ],
            init_script: None,
        }
    }

    fn raspberry_pi() -> Self {
        Self {
            name: "Raspberry Pi".into(),
            os: OperatingSystem::Linux(OsFlavor::Raspbian),
            profile_type: ProfileType::Generic,
            description: "Raspberry Pi optimizado con herramientas ligeras".into(),
            setup_commands: vec![
                "apt-get update -y".into(),
                "apt-get install -y curl wget vim htop git".into(),
            ],
            default_packages: vec![
                "curl".into(), "wget".into(), "vim".into(), "htop".into(), "git".into(),
            ],
            services: vec!["sshd".into()],
            ports: vec![22],
            firewall_rules: vec![],
            monitor_checks: vec![
                "systemctl is-active sshd".into(),
                "vcgencmd measure_temp 2>/dev/null || echo 'N/A'".into(),
            ],
            init_script: None,
        }
    }
}
