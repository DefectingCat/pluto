use std::{thread, time::Duration};

use anyhow::Result;
use clap::Parser;
use pluto::{error::PlutoError, HttpMethod, PingMethod, Pluto};
use tokio::signal;

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
    /// Ignore count, send packages forever
    #[arg(short, long)]
    timeout: bool,
}

#[tokio::main(worker_threads = 1)]
async fn main() -> Result<()> {
    let args = Args::parse();
    let host = args.host.ok_or(PlutoError::ArgsError("no host"))?;

    let pluto = Pluto::build(args.method, host, args.port);
    let mut pluto = Pluto {
        wait: args.wait,
        bytes: args.bytes,
        http_method: args.x,
        timeout: args.timeout,
        ..pluto
    };

    tokio::select! {
        _ = signal::ctrl_c() => {},
        _ = ping(args.count, args.timeout, &mut pluto) => {}
    }

    match pluto.end() {
        Ok(_) => {}
        Err(err) => {
            eprintln!("{}", err);
        }
    }

    println!();
    println!("Ping statistics for {}", pluto.host);
    println!(
        "{} package sent, {} package success, {} package loss",
        pluto.result.total, pluto.result.success, pluto.result.loss
    );
    println!("Approximate trip times in milliseconds:");
    println!(
        "Minimum = {}ms, Maximum = {}ms, Average = {}ms",
        pluto.result.maximum, pluto.result.minimum, pluto.result.average
    );

    Ok(())
}

async fn ping(arg_count: usize, timeout: bool, pluto: &mut Pluto) {
    let mut count = 0;
    loop {
        if !timeout && count == arg_count {
            break;
        }
        count += 1;
        match pluto.ping().await {
            Ok(_) => {
                thread::sleep(Duration::from_millis(500));
            }
            Err(err) => {
                eprintln!("Ping {}", err)
            }
        };
    }
}
