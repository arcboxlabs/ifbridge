use std::io;
use std::time::Duration;

use crate::enumerate::list_bridges;
use crate::flags::{BridgeEntryFlags, BridgeMemberFlags};
use crate::mac::MacAddr;
use crate::sys::{self, ifbareq, ifbreq};
use crate::util::{cstr_from_buf, validate_bridge_name};

/// A member interface of a bridge.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BridgeMember {
    pub name: String,
    pub flags: BridgeMemberFlags,
}

/// An entry in the bridge forwarding database (FDB).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BridgeEntry {
    pub member: String,
    pub mac: MacAddr,
    pub vlan: u16,
    pub flags: BridgeEntryFlags,
    /// Remaining time before this entry expires.
    ///
    /// `None` if the entry is static or the kernel returned 0.
    /// The exact semantics are kernel-version-dependent.
    pub expires_in: Option<Duration>,
}

/// List member interfaces of a bridge.
///
/// Issues `BRDGGIFS` via `SIOCGDRVSPEC` to get the member list.
///
/// # Errors
///
/// Returns an error if the bridge name is invalid or the ioctl fails.
pub fn list_members(bridge: &str) -> io::Result<Vec<BridgeMember>> {
    validate_bridge_name(bridge)?;

    let fd = sys::bridge_socket()?;
    let result = list_members_inner(fd, bridge);
    sys::close_fd(fd);
    result
}

fn list_members_inner(fd: i32, bridge: &str) -> io::Result<Vec<BridgeMember>> {
    let data = sys::grow_fetch(
        fd,
        bridge,
        sys::BRDGGIFS,
        sys::IFBIFCONF_SIZE,
        sys::IFBREQ_SIZE,
    )?;

    if data.is_empty() {
        return Ok(Vec::new());
    }

    let entry_size = size_of::<ifbreq>();
    let count = data.len() / entry_size;
    let mut members = Vec::with_capacity(count);

    for i in 0..count {
        // SAFETY: `data` is a contiguous buffer of `count` ifbreq structs.
        // We copy bytes into a zeroed struct to avoid alignment issues
        // (ifbreq is packed(4), but data pointer is only 1-byte aligned).
        let entry: ifbreq = unsafe {
            let mut entry: ifbreq = std::mem::zeroed();
            std::ptr::copy_nonoverlapping(
                data.as_ptr().add(i * entry_size),
                std::ptr::from_mut(&mut entry).cast::<u8>(),
                entry_size,
            );
            entry
        };

        let name = cstr_from_buf(&entry.ifbr_ifsname).to_owned();
        let flags = BridgeMemberFlags::from_bits_truncate(entry.ifbr_ifsflags);

        members.push(BridgeMember { name, flags });
    }

    Ok(members)
}

/// List forwarding database entries of a bridge.
///
/// Issues `BRDGRTS` via `SIOCGDRVSPEC` to get the FDB.
///
/// # Errors
///
/// Returns an error if the bridge name is invalid or the ioctl fails.
pub fn list_fdb(bridge: &str) -> io::Result<Vec<BridgeEntry>> {
    validate_bridge_name(bridge)?;

    let fd = sys::bridge_socket()?;
    let result = list_fdb_inner(fd, bridge);
    sys::close_fd(fd);
    result
}

fn list_fdb_inner(fd: i32, bridge: &str) -> io::Result<Vec<BridgeEntry>> {
    let data = sys::grow_fetch(
        fd,
        bridge,
        sys::BRDGRTS,
        sys::IFBACONF_SIZE,
        sys::IFBAREQ_SIZE,
    )?;

    if data.is_empty() {
        return Ok(Vec::new());
    }

    let entry_size = size_of::<ifbareq>();
    let count = data.len() / entry_size;
    let mut entries = Vec::with_capacity(count);

    for i in 0..count {
        // SAFETY: same as list_members — copying bytes into a zeroed struct
        // to avoid alignment issues with packed(4) structs.
        let entry: ifbareq = unsafe {
            let mut entry: ifbareq = std::mem::zeroed();
            std::ptr::copy_nonoverlapping(
                data.as_ptr().add(i * entry_size),
                std::ptr::from_mut(&mut entry).cast::<u8>(),
                entry_size,
            );
            entry
        };

        let member = cstr_from_buf(&entry.ifba_ifsname).to_owned();
        let mac = MacAddr::new(entry.ifba_dst);
        let vlan = entry.ifba_vlan;
        let flags = BridgeEntryFlags::from_bits_truncate(entry.ifba_flags);

        let expires_in = if entry.ifba_expire == 0 {
            None
        } else {
            Some(Duration::from_secs(entry.ifba_expire as u64))
        };

        entries.push(BridgeEntry {
            member,
            mac,
            vlan,
            flags,
            expires_in,
        });
    }

    Ok(entries)
}

/// Find the bridge that has learned a specific MAC address.
///
/// Enumerates all bridges, queries each FDB, and returns the first bridge
/// that contains the target MAC. Returns `None` if no bridge has it.
///
/// # Errors
///
/// Returns an error if bridge enumeration or FDB query fails.
pub fn find_bridge_by_mac(target: MacAddr) -> io::Result<Option<String>> {
    let bridges = list_bridges()?;
    for bridge in bridges {
        let entries = list_fdb(&bridge)?;
        if entries.iter().any(|e| e.mac == target) {
            return Ok(Some(bridge));
        }
    }
    Ok(None)
}
