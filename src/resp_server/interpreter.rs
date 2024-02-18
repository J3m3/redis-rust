use anyhow::Context;

use super::{bail, Result};
use super::{Command, Echo, Get, Ping, RespValue, Set};

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
      Ok(Command::Echo(Echo {
        message: message.clone(),
      }))
    }
    "PING" => {
      if let Some(message) = cmd_iter.next() {
        let RespValue::BulkString(message) = message else {
          bail!("PING should contain message as BulkString if exists");
        };
        Ok(Command::Ping(Ping {
          message: Some(message.clone()),
        }))
      } else {
        Ok(Command::Ping(Ping { message: None }))
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

      let mut set_cmd = Set {
        key: key.clone(),
        value: value.clone(),
        ..Set::default()
      };
      set_cmd
        .set_option(&mut cmd_iter)
        .context("failed to set option")?;

      Ok(Command::Set(set_cmd))
    }
    "GET" => {
      let Some(key) = cmd_iter.next() else {
        bail!("GET should contain a key, but nothing is given");
      };
      let RespValue::BulkString(key) = key else {
        bail!("GET expects its key is BulkString");
      };
      Ok(Command::Get(Get { key: key.clone() }))
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
      let expected_command = Command::Echo(Echo {
        message: "hey".to_owned(),
      });

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
      let expected_command = Command::Echo(Echo {
        message: "".to_owned(),
      });

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
      let expected_command = Command::Ping(Ping {
        message: Some("hey".to_owned()),
      });

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
      let expected_command = Command::Ping(Ping { message: None });

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

  mod test_set {
    use super::*;

    #[test]
    fn should_work_with_proper_key_value() {
      // SET mykey myvalue
      let client_request = "*3\r\n$3\r\nSET\r\n$5\r\nmykey\r\n$7\r\nmyvalue\r\n".to_owned();
      let tokens = tokenize(&client_request).unwrap();
      let intermediate_representation = parse(&tokens).unwrap();
      let command = interpret(&intermediate_representation).unwrap();

      let expected_tokens = vec![
        Token::Array(3),
        Token::BulkString(3),
        Token::StringValue("SET".to_owned()),
        Token::BulkString(5),
        Token::StringValue("mykey".to_owned()),
        Token::BulkString(7),
        Token::StringValue("myvalue".to_owned()),
      ];
      let expected_intermediate_representation = RespValue::Array(vec![
        RespValue::BulkString("SET".to_owned()),
        RespValue::BulkString("mykey".to_owned()),
        RespValue::BulkString("myvalue".to_owned()),
      ]);
      let expected_command = Command::Set(Set {
        key: "mykey".to_owned(),
        value: "myvalue".to_owned(),
        ..Set::default()
      });

      assert!(
        tokens == expected_tokens,
        "tokenizer: SET with proper key value"
      );
      assert!(
        intermediate_representation == expected_intermediate_representation,
        "parser: SET with proper key value"
      );
      assert!(
        command == expected_command,
        "interperter: SET with proper key value"
      );
    }
  }

  mod test_get {
    use super::*;

    #[test]
    fn should_work_with_proper_key() {
      // SET mykey myvalue
      let client_request = "*2\r\n$3\r\nGET\r\n$5\r\nmykey\r\n".to_owned();
      let tokens = tokenize(&client_request).unwrap();
      let intermediate_representation = parse(&tokens).unwrap();
      let command = interpret(&intermediate_representation).unwrap();

      let expected_tokens = vec![
        Token::Array(2),
        Token::BulkString(3),
        Token::StringValue("GET".to_owned()),
        Token::BulkString(5),
        Token::StringValue("mykey".to_owned()),
      ];
      let expected_intermediate_representation = RespValue::Array(vec![
        RespValue::BulkString("GET".to_owned()),
        RespValue::BulkString("mykey".to_owned()),
      ]);
      let expected_command = Command::Get(Get {
        key: "mykey".to_owned(),
      });

      assert!(tokens == expected_tokens, "tokenizer: GET with proper key");
      assert!(
        intermediate_representation == expected_intermediate_representation,
        "parser: GET with proper key"
      );
      assert!(
        command == expected_command,
        "interperter: GET with proper key"
      );
    }
  }
}
