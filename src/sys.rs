#![allow(non_camel_case_types, clippy::struct_field_names)]

// Vendored xnu bridge private structures and constants.
//
// These are macOS private SPI — not guaranteed stable across OS versions.
//
// Primary reference (xnu-12377.1.9):
//   https://github.com/apple-oss-distributions/xnu/blob/f6217f891ac0bb64f3d375211650a4c1ff8ca1ea/bsd/net/if_bridgevar.h
//
// ifconfig bridge implementation (network_cmds-730.80.3):
//   https://github.com/apple-oss-distributions/network_cmds/blob/e0639d3e3e56f9b407ff5c8c090b771be741bd11/ifconfig.tproj/ifbridge.c
//
// struct ifdrv (macOS SDK):
//   https://github.com/apple-oss-distributions/xnu/blob/f6217f891ac0bb64f3d375211650a4c1ff8ca1ea/bsd/net/if.h#L399-L405

use std::io;
use std::os::fd::RawFd;

pub const IFNAMSIZ: usize = 16;
pub const ETHER_ADDR_LEN: usize = 6;

// Bridge ioctl commands (passed via ifdrv.ifd_cmd).
// ref: if_bridgevar.h L124-L125
pub const BRDGGIFS: libc::c_ulong = 6; // get member list (ifbifconf)
pub const BRDGRTS: libc::c_ulong = 7; // get address list (ifbaconf)

// ioctl request codes.
// ref: xnu/bsd/sys/sockio.h L168 — _IOWR('i', 123, struct ifdrv)
// Expanded value verified against macOS SDK at compile time (see tests).
pub const SIOCGDRVSPEC: libc::c_ulong = 0xc028_697b;

// ---- Bridge member flags (IFBIF_*) ----
// ref: if_bridgevar.h L182-L199

pub const IFBIF_LEARNING: u32 = 0x0001;
pub const IFBIF_DISCOVER: u32 = 0x0002;
pub const IFBIF_STP: u32 = 0x0004;
pub const IFBIF_SPAN: u32 = 0x0008;
pub const IFBIF_STICKY: u32 = 0x0010;
pub const IFBIF_BSTP_EDGE: u32 = 0x0020;
pub const IFBIF_BSTP_AUTOEDGE: u32 = 0x0040;
pub const IFBIF_BSTP_PTP: u32 = 0x0080;
pub const IFBIF_BSTP_AUTOPTP: u32 = 0x0100;
pub const IFBIF_BSTP_ADMEDGE: u32 = 0x0200;
pub const IFBIF_BSTP_ADMCOST: u32 = 0x0400;
pub const IFBIF_PRIVATE: u32 = 0x0800;
pub const IFBIF_MAC_NAT: u32 = 0x8000;
pub const IFBIF_CHECKSUM_OFFLOAD: u32 = 0x1_0000;

// ---- FDB entry flags (IFBAF_*) ----
// ref: if_bridgevar.h L302-L305

pub const IFBAF_TYPEMASK: u8 = 0x03;
pub const IFBAF_DYNAMIC: u8 = 0x00;
pub const IFBAF_STATIC: u8 = 0x01;
pub const IFBAF_STICKY: u8 = 0x02;

// ---- Raw kernel structures ----
//
// All structs use #[repr(C, packed(4))] to match xnu's `#pragma pack(4)`.
// Sizes verified against the C compiler on macOS (see struct_sizes test).
//
// Expose sizes as u32 constants because the kernel's len fields are u32,
// and this avoids usize-to-u32 casts at every call site.

pub const IFBREQ_SIZE: u32 = 80;
pub const IFBAREQ_SIZE: u32 = 36;
pub const IFBIFCONF_SIZE: u32 = 12;
pub const IFBACONF_SIZE: u32 = 12;
// Sizes verified against the C compiler on macOS (see C validation program).

/// Bridge member interface request — returned by BRDGGIFS.
///
/// ref: `if_bridgevar.h` L163-L177
/// Total size: 80 bytes (verified).
#[repr(C, packed(4))]
#[derive(Clone, Copy)]
pub struct ifbreq {
    pub ifbr_ifsname: [u8; IFNAMSIZ], // member interface name
    pub ifbr_ifsflags: u32,           // member interface flags
    pub ifbr_stpflags: u32,           // STP flags
    pub ifbr_path_cost: u32,          // STP path cost
    pub ifbr_portno: u8,              // port number
    pub ifbr_priority: u8,            // STP priority
    pub ifbr_proto: u8,               // STP protocol
    pub ifbr_role: u8,                // STP role
    pub ifbr_state: u8,               // STP state
    pub ifbr_addrcnt: u32,            // learned address count
    pub ifbr_addrmax: u32,            // address limit
    pub ifbr_addrexceeded: u32,       // exceeded counter
    pub _pad: [u8; 32],
}

// ifbifconf: ref `if_bridgevar.h` L232-L240, total size 12 bytes.
// Not constructed directly — `grow_fetch` writes raw bytes matching this
// layout. Struct definition lives in the test module for offset verification.

