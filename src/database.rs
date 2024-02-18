use anyhow::{Context, Result};
use std::collections::HashMap;

pub type ExpireTime = u128;

trait Expire {
  fn expire_time(&self) -> Option<ExpireTime>;
}

#[derive(Debug, Clone)]
pub struct Data {
  pub value: String,
  pub expire_time: Option<ExpireTime>,
}

impl Expire for Data {
  fn expire_time(&self) -> Option<ExpireTime> {
    self.expire_time
  }
}

impl Expire for &Data {
  fn expire_time(&self) -> Option<ExpireTime> {
    self.expire_time
  }
}

#[derive(Debug)]
pub struct Database {
  db: HashMap<String, Data>,
}

impl Database {
  pub fn new() -> Self {
    Database { db: HashMap::new() }
  }

  pub fn get(&self, key: &str) -> Option<&Data> {
    self.db.get(key).and_then(|data| self.handle_expiry(data))
  }

  pub fn set(&mut self, key: &str, val: &Data) -> Option<Data> {
    self
      .db
      .insert(key.to_owned(), val.clone())
      .and_then(|data| self.handle_expiry(data))
  }

  fn handle_expiry<T: Expire>(&self, data: T) -> Option<T> {
    match data.expire_time() {
      None => Some(data),
      Some(expire_time) => match is_expired(expire_time) {
        Ok(expired) if !expired => Some(data),
        Ok(_) => None,
        Err(e) => {
          eprintln!("{}", e);
          None
        }
      },
    }
  }
}

fn is_expired(expire_time: u128) -> Result<bool> {
  use std::time::SystemTime;
  // duration since UNIX EPOCH(1970-01-01 00:00:00 UTC)
  let now = SystemTime::now()
    .duration_since(SystemTime::UNIX_EPOCH)
    .context("SystemTime before unix epoch")?
    .as_millis();
  Ok(now > expire_time)
}
