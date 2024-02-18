use std::sync::{Arc, Mutex};

use crate::database::Database;
use crate::resp_server::command::Execute;

use super::{interpreter, parser, tokenizer, ExecutionContext};
use super::{Context, Result};

pub fn generate_response(request: &[u8], db: &Arc<Mutex<Database>>) -> Result<Vec<u8>> {
  let str = std::str::from_utf8(request)
    .context("failed to convert raw binary request to utf-8 string slice")?;

  let tokens = tokenizer::tokenize(str).context("tokenization failed")?;
  let intermediate_representation = parser::parse(&tokens).context("parsing failed")?;
  let command =
    interpreter::interpret(&intermediate_representation).context("interpretation failed")?;

  let context = ExecutionContext { db };
  let response = command
    .execute(&context)
    .context("failed to execute command")?;

  Ok(response)
}

#[cfg(test)]
mod tests_response_generation {
  use super::*;
  use crate::database::*;
  use crate::resp_server::*;

  fn mock_db() -> Arc<Mutex<Database>> {
    Arc::new(Mutex::new(Database::new()))
  }

  #[test]
  fn should_work_with_case_insensitivity() {
    let db = mock_db();

    let client_query = "*2\r\n$4\r\nECHO\r\n$3\r\nhey\r\n".to_owned();
    let response = generate_response(client_query.as_bytes(), &Arc::clone(&db)).unwrap();
    let expected_response = b"+hey\r\n".to_vec();
    assert_eq!(response, expected_response);

    let client_query = "*2\r\n$4\r\necho\r\n$3\r\nhey\r\n".to_owned();
    let response = generate_response(client_query.as_bytes(), &Arc::clone(&db)).unwrap();
    let expected_response = b"+hey\r\n".to_vec();
    assert_eq!(response, expected_response);
  }

  mod test_db_set {
    use super::*;

    #[test]
    fn should_insert_key_value_into_db() {
      let db = mock_db();

      let key = "mykey";
      let value = "myvalue";
      let client_request = format!("*3\r\n$3\r\nSET\r\n$5\r\n{}\r\n$7\r\n{}\r\n", key, value);
      let respone = generate_response(client_request.as_bytes(), &db).unwrap();
      let expected_response = b"+OK\r\n".to_vec();
      assert_eq!(respone, expected_response);

      assert_eq!(value, db.lock().unwrap().get(key).unwrap().value);
      assert_eq!(None, db.lock().unwrap().get(key).unwrap().expire_time);
    }

    #[test]
    fn should_update_value_of_specified_key() {
      let command_type = "SET";
      let key = "mykey";
      let value = "myvalue";
      let data = Data {
        value: value.to_owned(),
        expire_time: None,
      };

      let db = mock_db();
      db.lock().unwrap().set(key, &data);

      let new_value = "newvalue";

      let client_request = format!(
        "*3\r\n${}\r\n{}\r\n${}\r\n{}\r\n${}\r\n{}\r\n",
        command_type.len(),
        command_type,
        key.len(),
        key,
        new_value.len(),
        new_value
      );
      let respone = generate_response(client_request.as_bytes(), &db).unwrap();
      let expected_response = b"+OK\r\n".to_vec();
      assert_eq!(respone, expected_response);

      assert_eq!(new_value, db.lock().unwrap().get(key).unwrap().value);
      assert_eq!(None, db.lock().unwrap().get(key).unwrap().expire_time);
    }
  }

  mod test_db_get {
    use super::*;

    #[test]
    fn should_retrieve_value_associated_with_specified_key_from_db() {
      let command_type = "GET";
      let key = "mykey";
      let value = "myvalue";
      let data = Data {
        value: value.to_owned(),
        expire_time: None,
      };

      let db = mock_db();
      db.lock().unwrap().set(key, &data);

      let client_request = format!(
        "*2\r\n${}\r\n{}\r\n${}\r\n{}\r\n",
        command_type.len(),
        command_type,
        key.len(),
        key
      );
      let respone = generate_response(client_request.as_bytes(), &db).unwrap();
      let expected_response = format!("+{}\r\n", data.value).as_bytes().to_vec();
      println!(
        "response: {:?} / expected_response: {:?}",
        String::from_utf8(respone.clone()).unwrap(),
        String::from_utf8(expected_response.clone()).unwrap()
      );
      assert_eq!(respone, expected_response);
    }

    #[test]
    fn should_generate_nil_response_when_key_does_not_exist() {
      let command_type = "GET";
      let key = "randomkey";

      let db = mock_db();

      let client_request = format!(
        "*2\r\n${}\r\n{}\r\n${}\r\n{}\r\n",
        command_type.len(),
        command_type,
        key.len(),
        key
      );
      let respone = generate_response(client_request.as_bytes(), &db).unwrap();
      let expected_response = format!("{}\r\n", NULL_BULK_STRING).as_bytes().to_vec();
      assert_eq!(respone, expected_response);
    }
  }
}
