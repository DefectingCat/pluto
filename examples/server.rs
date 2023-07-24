use std::{env, time::Duration};

use anyhow::{Ok, Result};
use env_logger::{Builder, Env};
use log::info;
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::{TcpListener, TcpStream},
};

#[tokio::main]
async fn main() -> Result<()> {
    let env = Env::default().filter_or("RUST_LOG", "server");
    Builder::from_env(env).init();

    let args: Vec<_> = env::args().collect();
    let delay = args.contains(&"-d".to_owned());

    let listener = TcpListener::bind("0.0.0.0:4000").await?;
    info!("Server running at http://127.0.0.1:4000");

    loop {
        let (stream, _) = listener.accept().await?;
        tokio::spawn(async move {
            handler(stream, delay).await?;
            Ok::<()>(())
        });
    }
}

async fn handler(mut stream: TcpStream, delay: bool) -> Result<()> {
    let mut buf = BufReader::new(&mut stream);

    let mut request = String::new();
    loop {
        let byte = buf.read_line(&mut request).await?;
        if byte < 3 {
            break;
        }
    }
    info!("Got connection {:?}", request);

    if delay {
        info!("Delay with 1s");
        std::thread::sleep(Duration::from_secs(1));
    }

    stream
        .write_all("HTTP/1.1 200 OK\r\nContent-type: text/plain\r\n\r\nHello world".as_bytes())
        .await?;
    Ok(())
}
