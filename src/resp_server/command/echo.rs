use super::Result;
use super::{Execute, ExecutionContext};

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Echo {
  pub message: String,
}

impl Execute for Echo {
  fn execute(&self, _ctx: &ExecutionContext) -> Result<Vec<u8>> {
    Ok(format!("+{}\r\n", self.message).into_bytes())
  }
}
