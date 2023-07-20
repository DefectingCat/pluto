pub mod error;

use anyhow::Result;

use std::{
    net::{TcpStream, ToSocketAddrs},
    time::{Duration, Instant},
};

use clap::ValueEnum;

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

#[derive(Debug)]
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
    pub fn build(method: PingMethod, host: String, port: u32) -> Self {
        let host = format!("{}:{}", host, port);
        Self {
            method,
            host,
            ..Self::default()
        }
    }
    pub fn ping(&mut self) -> Result<()> {
        use PingMethod::*;
        let frame = match self.method {
            Http => {
                todo!()
            }
            Tcp => self.tcp_ping()?,
        };
        println!(
            "Ping tcp::{} - Connected - time={}ms",
            self.host, frame.elapsed
        );
        self.queue.push(frame);
        Ok(())
    }
    /// Send tcp ping with TcpStream connection,
    /// calculate time with host accepted connection.
    fn tcp_ping(&self) -> Result<TcpFrame> {
        let mut frame = TcpFrame {
            start: Instant::now(),
            elapsed: 0.0,
        };

        let host: Vec<_> = self.host.to_socket_addrs()?.collect();
        let stream = TcpStream::connect_timeout(&host[0], Duration::from_millis(500))?;

        stream.shutdown(std::net::Shutdown::Both)?;

        frame.elapsed = (frame.start.elapsed().as_nanos() as f32) / (1_000_000 as f32);

        Ok(frame)
    }
}
