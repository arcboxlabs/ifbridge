/// Integration tests for ifbridge.
///
/// These tests require a real macOS system with bridge interfaces.
/// Run with: sudo cargo test -- --ignored
///
/// Tests marked `#[ignore]` are skipped by default in `cargo test`.

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
