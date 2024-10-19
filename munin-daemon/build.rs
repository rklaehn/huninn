use std::str::FromStr;

const MUNIN_ALLOWED_NODES: &str = "MUNIN_ALLOWED_NODES";

fn main() {
    println!("cargo:rerun-if-env-changed={MUNIN_ALLOWED_NODES}");
    if let Ok(x) = std::env::var(MUNIN_ALLOWED_NODES) {
        let nodes = x
            .split(',')
            .map(iroh_base::key::NodeId::from_str)
            .collect::<Result<Vec<_>, _>>();
        if let Err(e) = nodes {
            eprintln!("{MUNIN_ALLOWED_NODES} is not a valid comma separated list of node ids");
            eprintln!("  {}", x);
            eprintln!("  {}", e);
            std::process::exit(1);
        }
    }
}
