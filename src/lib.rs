pub mod error;

use std::time::Duration;

use anyhow::Result;

use clap::ValueEnum;
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::TcpStream,
    time::{timeout, Instant},
};

use crate::error::PlutoError;

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

#[derive(Debug, Default, ValueEnum, Clone, Copy)]
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
    id: usize,
    /// request start time
    start: Instant,
    /// elapsed time millis
    pub elapsed: f32,
    /// The ping package is loss or not
    pub success: bool,
    /// The frame is sent successful
    send_success: bool,
}
impl Default for TcpFrame {
    fn default() -> Self {
        Self {
            id: 0,
            start: Instant::now(),
            elapsed: 0.0,
            success: false,
            send_success: false,
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
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
    fn lt(&self, other: &Self) -> bool {
        self.elapsed < other.elapsed
    }
    fn le(&self, other: &Self) -> bool {
        self.elapsed <= other.elapsed
    }
    fn gt(&self, other: &Self) -> bool {
        self.elapsed > other.elapsed
    }
    fn ge(&self, other: &Self) -> bool {
        self.elapsed >= other.elapsed
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
    pub total: usize,
    pub average: f32,
    pub loss: usize,
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
    /// Timeout for waiting each package time
    pub wait_timeout: u64,
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
            wait_timeout: 300,
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

        let queue: Vec<_> = self.queue.iter().filter(|q| q.send_success).collect();
        let total_len = queue.len();
        let sucess_queue = queue.iter().filter(|f| f.send_success).collect::<Vec<_>>();
        self.result.maximum = sucess_queue
            .iter()
            .max_by(|x, y| x.cmp(y))
            .map(|f| **f)
            .unwrap_or(&default_frame)
            .elapsed;
        self.result.minimum = sucess_queue
            .iter()
            .min()
            .map(|f| **f)
            .unwrap_or(&default_frame)
            .elapsed;
        let total = sucess_queue
            .iter()
            .fold(0.0, |prev, frame| prev + frame.elapsed);
        self.result.total = total_len;

        let average = total / total_len as f32;
        self.result.average = if f32::is_nan(average) { 0.0 } else { average };
        self.result.success = queue
            .iter()
            .filter(|frame| frame.success)
            .collect::<Vec<_>>()
            .len();
        self.result.loss = total_len - self.result.success;
        Ok(())
    }

    /// Build a tcp stream
    async fn client(&self) -> Result<TcpStream> {
        let stream = TcpStream::connect(&self.host);
        let stream = timeout(Duration::from_millis(self.wait_timeout), stream).await??;
        Ok(stream)
    }

    /// Send tcp ping with TcpStream connection,
    /// calculate time with host accepted connection.
    async fn tcp_ping(&mut self) -> Result<()> {
        let mut stream = self.client().await?;
        self.queue.push(TcpFrame::default());
        let len = self.queue.len();
        let frame = self
            .queue
            .last_mut()
            .ok_or(PlutoError::CommonError(format!(
                "access frame {} failed",
                len
            )))?;
        frame.id = len;

        let data = vec![255_u8; self.bytes];
        stream.write_all(&data).await?;
        stream.flush().await?;
        frame.send_success = true;

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
        self.queue.push(TcpFrame::default());
        let len = self.queue.len();
        let frame = self
            .queue
            .last_mut()
            .ok_or(PlutoError::CommonError(format!(
                "access frame {} failed",
                len
            )))?;
        frame.id = len;
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
            frame.send_success = true;
        } else {
            frame.send_success = true;
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
