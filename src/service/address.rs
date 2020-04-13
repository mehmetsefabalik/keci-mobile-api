use bson::{oid::ObjectId, to_bson, Bson};
use mongodb::{
  error::{Error, ErrorKind},
  results::InsertOneResult,
  Collection,
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
struct Address {
  user_id: ObjectId,
  name: String,
  surname: String,
  title: String,
  text: String,
  district_id: i32,
  neighborhood_id: i32,
}

pub fn create(
  collection: &Collection,
  user_id: &str,
  name: &String,
  surname: &String,
  title: &String,
  text: &String,
  district_id: i32,
  neighborhood_id: i32,
) -> Result<InsertOneResult, Error> {
  let name = name.clone();
  let surname = surname.clone();
  let title = title.clone();
  let text = text.clone();
  let address = Address {
    user_id: ObjectId::with_string(user_id).expect("Invalid ObjectId string"),
    name: name,
    surname: surname,
    title: title,
    text: text,
    district_id: district_id,
    neighborhood_id: neighborhood_id,
  };
  let serialized_address = to_bson(&address).unwrap();
  if let Bson::Document(document) = serialized_address {
    match collection.insert_one(document, None) {
      Ok(insert_result) => Ok(insert_result),
      Err(e) => Err(e),
    }
  } else {
    Err(Error::from(ErrorKind::OperationError {
      message: String::from("Can not create address"),
    }))
  }
}
