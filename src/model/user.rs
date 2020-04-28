use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct User {
  phone: String,
  password: String,
}

impl User {
  pub fn new(phone: &str, password: &str) -> User {
    User {
      phone: String::from(phone),
      password: String::from(password),
    }
  }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
  pub sub: String,
  pub user_type: String,
}
