use pluto::Pluto;
use tokio::time::Instant;

fn main() {
    let start = Instant::now();
    let pluto = Pluto::default();
    println!("Hello, world!");
    let nanos = start.elapsed().as_nanos();
    let millis = nanos as f32 / 1000000 as f32;
    println!("{}", millis);
}
