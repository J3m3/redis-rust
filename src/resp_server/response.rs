use super::{interpreter, parser, tokenizer, Command};
use super::{Context, Result};

pub fn generate_response(request: &[u8]) -> Result<Vec<u8>> {
  let str = std::str::from_utf8(request)
    .context("failed to convert raw binary request to utf-8 string slice")?;

  let tokens = tokenizer::tokenize(str).context("tokenization failed")?;
  let intermediate_representation = parser::parse(&tokens).context("parsing failed")?;
  let command =
    interpreter::interpret(&intermediate_representation).context("interpretation failed")?;

  let response = match command {
    Command::Echo { message } => format!("+{}\r\n", message).into_bytes(),
    Command::Ping { message } => {
      if let Some(m) = message {
        format!("+{}\r\n", m).into_bytes()
      } else {
        "+PONG\r\n".to_owned().into_bytes()
      }
    }
  };

  Ok(response)
}

#[cfg(test)]
mod tests_response_generation {
  use super::*;
  #[test]
  fn should_work_with_case_insensitivity() {
    let client_query = "*2\r\n$4\r\nECHO\r\n$3\r\nhey\r\n".to_owned();
    let response = generate_response(client_query.as_bytes()).unwrap();
    let expected_response = b"+hey\r\n".to_vec();
    assert_eq!(response, expected_response);

    let client_query = "*2\r\n$4\r\necho\r\n$3\r\nhey\r\n".to_owned();
    let response = generate_response(client_query.as_bytes()).unwrap();
    let expected_response = b"+hey\r\n".to_vec();
    assert_eq!(response, expected_response);
  }
}
