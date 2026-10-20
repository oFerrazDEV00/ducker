//! Debugger do Pato — Laboratório e Diagnóstico de Redes Local (P2P / LocalSend v2)
//!
//! Este módulo coleta dados reais sobre a infraestrutura de rede local, interfaces,
//! portas, multicast e conectividade entre peers, sem invasão de privacidade e sem
//! captura de pacotes brutos.

use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, TcpListener, UdpSocket};
use std::time::Instant;

use serde::{Deserialize, Serialize};

use crate::identity::Identity;
use crate::model::{DeviceInfo, Protocol, DEFAULT_PORT};
use crate::node::Node;

/// Informações de uma interface de rede detectada no sistema operacional.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NetworkInterfaceInfo {
    pub name: String,
    pub ipv4: Option<Ipv4Addr>,
    pub ipv6: Option<Ipv6Addr>,
    pub netmask_v4: Option<Ipv4Addr>,
    pub broadcast_v4: Option<Ipv4Addr>,
    pub is_loopback: bool,
    pub is_up: bool,
    pub is_multicast_capable: bool,
}

/// Estado de uma porta de rede.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PortStatus {
    pub port: u16,
    pub protocol: String,
    pub state: String,
    pub message: String,
}

/// Estado da descoberta multicast UDP do Ducker.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MulticastStatus {
    pub group: Ipv4Addr,
    pub port: u16,
    pub interfaces_joined: usize,
    pub active: bool,
    pub message: String,
}

/// Anomalia identificada no diagnóstico com explicação educacional.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DiagnosticAnomaly {
    pub category: String,
    pub observed: String,
    pub meaning: String,
    pub probable_cause: String,
    pub recommended_action: String,
}

/// Resultado de uma sonda (ping / health-check) contra um dispositivo remoto.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PeerDiagnosticProbe {
    pub peer_ip: IpAddr,
    pub peer_port: u16,
    pub protocol: Protocol,
    pub reachable: bool,
    pub latency_ms: Option<u64>,
    pub alias: Option<String>,
    pub device_model: Option<String>,
    pub fingerprint: Option<String>,
    pub fingerprint_verified: Option<bool>,
    pub error: Option<String>,
}

/// Registro educativo de evento de tráfego do Ducker.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TrafficEventSummary {
    pub timestamp: String,
    pub event_type: String,
    pub details: String,
    pub bytes: Option<u64>,
}

/// Relatório completo de diagnóstico de saúde de rede ("Debugger do Pato").
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NetworkHealthReport {
    pub generated_at: String,
    pub os: String,
    pub node_alias: String,
    pub node_quac_id: String,
    pub node_fingerprint_short: String,
    pub interfaces: Vec<NetworkInterfaceInfo>,
    pub ports: Vec<PortStatus>,
    pub multicast: MulticastStatus,
    pub peers_discovered_count: usize,
    pub anomalies: Vec<DiagnosticAnomaly>,
    pub traffic_events: Vec<TrafficEventSummary>,
}

