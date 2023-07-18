use clap::Parser;
use pluto::Pluto;
use tokio::time::Instant;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Use http protocol
    #[arg(short = 'H', long)]
    http: bool,
}

fn main() {
    let args = Args::parse();
    println!("{}", args.http);

    let start = Instant::now();

    let method = if args.http { "http" } else { "tcp" };

    let pluto = Pluto::build(method);
    println!("Hello, world!");
    let nanos = start.elapsed().as_nanos();
    let millis = nanos as f32 / 1000000 as f32;
    println!("{}ms", millis);
}
