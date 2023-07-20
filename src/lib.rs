pub mod error;
use anyhow::Result;

use std::{io::Write, net::TcpStream};

use clap::ValueEnum;
use tokio::time::Instant;

#[derive(Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
pub enum PingMethod {
    #[default]
    Tcp,
    Http,
}
impl From<&str> for PingMethod {
    fn from(value: &str) -> Self {
        use PingMethod::*;
        match value.to_lowercase().as_str() {
            "tcp" => Tcp,
            "http" => Http,
            _ => Http,
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
    pub host: String,
}
impl Default for Pluto {
    fn default() -> Self {
        Self {
            method: PingMethod::default(),
            port: 80,
            queue: vec![],
            host: String::new(),
        }
    }
}

impl Pluto {
    pub fn build(method: PingMethod, host: String) -> Self {
        Self {
            method,
            host,
            ..Self::default()
        }
    }
    pub fn ping(&self) {
        use PingMethod::*;
        match self.method {
            Http => {}
            Tcp => {
                self.tcp_ping();
            }
        }
    }
    fn tcp_ping(&self) -> Result<()> {
        let mut stream = TcpStream::connect(&self.host)?;

        let bytes = [0u8; 4];
        stream.write_all(&bytes)?;

        Ok(())
    }
}
