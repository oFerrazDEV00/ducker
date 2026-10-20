//! # ducker-core
//!
//! Implementação em Rust do protocolo [LocalSend v2](https://github.com/localsend/protocol).
//! Cada dispositivo roda um [`Node`], que é servidor e cliente ao mesmo tempo.
//! Veja `HANDOFF.md` na raiz do repositório para a visão geral da arquitetura.

pub mod client;
pub mod diagnostics;
pub mod discovery;
pub mod duckive;
pub mod error;
pub mod identity;
pub mod model;
pub mod node;
pub mod quack;
mod server;
pub mod session;
pub mod tls;

pub use diagnostics::{
    check_local_ports, get_network_interfaces, probe_peer, run_diagnostics, DiagnosticAnomaly,
    MulticastStatus, NetworkHealthReport, NetworkInterfaceInfo, PeerDiagnosticProbe, PortStatus,
    TrafficEventSummary,
};
pub use duckive::{
    get_achievements_catalog, get_cosmetics_catalog, AchievementDefinition, AchievementState,
    AdminPermissions, AdminSession, CosmeticCategory, CosmeticItem, CosmeticItemStatus,
    DuckiveProfile, DuckiveView, EquippedItems, LevelInfo, Rarity, UnlockRequirement,
    UnlockSummary, DUCKIVE_FILE,
};
pub use error::{DuckerError, Result};
pub use identity::{default_alias, Identity};
pub use model::{DeviceInfo, DeviceType, FileDto, Protocol, DEFAULT_PORT, PROTOCOL_VERSION};
pub use node::{
    collect_files, count_files_and_bytes, default_save_dir, Node, NodeConfig, NodeEvent, Peer,
    SendOptions, SendProgress,
};
pub use quack::{QuackFile, QuackOptions, QuackPlan, QuackSource, QuackTarget, ResolvedQuackFile};
