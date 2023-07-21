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

#[derive(Debug, PartialEq)]
pub struct TcpFrame {
    // request start time
    start: Instant,
    // elapsed time millis
    pub elapsed: f32,
    // The package is sent successful
    pub success: bool,
}
impl TcpFrame {
    pub fn calculate_delay(&mut self) {
        self.elapsed = calculate_delay_millis(self.start)
    }
}
impl Eq for TcpFrame {}
impl PartialOrd for TcpFrame {
    fn ge(&self, other: &Self) -> bool {
        self.elapsed >= other.elapsed
    }
    fn gt(&self, other: &Self) -> bool {
        self.elapsed > other.elapsed
    }
    fn le(&self, other: &Self) -> bool {
        self.elapsed <= other.elapsed
    }
    fn lt(&self, other: &Self) -> bool {
        self.elapsed < other.elapsed
    }
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        use std::cmp::Ordering::*;
        match self.elapsed - other.elapsed {
            x if x < 0.0 => Some(Less),
            x if x > 0.0 => Some(Greater),
            x if x == 0.0 => Some(Equal),
            _ => Some(Equal),
        }
    }
}
impl Ord for TcpFrame {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        use std::cmp::Ordering::*;
        match self.elapsed - other.elapsed {
            x if x < 0.0 => Less,
            x if x > 0.0 => Greater,
            x if x == 0.0 => Equal,
            _ => Equal,
        }
    }
}

/// Calculate milliseconds until now.
fn calculate_delay_millis(start: Instant) -> f32 {
    let nanos = start.elapsed().as_nanos();
    (nanos as f32) / (1_000_000 as f32)
}

#[derive(Debug, Default)]
pub struct PingResult {
    pub minimum: f32,
    pub maximum: f32,
    pub average: f32,
    pub success: usize,
}
pub struct Pluto {
    /// Calculate total time
    start: Instant,
    pub method: PingMethod,
    pub port: u32,
    pub queue: Vec<TcpFrame>,
    pub host: String,
    // elapsed time millis
    pub elapsed: f32,
    pub result: PingResult,
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
            result: PingResult::default(),
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
        match self.method {
            Http => {
                todo!()
            }
            Tcp => self.tcp_ping()?,
        };
        Ok(())
    }
    pub fn end(&mut self) -> Result<()> {
        self.elapsed = calculate_delay_millis(self.start);

        self.result.maximum = self
            .queue
            .iter()
            .filter(|frame| frame.success)
            .max_by(|x, y| x.cmp(y))
            .unwrap_or(&TcpFrame {
                start: Instant::now(),
                elapsed: 0.0,
                success: false,
            })
            .elapsed;
        self.result.minimum = self
            .queue
            .iter()
            .filter(|frame| frame.success)
            .min()
            .unwrap_or(&TcpFrame {
                start: Instant::now(),
                elapsed: 0.0,
                success: false,
            })
            .elapsed;
        let total = self
            .queue
            .iter()
            .filter(|frame| frame.success)
            .fold(0.0, |prev, frame| prev + frame.elapsed);
        self.result.average = total / self.queue.len() as f32;
        self.result.success = self
            .queue
            .iter()
            .filter(|frame| frame.success)
            .collect::<Vec<_>>()
            .len();
        Ok(())
    }

    /// Send tcp ping with TcpStream connection,
    /// calculate time with host accepted connection.
    fn tcp_ping(&mut self) -> Result<()> {
        self.queue.push(TcpFrame {
            start: Instant::now(),
            elapsed: 0.0,
            success: false,
        });
        let len = self.queue.len();
        let frame = &mut self.queue[len - 1];

        let host: Vec<_> = self.host.to_socket_addrs()?.collect();
        let stream = TcpStream::connect_timeout(&host[0], Duration::from_millis(500))?;

        stream.shutdown(std::net::Shutdown::Both)?;

        frame.calculate_delay();
        frame.success = true;

        println!(
            "Ping tcp::{} - Connected - time={}ms",
            self.host, frame.elapsed
        );

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::time::Instant;

    use crate::{PingMethod, TcpFrame};

    #[test]
    fn serialize_method() {
        let tcp = PingMethod::from("tcp");
        assert_eq!(tcp, PingMethod::Tcp);
        let http = PingMethod::from("http");
        assert_eq!(http, PingMethod::Http);
        let other = PingMethod::from("rua");
        assert_eq!(other, PingMethod::Http);
    }

    #[test]
    fn cmp_frame() {
        let lager = TcpFrame {
            start: Instant::now(),
            elapsed: 0.12,
            success: true,
        };
        let samller = TcpFrame {
            start: Instant::now(),
            elapsed: 0.1,
            success: true,
        };
        assert!(lager > samller);
    }
}
