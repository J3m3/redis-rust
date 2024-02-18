use super::Result;
use super::{Execute, ExecutionContext};

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Ping {
  pub message: Option<String>,
}

impl Execute for Ping {
  fn execute(&self, _ctx: &ExecutionContext) -> Result<Vec<u8>> {
    Ok(match &self.message {
      Some(message) => format!("+{}\r\n", message).into_bytes(),
      None => "+PONG\r\n".to_owned().into_bytes(),
    })
  }
}