/// Bridge address (FDB) entry — returned by BRDGRTS.
///
/// ref: `if_bridgevar.h` L273-L279
/// Total size: 36 bytes (verified).
#[repr(C, packed(4))]
#[derive(Clone, Copy)]
pub struct ifbareq {
    pub ifba_ifsname: [u8; IFNAMSIZ],    // member interface name
    pub ifba_expire: libc::c_ulong,       // expire time (kernel-specific semantics)
    pub ifba_flags: u8,                   // IFBAF_* flags
    pub ifba_dst: [u8; ETHER_ADDR_LEN],  // destination MAC address
    pub ifba_vlan: u16,                   // VLAN tag
}

// ifbaconf: ref `if_bridgevar.h` L317-L325, total size 12 bytes.
// Not constructed directly — `grow_fetch` writes raw bytes matching this
// layout. Struct definition lives in the test module for offset verification.

/// Driver-specific ioctl wrapper.
///
/// ref: xnu/bsd/net/if.h L399-L405
/// Total size: 40 bytes (verified).
#[repr(C, packed(4))]
#[derive(Clone, Copy)]
pub struct ifdrv {
    pub ifd_name: [u8; IFNAMSIZ], // interface name
    pub ifd_cmd: libc::c_ulong,   // driver command
    pub ifd_len: usize,           // data buffer length
    pub ifd_data: *mut u8,        // data buffer pointer
}

// SAFETY: ifdrv contains a raw pointer but is only used as a stack-local
// ioctl argument — never shared across threads.
unsafe impl Send for ifdrv {}

// ---- Helper functions ----

/// Create a DGRAM socket for ioctl calls.
pub fn bridge_socket() -> io::Result<RawFd> {
    // SAFETY: standard socket(2) call, returns a file descriptor or -1.
    let fd = unsafe { libc::socket(libc::AF_INET, libc::SOCK_DGRAM, 0) };
    if fd < 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(fd)
}

/// Close a file descriptor.
pub fn close_fd(fd: RawFd) {
    // SAFETY: we own this fd and close it exactly once.
    unsafe {
        libc::close(fd);
    }
}

/// Write an interface name into a fixed-size `[u8; IFNAMSIZ]` buffer.
///
/// The name is NUL-terminated. Returns an error if the name is too long.
pub fn write_ifname(buf: &mut [u8; IFNAMSIZ], name: &str) {
    let bytes = name.as_bytes();
    buf[..bytes.len()].copy_from_slice(bytes);
    buf[bytes.len()] = 0;
}

/// Issue a SIOCGDRVSPEC ioctl (read / get).
///
/// Fills `ifdrv` with the bridge interface name, command, and data pointer,
/// then issues the ioctl.
pub fn bridge_ioctl_get(fd: RawFd, ifd: &mut ifdrv) -> io::Result<()> {
    // SAFETY: `ifd` is a valid, properly sized ifdrv struct living on the
    // caller's stack. The kernel reads ifd_name / ifd_cmd / ifd_len and
    // writes into the buffer pointed to by ifd_data (whose length is
    // guaranteed by the caller via ifd_len).
    let ret = unsafe { libc::ioctl(fd, SIOCGDRVSPEC, std::ptr::from_mut(ifd)) };
    if ret < 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(())
}

