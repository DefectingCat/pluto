use anyhow::Result;
use clap::Parser;
use pluto::{error::PlutoError, PingMethod, Pluto};
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
    /// The protocol will used, http or tcp
    #[arg(short, long, value_enum, default_value_t = PingMethod::Tcp)]
    method: PingMethod,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let host = args.host.ok_or(PlutoError::ArgsError("no host"))?;

    // Total time
    let start = Instant::now();

    let mut pluto = Pluto::build(args.method, host, args.port);

    for _ in 0..args.count {
        match pluto.ping() {
            Ok(_) => {}
            Err(err) => {
                eprintln!("Send package failed {}", err)
            }
        };
    }

    Ok(())
}