impl NetworkHealthReport {
    /// Exporta o relatório para formato Markdown amigável e legível.
    pub fn to_markdown(&self) -> String {
        let mut md = String::new();
        md.push_str("# 🦆 Relatório do Debugger do Pato — Diagnóstico de Rede\n\n");
        md.push_str(&format!("- **Data/Hora:** {}\n", self.generated_at));
        md.push_str(&format!("- **Sistema Operacional:** {}\n", self.os));
        md.push_str(&format!(
            "- **Dispositivo Local:** {} (ID Quac: #{})\n",
            self.node_alias, self.node_quac_id
        ));
        md.push_str(&format!(
            "- **Fingerprint TLS (SHA-256):** {}\n\n",
            self.node_fingerprint_short
        ));

        md.push_str("## 1. Interfaces de Rede Locais\n\n");
        if self.interfaces.is_empty() {
            md.push_str("Nenhuma interface de rede detectada.\n\n");
        } else {
            md.push_str("| Interface | IPv4 | Máscara | Broadcast | Status |\n");
            md.push_str("|-----------|------|---------|-----------|--------|\n");
            for iface in &self.interfaces {
                let ip_str = iface
                    .ipv4
                    .map(|ip| ip.to_string())
                    .unwrap_or_else(|| "N/A".to_string());
                let mask_str = iface
                    .netmask_v4
                    .map(|m| m.to_string())
                    .unwrap_or_else(|| "-".to_string());
                let bcast_str = iface
                    .broadcast_v4
                    .map(|b| b.to_string())
                    .unwrap_or_else(|| "-".to_string());
                let status_str = if iface.is_loopback {
                    "Loopback"
                } else if iface.is_up {
                    "Ativa"
                } else {
                    "Inativa"
                };
                md.push_str(&format!(
                    "| {} | {} | {} | {} | {} |\n",
                    iface.name, ip_str, mask_str, bcast_str, status_str
                ));
            }
            md.push('\n');
        }

        md.push_str("## 2. Portas do Protocolo LocalSend v2\n\n");
        for p in &self.ports {
            md.push_str(&format!(
                "- **Porta {} ({})**: `{}` — {}\n",
                p.port, p.protocol, p.state, p.message
            ));
        }
        md.push('\n');

        md.push_str("## 3. Descoberta Multicast UDP\n\n");
        md.push_str(&format!(
            "- **Grupo:** {}:{}\n",
            self.multicast.group, self.multicast.port
        ));
        md.push_str(&format!(
            "- **Interfaces conectadas:** {}\n",
            self.multicast.interfaces_joined
        ));
        md.push_str(&format!("- **Status:** {}\n\n", self.multicast.message));

        md.push_str(&format!(
            "## 4. Dispositivos Vizinhos Descobertos ({})\n\n",
            self.peers_discovered_count
        ));

        md.push_str("## 5. Análise de Diagnóstico e Recomendações\n\n");
        if self.anomalies.is_empty() {
            md.push_str("✅ **Tudo certo!** Nenhuma anomalia de rede foi detectada no momento.\n");
        } else {
            for (i, a) in self.anomalies.iter().enumerate() {
                md.push_str(&format!("### {}. Categoria: {}\n\n", i + 1, a.category));
                md.push_str(&format!("- **O que foi observado:** {}\n", a.observed));
                md.push_str(&format!("- **O que significa:** {}\n", a.meaning));
                md.push_str(&format!("- **Causa provável:** {}\n", a.probable_cause));
                md.push_str(&format!(
                    "- **Ação recomendada:** {}\n\n",
                    a.recommended_action
                ));
            }
        }

        md.push_str("---\n*Relatório gerado pelo Ducker 1.0.0 (Debugger do Pato) — Nenhum dado privado ou chave criptográfica foi exposta.*\n");
        md
    }

    /// Exporta o relatório estruturado em JSON para integrações ou suporte técnico.
    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).unwrap_or_else(|_| "{}".to_string())
    }
}

/// Inspeciona todas as interfaces de rede locais ativas do sistema.
pub fn get_network_interfaces() -> Vec<NetworkInterfaceInfo> {
    let mut out = Vec::new();
    let addrs = match if_addrs::get_if_addrs() {
        Ok(a) => a,
        Err(_) => return out,
    };

    for iface in addrs {
        let is_loopback = iface.is_loopback();
        let mut ipv4 = None;
        let mut ipv6 = None;
        let mut netmask_v4 = None;
        let mut broadcast_v4 = None;

        match iface.addr {
            if_addrs::IfAddr::V4(ref v4) => {
                ipv4 = Some(v4.ip);
                netmask_v4 = Some(v4.netmask);
                broadcast_v4 = v4.broadcast;
            }
            if_addrs::IfAddr::V6(ref v6) => {
                ipv6 = Some(v6.ip);
            }
        }

        let is_up = if is_loopback {
            true
        } else if let Some(v4) = ipv4 {
            !v4.is_link_local()
        } else {
            ipv6.is_some()
        };

        let is_multicast_capable = !is_loopback && is_up && ipv4.is_some();

        out.push(NetworkInterfaceInfo {
            name: iface.name,
            ipv4,
            ipv6,
            netmask_v4,
            broadcast_v4,
            is_loopback,
            is_up,
            is_multicast_capable,
        });
    }

    out
}

