use bson::{doc, ordered};
use bson::{oid::ObjectId, to_bson, Bson};
use mongodb::{
  error::{Error, ErrorKind},
  results::InsertOneResult,
  results::UpdateResult,
  Collection,
};
use serde::{Deserialize, Serialize};
use std::vec;

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

pub fn get_all(
  collection: &Collection,
  user_id: &str,
) -> Result<std::vec::Vec<bson::ordered::OrderedDocument>, String> {
  match collection.find(
    doc! {"user_id": ObjectId::with_string(user_id).expect("user_id is not valid")},
    None,
  ) {
    Ok(cursor) => {
      let mut addresses: Vec<ordered::OrderedDocument> = vec![];
      for result in cursor {
        if let Ok(document) = result {
          addresses.push(document);
        } else {
          return Err(String::from("Can't find addresses"));
        }
      }
      Ok(addresses)
    }
    Err(_e) => Err(String::from("Error while getting addresses")),
  }
}

pub fn update(
  collection: &Collection,
  _id: &str,
  user_id: &str,
  name: &String,
  surname: &String,
  title: &String,
  text: &String,
  district_id: i32,
  neighborhood_id: i32,
) -> Result<UpdateResult, Error> {
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
    match collection.replace_one(
      doc! {"_id": ObjectId::with_string(_id).expect("address id not valid")},
      document,
      None,
    ) {
      Ok(insert_result) => Ok(insert_result),
      Err(e) => Err(e),
    }
  } else {
    Err(Error::from(ErrorKind::OperationError {
      message: String::from("Can not create address"),
    }))
  }
}