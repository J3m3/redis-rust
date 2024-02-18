mod echo;
mod get;
mod ping;
mod set;

use std::sync::{Arc, Mutex};

pub use echo::*;
pub use get::*;
pub use ping::*;
pub use set::*;

use crate::database::Database;
use crate::resp_server::Result;

pub trait Execute {
  fn execute(&self, ctx: &ExecutionContext) -> Result<Vec<u8>>;
}

pub struct ExecutionContext<'a> {
  pub db: &'a Arc<Mutex<Database>>,
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum Command {
  Ping(Ping),
  Echo(Echo),
  Set(Set),
  Get(Get),
}

impl Execute for Command {
  fn execute(&self, ctx: &ExecutionContext) -> Result<Vec<u8>> {
    match self {
      Command::Ping(ping) => ping.execute(ctx),
      Command::Echo(echo) => echo.execute(ctx),
      Command::Set(set) => set.execute(ctx),
      Command::Get(get) => get.execute(ctx),
    }
  }
}
