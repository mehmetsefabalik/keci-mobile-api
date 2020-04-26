use bson::{doc, oid::ObjectId, to_bson, Bson};
use mongodb::{
  error::{Error, ErrorKind},
  results::{InsertOneResult, UpdateResult},
  Collection,
};
use crate::model::user::User;

pub fn get(
  collection: Collection,
  phone: &String,
) -> Result<
  (
    Option<bson::ordered::OrderedDocument>,
    Collection
  ),
  Error,
> {
  match collection.find_one(doc! {"phone": phone}, None) {
    Ok(result) => Ok((result, collection)),
    Err(e) => Err(e),
  }
}

pub fn create(
  collection: Collection,
  user: &User,
) -> Result<InsertOneResult, Error> {
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
