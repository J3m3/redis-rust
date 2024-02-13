use super::Token;
use super::{bail, Context, Result};

pub fn tokenize(input: &str) -> Result<Vec<Token>> {
  let mut tokens = Vec::new();
  let mut chars = input.chars();

  while let Some(c) = chars.next() {
    match c {
      '*' => {
        let length = read_until_crlf(&mut chars)
          .parse()
          .context("unexpected token: usize value expected while lexing array")?;
        tokens.push(Token::Array(length));
      }
      '$' => {
        let length: isize = read_until_crlf(&mut chars)
          .parse()
          .context("unexpected token: usize value expected while lexing bulk string")?;
        if length < 0 {
          tokens.push(Token::NullBulkString);
        } else {
          tokens.push(Token::BulkString(length as usize));

          let string = read_until_crlf(&mut chars);
          tokens.push(Token::StringValue(string));
        }
      }
      _ => {
        bail!("unexpected token: the given token not exists in grammar (or not yet implemented to be parsed)")
      }
    }
  }

  Ok(tokens)
}

fn read_until_crlf(chars: &mut std::str::Chars) -> String {
  let mut result = "".to_owned();
  while let Some(c) = chars.next() {
    if c == '\r' {
      if chars.next() == Some('\n') {
        break;
      }
    } else {
      result.push(c);
    }
  }
  result
}
