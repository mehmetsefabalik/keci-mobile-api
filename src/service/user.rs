use crate::model::user::User;
use crate::traits::service::Creator;
use bson::{doc, oid::ObjectId, ordered::OrderedDocument, to_bson, Bson};
use mongodb::{
  error::{Error, ErrorKind},
  results::{InsertOneResult, UpdateResult},
  Collection,
};

#[derive(Clone)]
pub struct UserService {
  collection: Collection,
}

impl UserService {
  pub fn new(collection: Collection) -> Self {
    UserService { collection }
  }
  pub fn get(&self, phone: &String) -> Result<Option<OrderedDocument>, Error> {
    self.collection.find_one(doc! {"phone": phone}, None)
  }

  pub fn create_anon(&self) -> Result<InsertOneResult, Error> {
    self.collection.insert_one(doc! {"created_at": chrono::Utc::now()}, None)
  }

  pub fn register(
    &self,
    user_id: &str,
    phone: &str,
    password: &str,
  ) -> Result<UpdateResult, Error> {
    self.collection.update_one(
      doc! {"_id": ObjectId::with_string(&user_id).expect("Id not valid")},
      doc! {"$set": {"phone": String::from(phone), "password": String::from(password)}, "updated_at": chrono::Utc::now()},
      None,
    )
  }
}

impl Creator<User> for UserService {
  fn create(&self, user: &User) -> Result<InsertOneResult, Error> {
    let serialized_user = to_bson(&user).unwrap();
    if let Bson::Document(mut document) = serialized_user {
      document.insert("created_at", chrono::Utc::now());
      match self.collection.insert_one(document, None) {
        Ok(insert_result) => Ok(insert_result),
        Err(e) => Err(e),
      }
    } else {
      Err(Error::from(ErrorKind::OperationError {
        message: String::from("Can not create User"),
      }))
    }
  }
}
