pub mod error;

use tokio::time::Instant;

#[derive(Default)]
pub enum PingMethod {
    #[default]
    Tcp,
    Http,
    Unknow,
}
impl From<&str> for PingMethod {
    fn from(value: &str) -> Self {
        use PingMethod::*;
        match value.to_lowercase().as_str() {
            "tcp" => Tcp,
            "http" => Http,
            _ => Unknow,
        }
    }
}

pub struct TcpFrame {
    // request start time
    start: Instant,
    // elapsed time millis
    pub elapsed: f32,
}
impl TcpFrame {
    pub fn calculate_delay(&mut self) {
        let nanos = self.start.elapsed().as_nanos();
        let millis = nanos as f32 / 1000000 as f32;
        self.elapsed = millis
    }
}

pub struct Pluto {
    pub method: PingMethod,
    pub port: u32,
    pub queue: Vec<TcpFrame>,
}
impl Default for Pluto {
    fn default() -> Self {
        Self {
            method: PingMethod::default(),
            port: 80,
            queue: vec![],
        }
    }
}

impl Pluto {
    pub fn build(method: &str) -> Self {
        Self {
            method: PingMethod::from(method),
            ..Self::default()
        }
    }
}
