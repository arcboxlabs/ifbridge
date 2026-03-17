use crate::sys;

bitflags::bitflags! {
    /// Flags for a bridge member interface (IFBIF_*).
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct BridgeMemberFlags: u32 {
        const LEARNING         = sys::IFBIF_LEARNING;
        const DISCOVER         = sys::IFBIF_DISCOVER;
        const STP              = sys::IFBIF_STP;
        const SPAN             = sys::IFBIF_SPAN;
        const STICKY           = sys::IFBIF_STICKY;
        const BSTP_EDGE        = sys::IFBIF_BSTP_EDGE;
        const BSTP_AUTOEDGE    = sys::IFBIF_BSTP_AUTOEDGE;
        const BSTP_PTP         = sys::IFBIF_BSTP_PTP;
        const BSTP_AUTOPTP     = sys::IFBIF_BSTP_AUTOPTP;
        const BSTP_ADMEDGE     = sys::IFBIF_BSTP_ADMEDGE;
        const BSTP_ADMCOST     = sys::IFBIF_BSTP_ADMCOST;
        const PRIVATE          = sys::IFBIF_PRIVATE;
        const MAC_NAT          = sys::IFBIF_MAC_NAT;
        const CHECKSUM_OFFLOAD = sys::IFBIF_CHECKSUM_OFFLOAD;
    }
}

bitflags::bitflags! {
    /// Flags for a bridge FDB entry (IFBAF_*).
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct BridgeEntryFlags: u8 {
        const STATIC  = sys::IFBAF_STATIC;
        const STICKY_ = sys::IFBAF_STICKY;
    }
}

impl BridgeEntryFlags {
    /// Returns true if this is a dynamically learned entry.
    #[must_use]
    pub fn is_dynamic(self) -> bool {
        (self.bits() & sys::IFBAF_TYPEMASK) == sys::IFBAF_DYNAMIC
    }

    /// Returns true if this is a statically configured entry.
    #[must_use]
    pub fn is_static(self) -> bool {
        (self.bits() & sys::IFBAF_TYPEMASK) == sys::IFBAF_STATIC
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn member_flags_from_bits() {
        let flags = BridgeMemberFlags::from_bits_truncate(0x0003);
        assert!(flags.contains(BridgeMemberFlags::LEARNING));
        assert!(flags.contains(BridgeMemberFlags::DISCOVER));
        assert!(!flags.contains(BridgeMemberFlags::STP));
    }

    #[test]
    fn entry_flags_dynamic() {
        let flags = BridgeEntryFlags::from_bits_truncate(0x00);
        assert!(flags.is_dynamic());
        assert!(!flags.is_static());
    }

    #[test]
    fn entry_flags_static() {
        let flags = BridgeEntryFlags::from_bits_truncate(0x01);
        assert!(flags.is_static());
        assert!(!flags.is_dynamic());
    }
}
