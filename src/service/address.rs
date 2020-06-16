use crate::model::address::Address;
use crate::traits::service::{Creator, Finder, Getter, Updater};
use bson::{doc, ordered};
use bson::{oid::ObjectId, to_bson, Bson};
use mongodb::{
  error::{Error, ErrorKind},
  results::InsertOneResult,
  results::UpdateResult,
  Collection,
};
use std::vec;

#[derive(Clone)]
pub struct AddressService {
  collection: Collection,
}

impl AddressService {
  pub fn new(collection: Collection) -> AddressService {
    AddressService { collection }
  }
}

impl Creator<Address> for AddressService {
  fn create(&self, address: &Address) -> Result<InsertOneResult, Error> {
    let serialized_address = to_bson(&address).unwrap();
    if let Bson::Document(mut document) = serialized_address {
      document.insert("created_at", chrono::Utc::now());
      match self.collection.insert_one(document, None) {
        Ok(insert_result) => Ok(insert_result),
        Err(e) => Err(e),
      }
    } else {
      Err(Error::from(ErrorKind::OperationError {
        message: String::from("Can not create address"),
      }))
    }
  }
}

impl Getter for AddressService {
  fn get_all(&self, id: &str) -> Result<std::vec::Vec<bson::ordered::OrderedDocument>, String> {
    match self.collection.find(
      doc! {"user_id": ObjectId::with_string(id).expect("user_id is not valid")},
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
}

impl Updater<Address> for AddressService {
  fn update(&self, address: &Address, id: &str) -> Result<UpdateResult, Error> {
    let serialized_address = to_bson(&address).unwrap();
    if let Bson::Document(mut document) = serialized_address {
      document.insert("updated_at", chrono::Utc::now());
      self.collection.replace_one(
        doc! {"_id": ObjectId::with_string(id).expect("address id not valid")},
        document,
        None,
      )
    } else {
      Err(Error::from(ErrorKind::OperationError {
        message: String::from("Can not update address"),
      }))
    }
  }
}

impl Finder for AddressService {
  fn find(&self, id: &str) -> Result<Option<bson::ordered::OrderedDocument>, Error> {
    self.collection.find_one(
      doc! {"_id": ObjectId::with_string(id).expect("Id not valid")},
      None,
    )
  }
}
