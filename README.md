## Pluto

A tcping tool.

## Binary usage

Basic:

```bash
❯ pluto google.com -m http -w
Ping http://google.com:80(198.18.1.70:80) - Connected - time=378.73608ms
Ping http://google.com:80(198.18.1.70:80) - Connected - time=370.01617ms
Ping http://google.com:80(198.18.1.70:80) - Connected - time=375.24347ms
Ping http://google.com:80(198.18.1.70:80) - Connected - time=370.21255ms

Ping statistics for google.com:80
4 package sent, 4 package success, 0 package loss
Approximate trip times in milliseconds:
Minimum = 378.73608ms, Maximum = 370.01617ms, Average = 373.55206ms
```

HTTP:

```bash
pluto google.com -m http
```

Waitting response (HTTP only)

```bash
pluto google.com -m http -w
```

And more:

```bash
❯ pluto google.com 443 -m http -w -b 512 -c 5
Ping http://google.com:443(198.18.1.70:443) - Connected - time=516.17883ms
Ping http://google.com:443(198.18.1.70:443) - Connected - time=308.3648ms
Ping http://google.com:443(198.18.1.70:443) - Connected - time=301.67624ms
Ping http://google.com:443(198.18.1.70:443) - Connected - time=267.50793ms
Ping http://google.com:443(198.18.1.70:443) - Connected - time=349.98373ms

Ping statistics for google.com:443
5 package sent, 5 package success, 0 package loss
Approximate trip times in milliseconds:
Minimum = 516.17883ms, Maximum = 267.50793ms, Average = 348.7423ms
```

```bash
❯ pluto -h
A tcping tool

Usage: pluto [OPTIONS] [HOST] [PORT]

Arguments:
  [HOST]  Target host address
  [PORT]  Target host port [default: 80]

Options:
  -c, --count <COUNT>    Total package send count [default: 4]
  -m, --method <METHOD>  The protocol will used, http or tcp [default: tcp] [possible values: tcp, http]
  -w, --wait             Wait http response, only for -m http
  -b, --bytes <BYTES>    Send package size, will add to body with http [default: 56]
  -X, --x <X>            Http request method [default: get] [possible values: get, head, post, put, delete, connect, options, trace, patch]
  -t, --timeout          Ignore count, send packages forever
  -h, --help             Print help
  -V, --version          Print version
```

## Build from source

```bash
cargo build --release
```

Install with cargo

```bash
cargo install --path .
```

### Cross compile

Requirement:

```bash
brew tap SergioBenitez/osxct
brew install FiloSottile/musl-cross/musl-cross
brew install SergioBenitez/osxct/x86_64-unknown-linux-gnu
brew install mingw-w64
```

```bash
rustup target add x86_64-unknown-linux-musl
rustup target add x86_64-unknown-linux-gnu
rustup target add x86_64-pc-windows-gnu
```

Build dynamic link for Linux on MacOS

```bash
TARGET_CC=x86_64-unknown-linux-gnu cargo build --release --target x86_64-unknown-linux-gnu
```

Static link for Linx on MacOS

```bash
TARGET_CC=x86_64-linux-musl-gcc \
RUSTFLAGS="-C linker=x86_64-linux-musl-gcc" \
cargo build --target=x86_64-unknown-linux-musl --release
```

For Windows on MacOS

```bash
cargo build --target=x86_64-pc-windows-gnu --release
```

## Library usage

### Install

```toml
# Cargo.toml
[dependencies]
pluto = { git = "https://github.com/DefectingCat/pluto", branch = 'master' }
```

Minimum dependencies:

```toml
anyhow = "1.0.72"
clap = { version = "4.3.15", features = ["derive"] }
thiserror = "1.0.43"
```

### Build with args

```rust
let mut pluto = Pluto::build(args.method, host, args.port);
pluto = Pluto {
    wait: args.wait,
    bytes: args.bytes,
    http_method: args.x,
    ..pluto
};
pluto.ping().await?;
```

### Full example

```rust
let mut pluto = Pluto::build(args.method, host, args.port);
pluto = Pluto {
    wait: args.wait,
    bytes: args.bytes,
    http_method: args.x,
    ..pluto
};
for _ in 0..args.count {
    match pluto.ping().await {
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
```

## Options

```rust
pub struct Pluto {
    /// Calculate total time
    pub start: Instant,
    /// Connect method, tcp or http
    pub method: PingMethod,
    /// Target host port, default 80
    pub port: u32,
    /// Tcp package queue
    pub queue: Vec<TcpFrame>,
    /// Target host
    pub host: String,
    /// elapsed time millis
    pub elapsed: f32,
    /// Wait target host response, only for http
    pub wait: bool,
    /// Data length
    pub bytes: usize,
    /// The method of http
    pub http_method: HttpMethod,
    /// All results
    pub result: PingResult,
}
```
