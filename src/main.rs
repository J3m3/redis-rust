pub mod database;
pub mod resp_server;

use std::sync::{Arc, Mutex};

use anyhow::{bail, Context, Result};
use bytes::BytesMut;
use database::DataBase;
use resp_server::generate_response;
use tokio::{
  io::AsyncReadExt,
  io::AsyncWriteExt,
  net::{TcpListener, TcpStream},
};

#[tokio::main]
async fn main() -> Result<()> {
  let db = Arc::new(Mutex::new(DataBase::new()));

  let listener = TcpListener::bind("127.0.0.1:6379")
    .await
    .context("failed to bind to address")?;

  loop {
    match listener.accept().await {
      Ok((connection, addr)) => {
        println!("accepted new connection from {}", addr);

        let db = Arc::clone(&db);
        tokio::spawn(async move {
          if let Err(e) = handle_connection(connection, &db)
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

async fn handle_connection(mut connection: TcpStream, db: &Arc<Mutex<DataBase>>) -> Result<()> {
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
