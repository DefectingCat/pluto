use anyhow::{Ok, Result};
use tokio::net::{TcpListener, TcpStream};

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    let listener = TcpListener::bind("127.0.0.1:4000").await?;

    loop {
        let (stream, _) = listener.accept().await?;
        tokio::spawn(async move {
            handler(stream).await?;
            Ok::<()>(())
        });
    }
}

async fn handler(stream: TcpStream) -> Result<()> {
    Ok(())
}
