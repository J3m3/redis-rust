mod interpreter;
mod parser;
mod response;
mod tokenizer;

pub use response::generate_response;

use anyhow::{bail, Context, Result};

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum Command {
  Ping { message: Option<String> },
  Echo { message: String },
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum Token {
  Array(usize),
  BulkString(usize),
  NullBulkString,
  StringValue(String),
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum RespValue {
  Array(Vec<RespValue>),
  BulkString(String),
  NullBulkString,
}

const NULL_BULK_STRING: &str = "$-1";
