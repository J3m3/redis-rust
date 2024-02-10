use anyhow::{Context, Result};
use bytes::{BufMut, BytesMut};
use std::io::Write;
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
  let mut buf = BytesMut::with_capacity(1024);

  let response = "+PONG\r\n";
  buf.put(response.as_bytes());

  connection
    .write_all(&buf)
    .context("failed to write to stream")?;

  Ok(())
}
