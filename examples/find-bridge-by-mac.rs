/// Look up which bridge has learned a given MAC address.
///
/// Usage: cargo run --example find-bridge-by-mac -- aa:bb:cc:dd:ee:ff
/// (may require root)
fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        eprintln!("usage: {} <mac-address>", args[0]);
        eprintln!("  e.g. {} aa:bb:cc:dd:ee:ff", args[0]);
        std::process::exit(1);
    }

    let mac: ifbridge::MacAddr = match args[1].parse() {
        Ok(m) => m,
        Err(e) => {
            eprintln!("invalid MAC address '{}': {e}", args[1]);
            std::process::exit(1);
        }
    };

    match ifbridge::find_bridge_by_mac(mac) {
        Ok(Some(bridge)) => println!("{mac} found on {bridge}"),
        Ok(None) => {
            println!("{mac} not found on any bridge");
            std::process::exit(1);
        }
        Err(e) => {
            eprintln!("error: {e}");
            std::process::exit(1);
        }
    }
}
