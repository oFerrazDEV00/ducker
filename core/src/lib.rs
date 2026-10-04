pub mod discovery;
pub mod error;
pub mod identity;
pub mod mobile_bridge;
pub mod protocol;
pub mod transfer;

pub use discovery::{DiscoveredDevice, DiscoveryManager};
pub use error::{DuckerError, ErrorCode};
pub use identity::DeviceIdentity;
pub use mobile_bridge::{MobileBridge, MobileBridgeState, MobileWsMessage, DEFAULT_MOBILE_BRIDGE_PORT};
pub use protocol::{
    DiscoveryAnnouncement, HandshakeStatus, TransferHandshakeRequest,
    TransferHandshakeResponse, TransferResult, CURRENT_PROTOCOL_VERSION,
    DEFAULT_DISCOVERY_PORT, DEFAULT_TRANSFER_PORT,
};
pub use transfer::{FileReceiver, FileSender, TransferEvent};
