use bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Address {
  user_id: ObjectId,
  name: String,
  surname: String,
  title: String,
  text: String,
  district_id: i32,
  neighborhood_id: i32,
}

impl Address {
  pub fn new(
    user_id: ObjectId,
    name: &str,
    surname: &str,
    title: &str,
    text: &str,
    district_id: i32,
    neighborhood_id: i32,
  ) -> Address {
    Address {
      user_id,
      name: String::from(name),
      surname: String::from(surname),
      title: String::from(title),
      text: String::from(text),
      district_id,
      neighborhood_id,
    }
  }
}
