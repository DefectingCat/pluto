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
        self.elapsed = calculate_delay_millis(self.start)
    }
}

fn calculate_delay_millis(start: Instant) -> f32 {
    let nanos = start.elapsed().as_nanos();
    (nanos as f32) / (1_000_000 as f32)
}

pub struct Pluto {
    /// Calculate total time
    pub start: Instant,
    pub method: PingMethod,
    pub port: u32,
    pub queue: Vec<TcpFrame>,
    pub host: String,
    // elapsed time millis
    pub elapsed: f32,
}
impl Default for Pluto {
    fn default() -> Self {
        Self {
            start: Instant::now(),
            method: PingMethod::default(),
            port: 80,
            queue: vec![],
            host: String::new(),
            elapsed: 0.0,
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
    pub fn end(&mut self) {
        self.elapsed = calculate_delay_millis(self.start);
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

        frame.calculate_delay();

        Ok(frame)
    }
}
