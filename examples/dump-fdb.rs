/// Dump all bridges, their members, and FDB entries.
///
/// Run with: cargo run --example dump-fdb
/// (may require root for bridge ioctl access)
fn main() {
    let bridges = match ifbridge::list_bridges() {
        Ok(b) => b,
        Err(e) => {
            eprintln!("failed to list bridges: {e}");
            std::process::exit(1);
        }
    };

    if bridges.is_empty() {
        println!("no bridge interfaces found");
        return;
    }

    for bridge in &bridges {
        println!("=== {bridge} ===");

        match ifbridge::list_members(bridge) {
            Ok(members) => {
                println!("  members:");
                for m in &members {
                    println!("    {} flags={:?}", m.name, m.flags);
                }
            }
            Err(e) => eprintln!("  failed to list members: {e}"),
        }

        match ifbridge::list_fdb(bridge) {
            Ok(entries) => {
                println!("  FDB ({} entries):", entries.len());
                for e in &entries {
                    let expire = match e.expires_in {
                        Some(d) => format!("{}s", d.as_secs()),
                        None if e.flags.is_static() => "static".to_string(),
                        None => "0".to_string(),
                    };
                    println!(
                        "    {} on {} vlan={} flags={:?} expire={}",
                        e.mac, e.member, e.vlan, e.flags, expire
                    );
                }
            }
            Err(e) => eprintln!("  failed to list FDB: {e}"),
        }

        println!();
    }
}
