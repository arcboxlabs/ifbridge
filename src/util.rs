use std::io;

use crate::sys::IFNAMSIZ;

/// Validate a bridge interface name.
///
/// Requirements:
/// - Non-empty
/// - Starts with "bridge"
/// - Total length < IFNAMSIZ (must leave room for NUL terminator)
/// - ASCII only (interface names are always ASCII)
pub fn validate_bridge_name(name: &str) -> io::Result<()> {
    if name.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "bridge name is empty",
        ));
    }

    if !name.starts_with("bridge") {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("interface name '{name}' does not start with 'bridge'"),
        ));
    }

    if name.len() >= IFNAMSIZ {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!(
                "interface name '{name}' too long ({} >= {IFNAMSIZ})",
                name.len(),
            ),
        ));
    }

    if !name.is_ascii() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("interface name '{name}' contains non-ASCII characters"),
        ));
    }

    if name.contains('\0') {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "interface name contains embedded NUL byte",
        ));
    }

    Ok(())
}

/// Extract a NUL-terminated C string from a fixed-size byte buffer.
///
/// Returns the string up to (but not including) the first NUL byte,
/// or the entire buffer if no NUL is found.
pub fn cstr_from_buf(buf: &[u8]) -> &str {
    let len = buf.iter().position(|&b| b == 0).unwrap_or(buf.len());
    // Interface names are always ASCII, so this won't panic.
    std::str::from_utf8(&buf[..len]).unwrap_or("")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_bridge_names() {
        assert!(validate_bridge_name("bridge0").is_ok());
        assert!(validate_bridge_name("bridge104").is_ok());
        assert!(validate_bridge_name("bridge99999").is_ok());
    }

    #[test]
    fn rejects_empty() {
        assert!(validate_bridge_name("").is_err());
    }

    #[test]
    fn rejects_non_bridge_prefix() {
        assert!(validate_bridge_name("en0").is_err());
        assert!(validate_bridge_name("lo0").is_err());
        assert!(validate_bridge_name("vmenet0").is_err());
    }

    #[test]
    fn rejects_embedded_nul() {
        assert!(validate_bridge_name("bridge0\0junk").is_err());
    }

    #[test]
    fn rejects_too_long() {
        // IFNAMSIZ is 16, so max name is 15 chars
        let long_name = "bridge1234567890"; // 16 chars
        assert!(validate_bridge_name(long_name).is_err());
    }

    #[test]
    fn cstr_extraction() {
        let buf = [b'e', b'n', b'0', 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        assert_eq!(cstr_from_buf(&buf), "en0");
    }

    #[test]
    fn cstr_no_nul() {
        let buf = [b'e', b'n', b'0'];
        assert_eq!(cstr_from_buf(&buf), "en0");
    }
}
