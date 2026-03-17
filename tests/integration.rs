/// Integration tests for ifbridge.
///
/// Tests require a real macOS system with bridge interfaces.
/// Run with: sudo cargo test -- --ignored
///
/// Tests marked `#[ignore]` are skipped by default in `cargo test`.

use std::collections::HashSet;
use std::process::Command;

#[test]
fn list_bridges_succeeds() {
    // This should always succeed (may return an empty list).
    let bridges = ifbridge::list_bridges().unwrap();
    for name in &bridges {
        assert!(name.starts_with("bridge"), "unexpected interface: {name}");
    }
}

#[test]
#[ignore]
fn list_members_on_real_bridge() {
    let bridges = ifbridge::list_bridges().unwrap();
    assert!(!bridges.is_empty(), "no bridges found — is vmnet/bridge running?");

    let members = ifbridge::list_members(&bridges[0]).unwrap();
    println!("{}: {} members", bridges[0], members.len());
    for m in &members {
        println!("  {} {:?}", m.name, m.flags);
    }
}

#[test]
#[ignore]
fn list_fdb_on_real_bridge() {
    let bridges = ifbridge::list_bridges().unwrap();
    assert!(!bridges.is_empty(), "no bridges found");

    let entries = ifbridge::list_fdb(&bridges[0]).unwrap();
    println!("{}: {} FDB entries", bridges[0], entries.len());
    for e in &entries {
        println!("  {} on {} vlan={}", e.mac, e.member, e.vlan);
    }
}

#[test]
fn invalid_bridge_name_returns_error() {
    assert!(ifbridge::list_members("en0").is_err());
    assert!(ifbridge::list_members("").is_err());
    assert!(ifbridge::list_fdb("lo0").is_err());
}

#[test]
fn nonexistent_bridge_returns_error() {
    // bridge99999 almost certainly doesn't exist.
    let result = ifbridge::list_members("bridge99999");
    assert!(result.is_err());
}

// ---- Comparison tests (对拍) ----
//
// Compare crate output against `ifconfig` text to verify correctness.
// These parse ifconfig output and check that the crate agrees on core facts.

/// Parse member interface names from `ifconfig <bridge>` output.
///
/// Looks for lines like: `	member: en2 flags=3<LEARNING,DISCOVER>`
fn parse_ifconfig_members(output: &str) -> HashSet<String> {
    let mut members = HashSet::new();
    for line in output.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("member: ") {
            if let Some(name) = rest.split_whitespace().next() {
                members.insert(name.to_owned());
            }
        }
    }
    members
}

/// Parse FDB entries from `ifconfig <bridge>` output.
///
/// Looks for lines in the "Address cache:" section like:
///   `2:2e:d0:c:b0:88 Vlan1 vmenet4 0 flags=0<>`
///
/// Returns a set of (normalized_mac, member_name, vlan) tuples.
fn parse_ifconfig_fdb(output: &str) -> HashSet<(String, String, u16)> {
    let mut entries = HashSet::new();
    let mut in_cache = false;
    for line in output.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("Address cache:") {
            in_cache = true;
            continue;
        }
        // Sections after address cache start without a tab indent at root level.
        if in_cache && !line.starts_with('\t') {
            break;
        }
        if !in_cache {
            continue;
        }
        // Format: "MAC VlanN MEMBER EXPIRE flags=..."
        let parts: Vec<&str> = trimmed.split_whitespace().collect();
        if parts.len() >= 3 && parts[1].starts_with("Vlan") {
            let mac = normalize_mac(parts[0]);
            let vlan: u16 = parts[1]
                .strip_prefix("Vlan")
                .and_then(|v| v.parse().ok())
                .unwrap_or(0);
            let member = parts[2].to_owned();
            entries.insert((mac, member, vlan));
        }
    }
    entries
}

/// Normalize a MAC address to zero-padded lowercase colon-separated format.
///
/// ifconfig outputs `2:2e:d0:c:b0:88`, we normalize to `02:2e:d0:0c:b0:88`.
fn normalize_mac(raw: &str) -> String {
    raw.split(':')
        .map(|octet| format!("{:02x}", u8::from_str_radix(octet, 16).unwrap_or(0)))
        .collect::<Vec<_>>()
        .join(":")
}

#[test]
#[ignore]
fn compare_members_with_ifconfig() {
    let bridges = ifbridge::list_bridges().unwrap();
    assert!(!bridges.is_empty(), "no bridges found");

    for bridge in &bridges {
        let output = Command::new("ifconfig")
            .arg(bridge)
            .output()
            .expect("failed to run ifconfig");
        let stdout = String::from_utf8_lossy(&output.stdout);

        let ifconfig_members = parse_ifconfig_members(&stdout);
        let crate_members: HashSet<String> = ifbridge::list_members(bridge)
            .unwrap()
            .into_iter()
            .map(|m| m.name)
            .collect();

        assert_eq!(
            crate_members, ifconfig_members,
            "member mismatch on {bridge}: crate={crate_members:?} ifconfig={ifconfig_members:?}"
        );
    }
}

#[test]
#[ignore]
fn compare_fdb_with_ifconfig() {
    let bridges = ifbridge::list_bridges().unwrap();
    assert!(!bridges.is_empty(), "no bridges found");

    for bridge in &bridges {
        let output = Command::new("ifconfig")
            .arg(bridge)
            .output()
            .expect("failed to run ifconfig");
        let stdout = String::from_utf8_lossy(&output.stdout);

        let ifconfig_fdb = parse_ifconfig_fdb(&stdout);
        let crate_fdb: HashSet<(String, String, u16)> = ifbridge::list_fdb(bridge)
            .unwrap()
            .into_iter()
            .map(|e| (e.mac.to_string(), e.member, e.vlan))
            .collect();

        // FDB can change between the two calls, so only check that every
        // ifconfig entry exists in the crate output (crate may have more
        // if entries were learned between calls).
        for entry in &ifconfig_fdb {
            assert!(
                crate_fdb.contains(entry),
                "FDB entry from ifconfig not found in crate output on {bridge}: {entry:?}\n\
                 crate has: {crate_fdb:?}"
            );
        }
    }
}
