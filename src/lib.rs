pub mod error;

use std::time::Duration;

use anyhow::Result;

use clap::ValueEnum;
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::TcpStream,
    time::{timeout, Instant},
};

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

#[derive(Debug, Default, ValueEnum, Clone)]
pub enum HttpMethod {
    #[default]
    GET,
    HEAD,
    POST,
    PUT,
    DELETE,
    CONNECT,
    OPTIONS,
    TRACE,
    PATCH,
}
impl HttpMethod {
    pub fn as_str(&self) -> &'static str {
        use HttpMethod::*;
        match self {
            GET => "GET",
            HEAD => "HEAD",
            POST => "POST",
            PUT => "PUT",
            DELETE => "DELETE",
            CONNECT => "CONNECT",
            OPTIONS => "OPTIONS",
            TRACE => "TRACE",
            PATCH => "PATCH",
        }
    }
}

#[derive(Debug)]
pub struct TcpFrame {
    /// request start time
    start: Instant,
    /// elapsed time millis
    pub elapsed: f32,
    /// The package is sent successful
    pub success: bool,
}
impl Default for TcpFrame {
    fn default() -> Self {
        Self {
            start: Instant::now(),
            elapsed: 0.0,
            success: false,
        }
    }
}
impl TcpFrame {
    pub fn calculate_delay(&mut self) {
        self.elapsed = calculate_delay_millis(self.start)
    }
}
impl PartialEq for TcpFrame {
    fn eq(&self, other: &Self) -> bool {
        self.elapsed == other.elapsed
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
    (nanos as f32) / (1_000_000_f32)
}

#[derive(Debug, Default)]
pub struct PingResult {
    pub minimum: f32,
    pub maximum: f32,
    pub average: f32,
    pub success: usize,
}
#[derive(Debug)]
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
    /// Ignore count, send packages forever
    pub timeout: bool,
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
            bytes: 56,
            http_method: HttpMethod::GET,
            wait: false,
            timeout: false,
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
    pub async fn ping(&mut self) -> Result<()> {
        use PingMethod::*;
        match self.method {
            Http => self.http_ping().await?,
            Tcp => self.tcp_ping().await?,
        };
        Ok(())
    }
    pub fn end(&mut self) -> Result<()> {
        self.elapsed = calculate_delay_millis(self.start);
        let default_frame = TcpFrame::default();

        self.result.maximum = self
            .queue
            .iter()
            .filter(|frame| frame.success)
            .max_by(|x, y| x.cmp(y))
            .unwrap_or(&default_frame)
            .elapsed;
        self.result.minimum = self
            .queue
            .iter()
            .filter(|frame| frame.success)
            .min()
            .unwrap_or(&default_frame)
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

    /// Build a tcp stream
    async fn client(&self) -> Result<TcpStream> {
        let stream = TcpStream::connect(&self.host);
        let stream = timeout(Duration::from_millis(500), stream).await??;

        Ok(stream)
    }

    /// Send tcp ping with TcpStream connection,
    /// calculate time with host accepted connection.
    async fn tcp_ping(&mut self) -> Result<()> {
        let mut stream = self.client().await?;
        let frame = TcpFrame {
            ..Default::default()
        };
        self.queue.push(frame);

        let len = self.queue.len();
        let frame = &mut self.queue[len - 1];

        let data = vec![255_u8; self.bytes];
        stream.write_all(&data).await?;
        stream.flush().await?;

        // stream.shutdown(std::net::Shutdown::Both)?;

        frame.calculate_delay();
        frame.success = true;

        println!(
            "Ping tcp::{}({}) - Connected - time={}ms",
            self.host,
            stream.peer_addr()?,
            frame.elapsed
        );
        Ok(())
    }

    /// Send ping package with http protocol
    async fn http_ping(&mut self) -> Result<()> {
        let mut stream = self.client().await?;
        let frame = TcpFrame {
            ..Default::default()
        };
        self.queue.push(frame);

        let len = self.queue.len();
        let frame = &mut self.queue[len - 1];

        let body = vec![255_u8; self.bytes];

        let first_line = format!("{} / HTTP/1.1\r\n", self.http_method.as_str());
        let headers = format!(
            "Host: {}\r\nUser-Agent: Pluto/{}\r\nContent-Type: {}\r\nContent-Length: {}\r\n\r\n",
            self.host,
            env!("CARGO_PKG_VERSION"),
            "text/plain",
            body.len(),
        );
        stream
            .write_all(
                format!("{first_line}{headers}{}", String::from_utf8_lossy(&body)).as_bytes(),
            )
            .await?;
        stream.flush().await?;

        if self.wait {
            read_response(&mut stream).await?;
        } else {
            // stream.shutdown(std::net::Shutdown::Both)?;
        }

        frame.calculate_delay();
        frame.success = true;

        println!(
            "Ping http://{}({}) - Connected - time={}ms",
            self.host,
            stream.peer_addr()?,
            frame.elapsed,
        );

        Ok(())
    }
}

async fn read_response(mut stream: &mut TcpStream) -> Result<()> {
    let mut reader = BufReader::new(&mut stream);
    let mut buf = String::new();
    loop {
        let bytes = reader.read_line(&mut buf).await?;
        if bytes < 3 {
            break;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
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
            elapsed: 0.12,
            success: true,
            ..Default::default()
        };
        let samller = TcpFrame {
            elapsed: 0.1,
            success: true,
            ..Default::default()
        };
        assert!(lager > samller);
    }
}
