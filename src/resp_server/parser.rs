use super::{bail, Context, Result};
use super::{RespValue, Token};

pub fn parse(tokens: &[Token]) -> Result<RespValue> {
  let mut iter = tokens.iter().peekable();

  fn _parse(iter: &mut std::iter::Peekable<std::slice::Iter<Token>>) -> Result<RespValue> {
    match iter.next() {
      Some(Token::Array(length)) => {
        let mut cmd = vec![];
        for _ in 0..*length {
          let element = _parse(iter).context("failed to parse array element")?;
          cmd.push(element);
        }
        if let Some(_) = iter.peek() {
          bail!(
            "Array length({}) exceeds the actual number of elements",
            *length
          )
        } else {
          Ok(RespValue::Array(cmd))
        }
      }
      Some(Token::BulkString(length)) => {
        let string_value_token = iter
          .next()
          .context("BulkString indicator exists, but actual string value is not given")?;
        if let Token::StringValue(string_value) = string_value_token {
          if *length != string_value.len() {
            bail!("BulkString length does not match with actual string value")
          } else {
            Ok(RespValue::BulkString(string_value.clone()))
          }
        } else {
          bail!(
            "expected BulkString string value, but {:?} is given",
            string_value_token
          )
        }
      }
      Some(Token::NullBulkString) => Ok(RespValue::NullBulkString),
      Some(Token::StringValue(_)) => {
        bail!("string value should always be consumed in BulkString match clause")
      }
      None => {
        bail!("expected valid RESP bytes, but nothing given (None)")
      }
    }
  }

  _parse(&mut iter).context("failed to parse")
}
