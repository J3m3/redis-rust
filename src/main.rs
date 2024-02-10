use anyhow::{Context, Result};
use bytes::{BufMut, BytesMut};
use std::{io::Write, net::TcpListener};

fn main() -> Result<()> {
  println!("Logs from your program will appear here!");

  let listener = TcpListener::bind("127.0.0.1:6379").context("failed to bind to address")?;

  for _stream in listener.incoming() {
    match _stream {
      Ok(mut stream) => {
        println!("accepted new connection");

        let mut buf = BytesMut::with_capacity(1024);

        let response = "+PONG\r\n";
        buf.put(response.as_bytes());

        stream
          .write_all(&buf)
          .context("failed to write to stream")?;
      }
      Err(e) => {
        println!("error: {}", e);
      }
    }
  }

  Ok(())
}