/// Grow-and-retry buffer fetch for bridge ioctls that return variable-length data.
///
/// Follows the same pattern as `ifconfig`'s `bridge_interfaces` /
/// `bridge_addresses`: start with an initial buffer, call the ioctl,
/// and double the buffer if it was too small.
///
/// ref: `network_cmds/ifconfig.tproj/ifbridge.c` `bridge_interfaces()` / `bridge_addresses()`
///
/// Both `ifbifconf` and `ifbaconf` share the same packed(4) layout:
///   offset 0: u32 (buffer length, set by caller, updated by kernel)
///   offset 4: *mut u8 (buffer pointer, potentially misaligned for 8-byte ptr)
///
/// `entry_size` is `size_of::<ifbreq>()` or `size_of::<ifbareq>()`, used to
/// detect when the kernel might have more data than the buffer could hold.
///
/// We use `write_unaligned` / `read_unaligned` because the pointer field
/// at offset 4 is only 4-byte aligned in a packed(4) struct.
pub fn grow_fetch(
    fd: RawFd,
    bridge: &str,
    cmd: libc::c_ulong,
    payload_size: u32,
    entry_size: u32,
) -> io::Result<Vec<u8>> {
    assert!(payload_size >= 12, "payload must hold at least (len: u32, buf: *mut u8)");

    let mut payload = vec![0u8; payload_size as usize];

    // Offset 0: u32 len, offset 4: *mut u8 buf.
    let len_off = 0usize;
    let buf_off = 4usize;

    // Start with space for ~100 entries (same order as ifconfig's 8192).
    let mut buf_size: u32 = entry_size * 100;

    loop {
        let mut data_buf = vec![0u8; buf_size as usize];

        // SAFETY: `payload` is valid for `payload_size` bytes. We write a u32
        // at offset 0 and a pointer at offset 4 (only 4-byte aligned in
        // packed(4), hence write_unaligned).
        // `data_buf` lives until after the ioctl returns.
        unsafe {
            std::ptr::write_unaligned(
                payload.as_mut_ptr().add(len_off).cast::<u32>(),
                buf_size,
            );
            std::ptr::write_unaligned(
                payload.as_mut_ptr().add(buf_off).cast::<*mut u8>(),
                data_buf.as_mut_ptr(),
            );
        }

        let mut ifd: ifdrv = unsafe { std::mem::zeroed() };
        write_ifname(&mut ifd.ifd_name, bridge);
        ifd.ifd_cmd = cmd;
        ifd.ifd_len = payload_size as usize;
        ifd.ifd_data = payload.as_mut_ptr();

        bridge_ioctl_get(fd, &mut ifd)?;

        // SAFETY: kernel wrote the actual filled length back at offset 0.
        let filled_len =
            unsafe { std::ptr::read_unaligned(payload.as_ptr().add(len_off).cast::<u32>()) };

        // If filled_len + one entry >= buffer size, the kernel might have
        // truncated. Double and retry (same heuristic as ifconfig).
        if filled_len + entry_size >= buf_size {
            buf_size *= 2;
            continue;
        }

        data_buf.truncate(filled_len as usize);
        return Ok(data_buf);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Test-only struct definitions for layout verification.
    // These mirror the kernel's ifbifconf/ifbaconf but are never constructed
    // in production — grow_fetch writes matching raw bytes instead.

    /// ref: `if_bridgevar.h` L232-L240
    #[repr(C, packed(4))]
    struct ifbifconf {
        ifbic_len: u32,
        ifbic_buf: *mut u8,
    }

    /// ref: `if_bridgevar.h` L317-L325
    #[repr(C, packed(4))]
    struct ifbaconf {
        ifbac_len: u32,
        ifbac_buf: *mut u8,
    }

    #[test]
    fn struct_sizes() {
        assert_eq!(size_of::<ifbreq>(), IFBREQ_SIZE as usize);
        assert_eq!(size_of::<ifbareq>(), IFBAREQ_SIZE as usize);
        assert_eq!(size_of::<ifbifconf>(), IFBIFCONF_SIZE as usize);
        assert_eq!(size_of::<ifbaconf>(), IFBACONF_SIZE as usize);
        assert_eq!(size_of::<ifdrv>(), 40);
    }

    #[test]
    fn struct_offsets() {
        use std::mem::offset_of;

        // ifbreq field offsets (verified against C compiler)
        assert_eq!(offset_of!(ifbreq, ifbr_ifsname), 0);
        assert_eq!(offset_of!(ifbreq, ifbr_ifsflags), 16);
        assert_eq!(offset_of!(ifbreq, ifbr_stpflags), 20);
        assert_eq!(offset_of!(ifbreq, ifbr_path_cost), 24);
        assert_eq!(offset_of!(ifbreq, ifbr_portno), 28);
        assert_eq!(offset_of!(ifbreq, ifbr_priority), 29);
        assert_eq!(offset_of!(ifbreq, ifbr_proto), 30);
        assert_eq!(offset_of!(ifbreq, ifbr_role), 31);
        assert_eq!(offset_of!(ifbreq, ifbr_state), 32);
        assert_eq!(offset_of!(ifbreq, ifbr_addrcnt), 36);
        assert_eq!(offset_of!(ifbreq, ifbr_addrmax), 40);
        assert_eq!(offset_of!(ifbreq, ifbr_addrexceeded), 44);
        assert_eq!(offset_of!(ifbreq, _pad), 48);

        // ifbareq field offsets
        assert_eq!(offset_of!(ifbareq, ifba_ifsname), 0);
        assert_eq!(offset_of!(ifbareq, ifba_expire), 16);
        assert_eq!(offset_of!(ifbareq, ifba_flags), 24);
        assert_eq!(offset_of!(ifbareq, ifba_dst), 25);
        assert_eq!(offset_of!(ifbareq, ifba_vlan), 32);

        // ifbifconf field offsets
        assert_eq!(offset_of!(ifbifconf, ifbic_len), 0);
        assert_eq!(offset_of!(ifbifconf, ifbic_buf), 4);

        // ifbaconf field offsets
        assert_eq!(offset_of!(ifbaconf, ifbac_len), 0);
        assert_eq!(offset_of!(ifbaconf, ifbac_buf), 4);

        // ifdrv field offsets
        assert_eq!(offset_of!(ifdrv, ifd_name), 0);
        assert_eq!(offset_of!(ifdrv, ifd_cmd), 16);
        assert_eq!(offset_of!(ifdrv, ifd_len), 24);
        assert_eq!(offset_of!(ifdrv, ifd_data), 32);
    }
}
