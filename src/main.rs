use anyhow::{bail, Context, Result};
use bytes::BytesMut;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};

fn main() -> Result<()> {
  println!("Logs from your program will appear here!");

  let listener = TcpListener::bind("127.0.0.1:6379").context("failed to bind to address")?;

  for stream in listener.incoming() {
    match stream {
      Ok(mut connection) => {
        println!("accepted new connection");
        handle_connection(&mut connection).context("failed to handle connection")?;
      }
      Err(e) => {
        println!("error: {}", e);
      }
    }
  }

  Ok(())
}

fn handle_connection(connection: &mut TcpStream) -> Result<()> {
  let mut recv_buf = BytesMut::with_capacity(1024);
  loop {
    match connection.read(&mut recv_buf) {
      Ok(0) => {
        println!("connection closed");
        return Ok(());
      }
      Ok(n) => {
        println!("read {} bytes", n);

        let response = "+PONG\r\n";
        let send_buf = response.as_bytes();

        connection
          .write_all(send_buf)
          .context("failed to write to stream")?;
        connection.flush().context("failed to flush stream")?;
      }
      Err(e) => {
        bail!("failed to read from stream: {}", e);
      }
    }
  }
}
