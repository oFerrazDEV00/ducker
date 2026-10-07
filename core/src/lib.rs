//! # ducker-core
//!
//! Implementação em Rust do protocolo [LocalSend v2](https://github.com/localsend/protocol).
//! Cada dispositivo roda um [`Node`], que é servidor e cliente ao mesmo tempo.
//! Veja `HANDOFF.md` na raiz do repositório para a visão geral da arquitetura.

pub mod client;
pub mod discovery;
pub mod error;
pub mod identity;
pub mod model;
pub mod node;
pub mod session;
pub mod tls;
mod server;

pub use error::{DuckerError, Result};
pub use identity::{default_alias, Identity};
pub use model::{DeviceInfo, DeviceType, FileDto, Protocol, DEFAULT_PORT, PROTOCOL_VERSION};
pub use node::{default_save_dir, Node, NodeConfig, NodeEvent, Peer, SendOptions, SendProgress};
