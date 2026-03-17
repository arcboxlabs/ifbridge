use std::fmt;
use std::str::FromStr;

/// A 6-byte IEEE 802 MAC address.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct MacAddr(pub(crate) [u8; 6]);

impl MacAddr {
    /// Create a `MacAddr` from raw bytes.
    #[must_use]
    pub const fn new(bytes: [u8; 6]) -> Self {
        Self(bytes)
    }

    /// Return the raw bytes.
    #[must_use]
    pub const fn octets(&self) -> [u8; 6] {
        self.0
    }
}

impl fmt::Debug for MacAddr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "MacAddr({self})")
    }
}

impl fmt::Display for MacAddr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let o = &self.0;
        write!(
            f,
            "{:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
            o[0], o[1], o[2], o[3], o[4], o[5]
        )
    }
}

/// Error returned when parsing a MAC address string fails.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseMacAddrError;

impl fmt::Display for ParseMacAddrError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("invalid MAC address")
    }
}

impl std::error::Error for ParseMacAddrError {}

impl FromStr for MacAddr {
    type Err = ParseMacAddrError;

    /// Parse a MAC address from colon-separated hex string (e.g. `"aa:bb:cc:dd:ee:ff"`).
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut octets = [0u8; 6];
        let mut parts = s.split(':');

        for octet in &mut octets {
            let part = parts.next().ok_or(ParseMacAddrError)?;
            *octet = u8::from_str_radix(part, 16).map_err(|_| ParseMacAddrError)?;
        }

        // Reject trailing parts.
        if parts.next().is_some() {
            return Err(ParseMacAddrError);
        }

        Ok(MacAddr(octets))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_roundtrip() {
        let mac = MacAddr::new([0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff]);
        assert_eq!(mac.to_string(), "aa:bb:cc:dd:ee:ff");
        assert_eq!(mac, "aa:bb:cc:dd:ee:ff".parse().unwrap());
    }

    #[test]
    fn display_leading_zeros() {
        let mac = MacAddr::new([0x01, 0x02, 0x03, 0x04, 0x05, 0x06]);
        assert_eq!(mac.to_string(), "01:02:03:04:05:06");
    }

    #[test]
    fn parse_uppercase() {
        let mac: MacAddr = "AA:BB:CC:DD:EE:FF".parse().unwrap();
        assert_eq!(mac.octets(), [0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff]);
    }

    #[test]
    fn parse_rejects_short() {
        assert!("aa:bb:cc:dd:ee".parse::<MacAddr>().is_err());
    }

    #[test]
    fn parse_rejects_long() {
        assert!("aa:bb:cc:dd:ee:ff:00".parse::<MacAddr>().is_err());
    }

    #[test]
    fn parse_rejects_bad_hex() {
        assert!("gg:bb:cc:dd:ee:ff".parse::<MacAddr>().is_err());
    }

    #[test]
    fn parse_rejects_empty() {
        assert!("".parse::<MacAddr>().is_err());
    }

    #[test]
    fn debug_format() {
        let mac = MacAddr::new([0x00; 6]);
        assert_eq!(format!("{mac:?}"), "MacAddr(00:00:00:00:00:00)");
    }
}