/// Verifica a disponibilidade das portas locais TCP e UDP usadas pelo protocolo LocalSend v2.
pub fn check_local_ports(tcp_port: u16, udp_port: u16) -> Vec<PortStatus> {
    let mut out = Vec::new();

    // Verificação TCP
    match TcpListener::bind(("0.0.0.0", tcp_port)) {
        Ok(_) => {
            out.push(PortStatus {
                port: tcp_port,
                protocol: "TCP".to_string(),
                state: "DISPONÍVEL".to_string(),
                message: "Porta livre para receber conexões HTTP/HTTPS".to_string(),
            });
        }
        Err(e) => {
            out.push(PortStatus {
                port: tcp_port,
                protocol: "TCP".to_string(),
                state: "OCUPADA".to_string(),
                message: format!(
                    "Porta já em uso ou bloqueada: {} (outro Ducker/LocalSend ativo?)",
                    e
                ),
            });
        }
    }

    // Verificação UDP
    match UdpSocket::bind(("0.0.0.0", udp_port)) {
        Ok(_) => {
            out.push(PortStatus {
                port: udp_port,
                protocol: "UDP".to_string(),
                state: "DISPONÍVEL".to_string(),
                message: "Porta livre para multicast e anúncios de presença".to_string(),
            });
        }
        Err(e) => {
            out.push(PortStatus {
                port: udp_port,
                protocol: "UDP".to_string(),
                state: "OCUPADA".to_string(),
                message: format!("Porta já em uso ou bloqueada: {}", e),
            });
        }
    }

    out
}

/// Executa uma sonda ativa (ping / info) contra um dispositivo na rede.
pub async fn probe_peer(
    ip: IpAddr,
    port: u16,
    protocol: Protocol,
    expected_fingerprint: Option<&str>,
) -> PeerDiagnosticProbe {
    let start = Instant::now();
    let scheme = protocol.scheme();
    let url = format!("{}://{}:{}/api/localsend/v2/info", scheme, ip, port);

    let client = match reqwest::Client::builder()
        .danger_accept_invalid_certs(true)
        .timeout(std::time::Duration::from_millis(1500))
        .build()
    {
        Ok(c) => c,
        Err(e) => {
            return PeerDiagnosticProbe {
                peer_ip: ip,
                peer_port: port,
                protocol,
                reachable: false,
                latency_ms: None,
                alias: None,
                device_model: None,
                fingerprint: None,
                fingerprint_verified: None,
                error: Some(format!("Falha ao inicializar cliente HTTP: {}", e)),
            };
        }
    };

    match client.get(&url).send().await {
        Ok(resp) => {
            let latency_ms = start.elapsed().as_millis() as u64;
            if resp.status().is_success() {
                match resp.json::<DeviceInfo>().await {
                    Ok(info) => {
                        let fp_verified = expected_fingerprint
                            .map(|exp| exp.trim().eq_ignore_ascii_case(info.fingerprint.trim()));

                        PeerDiagnosticProbe {
                            peer_ip: ip,
                            peer_port: port,
                            protocol,
                            reachable: true,
                            latency_ms: Some(latency_ms),
                            alias: Some(info.alias),
                            device_model: info.device_model,
                            fingerprint: Some(info.fingerprint),
                            fingerprint_verified: fp_verified,
                            error: None,
                        }
                    }
                    Err(e) => PeerDiagnosticProbe {
                        peer_ip: ip,
                        peer_port: port,
                        protocol,
                        reachable: true,
                        latency_ms: Some(latency_ms),
                        alias: None,
                        device_model: None,
                        fingerprint: None,
                        fingerprint_verified: None,
                        error: Some(format!("Resposta recebida com formato inválido: {}", e)),
                    },
                }
            } else {
                PeerDiagnosticProbe {
                    peer_ip: ip,
                    peer_port: port,
                    protocol,
                    reachable: false,
                    latency_ms: Some(latency_ms),
                    alias: None,
                    device_model: None,
                    fingerprint: None,
                    fingerprint_verified: None,
                    error: Some(format!(
                        "Dispositivo respondeu com status {}",
                        resp.status()
                    )),
                }
            }
        }
        Err(e) => PeerDiagnosticProbe {
            peer_ip: ip,
            peer_port: port,
            protocol,
            reachable: false,
            latency_ms: None,
            alias: None,
            device_model: None,
            fingerprint: None,
            fingerprint_verified: None,
            error: Some(format!("Falha ao conectar: {}", e)),
        },
    }
}

