use anyhow::Result;
use clap::Parser;
use pluto::{error::PlutoError, HttpMethod, PingMethod, Pluto};

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
    /// Wait http response, only for -m http
    #[arg(short, long)]
    wait: bool,
    /// Send package size, will add to body with http
    #[arg(short, long, default_value_t = 56)]
    bytes: usize,
    /// Http request method
    #[arg(short = 'X', long, value_enum, default_value_t = HttpMethod::GET)]
    x: HttpMethod,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let host = args.host.ok_or(PlutoError::ArgsError("no host"))?;

    let mut pluto = Pluto::build(args.method, host, args.port, args.wait);
    for _ in 0..args.count {
        match pluto.ping() {
            Ok(_) => {}
            Err(err) => {
                eprintln!("Ping {}", err)
            }
        };
    }
    match pluto.end() {
        Ok(_) => {}
        Err(err) => {
            eprintln!("{}", err);
        }
    }

    let len = pluto.queue.len();
    println!();
    println!("Ping statistics for {}", pluto.host);
    println!(
        "{} package sent, {} package success, {} package loss",
        len,
        pluto.result.success,
        len - pluto.result.success
    );
    println!("Approximate trip times in milliseconds:");
    println!(
        "Minimum = {}ms, Maximum = {}ms, Average = {}ms",
        pluto.result.maximum, pluto.result.minimum, pluto.result.average
    );

    Ok(())
}
