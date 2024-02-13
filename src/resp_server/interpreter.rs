use super::{bail, Result};
use super::{Command, RespValue};

pub fn interpret(ir: &RespValue) -> Result<Command> {
  let RespValue::Array(cmd) = ir else {
    bail!("client command should always generate array")
  };

  let mut cmd_iter = cmd.into_iter();

  let Some(RespValue::BulkString(string_value)) = cmd_iter.next() else {
    bail!("client command array should always contain BulkString");
  };

  match string_value.to_uppercase().as_str() {
    "ECHO" => {
      let Some(message) = cmd_iter.next() else {
        bail!("ECHO should contain message to echo");
      };
      let RespValue::BulkString(message) = message else {
        bail!("ECHO should contain message as BulkString");
      };
      Ok(Command::Echo {
        message: message.clone(),
      })
    }
    "PING" => {
      if let Some(message) = cmd_iter.next() {
        let RespValue::BulkString(message) = message else {
          bail!("PING should contain message as BulkString if exists");
        };
        Ok(Command::Ping {
          message: Some(message.clone()),
        })
      } else {
        Ok(Command::Ping { message: None })
      }
    }
    "SET" => {
      let Some(key) = cmd_iter.next() else {
        bail!("SET should contain a key, but nothing is given");
      };
      let Some(value) = cmd_iter.next() else {
        bail!("SET should contain a key, but nothing is given");
      };
      let RespValue::BulkString(key) = key else {
        bail!("SET expects its key is BulkString");
      };
      let RespValue::BulkString(value) = value else {
        bail!("SET expects its key is BulkString");
      };

      Ok(Command::Set {
        key: key.clone(),
        value: value.clone(),
      })
    }
    "GET" => {
      let Some(key) = cmd_iter.next() else {
        bail!("GET should contain a key, but nothing is given");
      };
      let RespValue::BulkString(key) = key else {
        bail!("GET expects its key is BulkString");
      };
      Ok(Command::Get { key: key.clone() })
    }
    _ => {
      bail!("unexpected command, or not yet implemented")
    }
  }
}

#[cfg(test)]
mod tests_command_generation {
  use crate::resp_server::*;
  use interpreter::*;
  use parser::*;
  use tokenizer::*;

  mod test_echo {
    use super::*;
    #[test]
    fn should_work_with_non_empty_message() {
      let client_query = "*2\r\n$4\r\nECHO\r\n$3\r\nhey\r\n".to_owned();
      let tokens = tokenize(&client_query).unwrap();
      let intermediate_representation = parse(&tokens).unwrap();
      let command = interpret(&intermediate_representation).unwrap();

      let expected_tokens = vec![
        Token::Array(2),
        Token::BulkString(4),
        Token::StringValue("ECHO".to_owned()),
        Token::BulkString(3),
        Token::StringValue("hey".to_owned()),
      ];
      let expected_intermediate_representation = RespValue::Array(vec![
        RespValue::BulkString("ECHO".to_owned()),
        RespValue::BulkString("hey".to_owned()),
      ]);
      let expected_command = Command::Echo {
        message: "hey".to_owned(),
      };

      assert!(
        tokens == expected_tokens,
        "tokenizer: ECHO with non-empty message"
      );
      assert!(
        intermediate_representation == expected_intermediate_representation,
        "parser: ECHO with non-empty message"
      );
      assert!(
        command == expected_command,
        "interpreter: ECHO with non-empty message"
      );
    }

    #[test]
    fn should_work_with_empty_message() {
      let client_query = "*2\r\n$4\r\nECHO\r\n$0\r\n\r\n".to_owned();
      let tokens = tokenize(&client_query).unwrap();
      let intermediate_representation = parse(&tokens).unwrap();
      let command = interpret(&intermediate_representation).unwrap();

      let expected_tokens = vec![
        Token::Array(2),
        Token::BulkString(4),
        Token::StringValue("ECHO".to_owned()),
        Token::BulkString(0),
        Token::StringValue("".to_owned()),
      ];
      let expected_intermediate_representation = RespValue::Array(vec![
        RespValue::BulkString("ECHO".to_owned()),
        RespValue::BulkString("".to_owned()),
      ]);
      let expected_command = Command::Echo {
        message: "".to_owned(),
      };

      assert!(
        tokens == expected_tokens,
        "tokenizer: ECHO with non-empty message"
      );
      assert!(
        intermediate_representation == expected_intermediate_representation,
        "parser: ECHO with non-empty message"
      );
      assert!(
        command == expected_command,
        "interpreter: ECHO with non-empty message"
      );
    }
  }

  mod test_ping {
    use super::*;
    #[test]
    fn should_work_with_optional_message() {
      let client_query = "*2\r\n$4\r\nPING\r\n$3\r\nhey\r\n".to_owned();
      let tokens = tokenize(&client_query).unwrap();
      let intermediate_representation = parse(&tokens).unwrap();
      let command = interpret(&intermediate_representation).unwrap();

      let expected_tokens = vec![
        Token::Array(2),
        Token::BulkString(4),
        Token::StringValue("PING".to_owned()),
        Token::BulkString(3),
        Token::StringValue("hey".to_owned()),
      ];
      let expected_intermediate_representation = RespValue::Array(vec![
        RespValue::BulkString("PING".to_owned()),
        RespValue::BulkString("hey".to_owned()),
      ]);
      let expected_command = Command::Ping {
        message: Some("hey".to_owned()),
      };

      assert!(tokens == expected_tokens, "tokenizer: PING with message");
      assert!(
        intermediate_representation == expected_intermediate_representation,
        "parser: PING with message"
      );
      assert!(
        command == expected_command,
        "interperter: PING with message"
      );
    }
    #[test]
    fn should_work_without_optional_message() {
      let client_query = "*1\r\n$4\r\nPING\r\n".to_owned();
      let tokens = tokenize(&client_query).unwrap();
      let intermediate_representation = parse(&tokens).unwrap();
      let command = interpret(&intermediate_representation).unwrap();

      let expected_tokens = vec![
        Token::Array(1),
        Token::BulkString(4),
        Token::StringValue("PING".to_owned()),
      ];
      let expected_intermediate_representation =
        RespValue::Array(vec![RespValue::BulkString("PING".to_owned())]);
      let expected_command = Command::Ping { message: None };

      assert!(
        tokens == expected_tokens,
        "tokenizer: PING without optional message"
      );
      assert!(
        intermediate_representation == expected_intermediate_representation,
        "parser: PING without optional message"
      );
      assert!(
        command == expected_command,
        "interpreter: PING without optional message"
      );
    }
  }
}
