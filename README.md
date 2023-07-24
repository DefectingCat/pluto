## Pluto

A tcping tool.

## Examples

### Build with args

```rust
let mut pluto = Pluto::build(args.method, host, args.port);
pluto = Pluto {
    wait: args.wait,
    bytes: args.bytes,
    http_method: args.x,
    ..pluto
};
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
