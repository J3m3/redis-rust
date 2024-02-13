use std::collections::HashMap;

#[derive(Debug)]
pub struct DataBase {
  db: HashMap<String, String>,
}

impl DataBase {
  pub fn new() -> Self {
    DataBase { db: HashMap::new() }
  }

  pub fn get(&self, key: &str) -> Option<&String> {
    self.db.get(key)
  }

  pub fn set(&mut self, key: &str, val: &str) -> Option<String> {
    self.db.insert(key.to_owned(), val.to_owned())
  }
}
