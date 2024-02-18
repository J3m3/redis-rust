use std::time::{Duration, SystemTime};

use crate::database::{Data, ExpireTime};
use crate::resp_server::{bail, Context, Result};
use crate::resp_server::{RespValue, NULL_BULK_STRING};

use super::{Execute, ExecutionContext};

#[derive(PartialEq, Eq, Debug, Clone, Default)]
pub struct Set {
  pub key: String,
  pub value: String,
  pub nx_or_xx: Option<String>,
  pub get: bool,
  pub expire_time: Option<ExpireTime>,
}

impl Execute for Set {
  fn execute(&self, ctx: &ExecutionContext) -> Result<Vec<u8>> {
    let Set {
      key,
      value,
      nx_or_xx,
      get,
      expire_time,
    } = self;
    let ExecutionContext { db } = ctx;

    let data = Data {
      value: value.to_owned(),
      expire_time: match expire_time {
        Some(t) => Some(*t),
        None => None,
      },
    };

    let mut db = db.lock().unwrap();

    let result = if *get {
      let old_data = db.get(key).cloned();
      match nx_or_xx {
        Some(opt) if opt == "NX" && old_data.is_none() => db.set(key, &data),
        Some(opt) if opt == "XX" && old_data.is_some() => db.set(key, &data),
        None => db.set(key, &data),
        Some(_) => bail!("unknown SET option: this is unreachable"),
      };
      match old_data {
        Some(data) => format!("+{}", data.value).into_bytes(),
        None => format!("{}\r\n", NULL_BULK_STRING).into_bytes(),
      }
    } else {
      match nx_or_xx {
        Some(opt) if opt == "NX" && db.get(key).is_none() => db.set(key, &data),
        Some(opt) if opt == "XX" && db.get(key).is_some() => db.set(key, &data),
        None => db.set(key, &data),
        Some(_) => bail!("unknown SET option: this is unreachable"),
      };
      format!("+OK\r\n").into_bytes()
    };

    Ok(result)
  }
}

impl Set {
  pub fn set_option(&mut self, cmd_iter: &mut std::slice::Iter<RespValue>) -> Result<()> {
    while let Some(option) = cmd_iter.next() {
      let RespValue::BulkString(option_str) = option else {
        bail!("SET options should be BulkString");
      };
      let option_str = option_str.to_uppercase();

      match option_str.as_str() {
        "NX" => {
          self.nx_or_xx = Some(option_str);
        }
        "XX" => {
          self.nx_or_xx = Some(option_str);
        }
        "GET" => {
          self.get = true;
        }
        "EX" | "PX" | "EXAT" | "PXAT" => {
          let Some(expire_time_resp) = cmd_iter.next() else {
            bail!("SET {} option should contain expire time", option_str);
          };
          let RespValue::BulkString(expire_time_str) = expire_time_resp else {
            bail!(
              "SET {} option should contain expire time as BulkString",
              option_str
            );
          };

          let Ok(expire_time) = expire_time_str.parse() else {
            bail!("SET EX value should be in valid format");
          };
          let expire_time = calculate_expire_time(&option_str, expire_time)
            .context("failed to calculate expire time")?;

          self.expire_time = Some(expire_time);
        }
        _ => bail!("Unknown option in SET command"),
      }
    }
    Ok(())
  }
}

fn calculate_expire_time(command_type: &str, duration: u64) -> Result<ExpireTime> {
  let duration = match command_type {
    "EX" => Duration::from_secs(duration),
    "PX" => Duration::from_millis(duration),
    "EXAT" => {
      return Ok(Duration::from_secs(duration).as_millis());
    }
    "PXAT" => {
      return Ok(Duration::from_millis(duration).as_millis());
    }
    _ => bail!("unknown expire time option"),
  };
  let now = SystemTime::now();

  let expire_time = now
    .checked_add(duration)
    .context("failed to add duration to systemtime")?;

  // duration since UNIX EPOCH(1970-01-01 00:00:00 UTC)
  let expire_time = expire_time
    .duration_since(SystemTime::UNIX_EPOCH)
    .context("SystemTime before unix epoch")?
    .as_millis();

  Ok(expire_time)
}
