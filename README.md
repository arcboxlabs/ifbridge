# ifbridge

`ifbridge` is a macOS-only Rust crate for reading bridge member and forwarding
database state via private bridge ioctls used by Apple's `ifconfig`.

## Why

The standard way to inspect bridge state on macOS is to parse `ifconfig`
text output. This is fragile, locale-dependent, and unnecessarily slow.

Under the hood, `ifconfig` itself talks to the kernel through
`SIOCGDRVSPEC` / `SIOCSDRVSPEC` ioctls defined in xnu's
`bsd/net/if_bridgevar.h`. This crate calls those ioctls directly and
returns typed Rust structs.

## API

```rust
// Enumerate all bridgeN interfaces on the system.
pub fn list_bridges() -> io::Result<Vec<String>>;

// List member interfaces of a bridge.
pub fn list_members(bridge: &str) -> io::Result<Vec<BridgeMember>>;

// List forwarding database (FDB) entries of a bridge.
pub fn list_fdb(bridge: &str) -> io::Result<Vec<BridgeEntry>>;

// Find which bridge has learned a given MAC address.
pub fn find_bridge_by_mac(target: MacAddr) -> io::Result<Option<String>>;
```

## Example

```rust,no_run
fn main() -> std::io::Result<()> {
    for bridge in ifbridge::list_bridges()? {
        println!("{bridge}:");
        for m in ifbridge::list_members(&bridge)? {
            println!("  member {} flags={:?}", m.name, m.flags);
        }
        for e in ifbridge::list_fdb(&bridge)? {
            println!("  fdb {} on {} vlan={}", e.mac, e.member, e.vlan);
        }
    }
    Ok(())
}
```

## Platform

macOS only (Apple Silicon and Intel). Attempting to compile on other
platforms will produce a `compile_error!`.

## Risk

This crate depends on **xnu bridge private SPI** (`bsd/net/if_bridgevar.h`).
Apple does not guarantee ABI or API stability for these interfaces.
A future macOS release may change struct layouts, ioctl semantics, or
remove these interfaces entirely. Pin your macOS deployment target and
test on each OS update.

## License

MIT OR Apache-2.0
