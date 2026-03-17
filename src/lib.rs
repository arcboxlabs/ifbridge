//! `ifbridge` — macOS-only typed access to bridge(4) forwarding database.
//!
//! This crate provides direct access to the bridge forwarding database (FDB)
//! and member interface list via the private `SIOCGDRVSPEC` ioctls that Apple's
//! `ifconfig` uses internally. No text parsing required.
//!
//! # Platform
//!
//! macOS only. This crate uses xnu private SPI (`bsd/net/if_bridgevar.h`).
//! Apple does not guarantee ABI stability for these interfaces across OS
//! versions.
//!
//! # Example
//!
//! ```no_run
//! let bridges = ifbridge::list_bridges().unwrap();
//! for bridge in &bridges {
//!     let members = ifbridge::list_members(bridge).unwrap();
//!     let fdb = ifbridge::list_fdb(bridge).unwrap();
//!     println!("{bridge}: {} members, {} FDB entries", members.len(), fdb.len());
//! }
//! ```

#[cfg(not(target_os = "macos"))]
compile_error!("ifbridge only supports macOS (uses xnu bridge private SPI)");

mod bridge;
mod enumerate;
mod flags;
mod mac;
pub(crate) mod sys;
mod util;

pub use bridge::{find_bridge_by_mac, list_fdb, list_members, BridgeEntry, BridgeMember};
pub use enumerate::list_bridges;
pub use flags::{BridgeEntryFlags, BridgeMemberFlags};
pub use mac::{MacAddr, ParseMacAddrError};
