pub mod database;
pub mod resp_server;

use std::sync::{Arc, Mutex};

use anyhow::{bail, Context, Result};
use bytes::{BufMut, BytesMut};
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
          generate_response(request, db).context("failed to generate response from request");

        match response {
          Ok(response) => {
            connection
              .write_all(&response)
              .await
              .context("failed to write response message to stream")?;
          }
          Err(err) => {
            let mut err_buf = BytesMut::with_capacity(4096);

            err_buf.put(format!("-ERR {}\n", err).as_bytes());
            for cause in err.chain().skip(1) {
              err_buf.put(format!("\tCaused by: {}\n", cause).as_bytes());
            }
            err_buf.put("\r\n".as_bytes());

            connection
              .write_all(&err_buf)
              .await
              .context("failed to write error message to stream")?;
          }
        }
        connection.flush().await.context("failed to flush stream")?;
      }
      Err(e) => {
        bail!("failed to read from stream: {}", e);
      }
    }
  }
}
