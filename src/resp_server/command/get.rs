use crate::resp_server::NULL_BULK_STRING;

use super::{Execute, ExecutionContext};
use crate::resp_server::Result;

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Get {
  pub key: String,
}

impl Execute for Get {
  fn execute(&self, ctx: &ExecutionContext) -> Result<Vec<u8>> {
    let Get { key } = self;
    let ExecutionContext { db } = ctx;
    Ok(match db.lock().unwrap().get(key) {
      Some(data) => format!("+{}\r\n", data.value).into_bytes(),
      None => format!("{}\r\n", NULL_BULK_STRING).into_bytes(),
    })
  }
}