/// Executa um diagnóstico completo do nó local e da conectividade de rede.
pub async fn run_diagnostics(
    node: Option<&Node>,
    identity: Option<&Identity>,
) -> NetworkHealthReport {
    let now = chrono_or_fallback_timestamp();
    let os = std::env::consts::OS.to_string();

    let node_alias = identity
        .map(|i| i.alias.clone())
        .unwrap_or_else(|| "Dispositivo não inicializado".to_string());
    let node_quac_id = identity
        .map(|i| i.quac_id.to_string())
        .unwrap_or_else(|| "N/A".to_string());
    let node_fingerprint_short = identity
        .map(|i| {
            if i.fingerprint.len() >= 16 {
                format!("{}...", &i.fingerprint[..16])
            } else {
                i.fingerprint.clone()
            }
        })
        .unwrap_or_else(|| "N/A".to_string());

    let interfaces = get_network_interfaces();
    let ports = check_local_ports(DEFAULT_PORT, DEFAULT_PORT);

    let active_v4 = interfaces
        .iter()
        .filter(|i| !i.is_loopback && i.is_up && i.ipv4.is_some())
        .count();

    let multicast = MulticastStatus {
        group: Ipv4Addr::new(224, 0, 0, 167),
        port: DEFAULT_PORT,
        interfaces_joined: active_v4,
        active: active_v4 > 0,
        message: if active_v4 > 0 {
            format!(
                "Multicast associado a {} interface(s) IPv4 ativa(s)",
                active_v4
            )
        } else {
            "Nenhuma interface ativa elegível para multicast".to_string()
        },
    };

    let peers_discovered_count = if let Some(n) = node {
        n.peers().len()
    } else {
        0
    };

    // Identificação de anomalias com mensagens explicativas
    let mut anomalies = Vec::new();

    if active_v4 == 0 {
        anomalies.push(DiagnosticAnomaly {
            category: "Conectividade Física / Wi-Fi".to_string(),
            observed: "Nenhuma interface de rede IPv4 local ativa encontrada.".to_string(),
            meaning: "O computador parece estar desconectado de qualquer roteador, Wi-Fi ou cabo de rede.".to_string(),
            probable_cause: "Wi-Fi desligado, cabo desconectado ou adaptador de rede desativado.".to_string(),
            recommended_action: "Conecte-se à mesma rede Wi-Fi ou cabo Ethernet dos outros aparelhos.".to_string(),
        });
    }

    for p in &ports {
        if p.state == "OCUPADA" {
            anomalies.push(DiagnosticAnomaly {
                category: "Portas de Rede em Conflito".to_string(),
                observed: format!("Porta {} ({}) está ocupada.", p.port, p.protocol),
                meaning: "Outro aplicativo ou outra instância do Ducker / LocalSend já está em execução no sistema.".to_string(),
                probable_cause: "Um processo anterior do Ducker está rodando em segundo plano.".to_string(),
                recommended_action: "Execute 'ducker stop' ou feche instâncias duplicadas no Gerenciador de Tarefas.".to_string(),
            });
        }
    }

    if active_v4 > 0 && peers_discovered_count == 0 {
        anomalies.push(DiagnosticAnomaly {
            category: "Descoberta de Dispositivos".to_string(),
            observed: "Interface ativa, porém nenhum dispositivo vizinho foi descoberto até o momento.".to_string(),
            meaning: "O tráfego multicast UDP pode estar bloqueado entre os aparelhos pelo roteador ou firewall.".to_string(),
            probable_cause: "Isolamento de cliente Wi-Fi (AP Isolation), rede de visitantes (Guest Network) ou firewall do Windows bloqueando a porta 53317.".to_string(),
            recommended_action: "1. Desative o 'Isolamento de Cliente' no roteador. 2. No terminal, execute 'ducker devices --scan' para forçar a busca HTTP na sub-rede.".to_string(),
        });
    }

    NetworkHealthReport {
        generated_at: now,
        os,
        node_alias,
        node_quac_id,
        node_fingerprint_short,
        interfaces,
        ports,
        multicast,
        peers_discovered_count,
        anomalies,
        traffic_events: Vec::new(),
    }
}

fn chrono_or_fallback_timestamp() -> String {
    let now = std::time::SystemTime::now();
    let duration = now
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    let secs = duration.as_secs();
    let hours = (secs / 3600) % 24;
    let mins = (secs / 60) % 60;
    let s = secs % 60;
    format!("{:02}:{:02}:{:02} UTC", hours, mins, s)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_network_interfaces_detection() {
        let ifaces = get_network_interfaces();
        // Em qualquer máquina com loopback ou rede, deve retornar ao menos 1 interface
        assert!(!ifaces.is_empty());
    }

    #[test]
    fn test_local_ports_check() {
        let statuses = check_local_ports(53399, 53399);
        assert_eq!(statuses.len(), 2);
        assert_eq!(statuses[0].protocol, "TCP");
        assert_eq!(statuses[1].protocol, "UDP");
    }

    #[tokio::test]
    async fn test_diagnostics_report_markdown() {
        let report = run_diagnostics(None, None).await;
        let md = report.to_markdown();
        assert!(md.contains("Debugger do Pato"));
        assert!(md.contains("Portas do Protocolo"));
        let json = report.to_json();
        assert!(json.contains("interfaces"));
    }
}
