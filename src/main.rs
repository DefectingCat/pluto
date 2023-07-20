use anyhow::{Ok, Result};
use clap::Parser;
use pluto::{error::PlutoError, Pluto};
use tokio::time::Instant;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Target host address
    host: Option<String>,
    /// Target host port
    #[arg(default_value_t = 80)]
    port: u32,
    /// Total package send count
    #[arg(short, long, default_value_t = 4)]
    count: usize,
    /// Use http protocol
    #[arg(short = 'H', long)]
    http: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let host = args.host.ok_or(PlutoError::ArgsError("no host"))?;

    // Total time
    let start = Instant::now();

    let method = if args.http { "http" } else { "tcp" };
    let pluto = Pluto::build(method);

    for i in 0..args.count {
        println!("{}", i);
    }

    Ok(())
}
