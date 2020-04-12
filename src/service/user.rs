use crate::controller::user::CreateUser;
use actix_web::web::Json;
use bson::{doc, oid::ObjectId, to_bson, Bson};
use mongodb::{
  error::{Error, ErrorKind},
  results::{InsertOneResult, UpdateResult},
  Collection,
};
use serde::{Deserialize, Serialize};

pub fn get(
  collection: Collection,
  user: Json<CreateUser>,
) -> Result<
  (
    Option<bson::ordered::OrderedDocument>,
    Collection,
    Json<CreateUser>,
  ),
  Error,
> {
  match collection.find_one(doc! {"phone": String::from(&user.phone)}, None) {
    Ok(result) => Ok((result, collection, user)),
    Err(e) => Err(e),
  }
}

#[derive(Serialize, Deserialize, Debug)]
struct User {
  phone: String,
  password: String,
}

pub fn create(
  collection: Collection,
  phone: &str,
  password: &str,
) -> Result<InsertOneResult, Error> {
  let user = User {
    phone: String::from(phone),
    password: String::from(password),
  };
  let serialized_user = to_bson(&user).unwrap();
  if let Bson::Document(document) = serialized_user {
    match collection.insert_one(document, None) {
      Ok(insert_result) => Ok(insert_result),
      Err(e) => Err(e),
    }
  } else {
    Err(Error::from(ErrorKind::OperationError {
      message: String::from("Can not create User"),
    }))
  }
}

pub fn create_anon(collection: Collection) -> Result<InsertOneResult, Error> {
  collection.insert_one(doc! {}, None)
}

pub fn register(
  collection: Collection,
  user_id: &str,
  phone: &str,
  password: &str,
) -> Result<UpdateResult, Error> {
  collection.update_one(
    doc! {"_id": ObjectId::with_string(&user_id).expect("Id not valid")},
    doc! {"$set": {"phone": String::from(phone), "password": String::from(password)}},
    None,
  )
}
