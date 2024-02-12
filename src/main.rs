pub mod resp_server;

use anyhow::{bail, Context, Result};
use bytes::BytesMut;
use resp_server::generate_response;
use tokio::{
  io::AsyncReadExt,
  io::AsyncWriteExt,
  net::{TcpListener, TcpStream},
};

#[tokio::main]
async fn main() -> Result<()> {
  println!("Logs from your program will appear here!");

  let listener = TcpListener::bind("127.0.0.1:6379")
    .await
    .context("failed to bind to address")?;

  loop {
    match listener.accept().await {
      Ok((connection, addr)) => {
        println!("accepted new connection from {}", addr);
        tokio::spawn(async move {
          if let Err(e) = handle_connection(connection)
            .await
            .context("failed to handle connection")
          {
            eprintln!("{}", e);
          }
        });
      }
      Err(e) => {
        eprintln!("failed to accept connection: {}", e);
      }
    }
  }
}

async fn handle_connection(mut connection: TcpStream) -> Result<()> {
  let mut recv_buf = BytesMut::zeroed(1024);
  loop {
    match connection.read(&mut recv_buf).await {
      Ok(0) => {
        println!("connection closed");
        return Ok(());
      }
      Ok(n) => {
        println!("read {} bytes", n);

        let request = &recv_buf[..n];
        let response =
          generate_response(request).context("failed to generate response from request")?;

        connection
          .write_all(&response)
          .await
          .context("failed to write to stream")?;
        connection.flush().await.context("failed to flush stream")?;
      }
      Err(e) => {
        bail!("failed to read from stream: {}", e);
      }
    }
  }
}
