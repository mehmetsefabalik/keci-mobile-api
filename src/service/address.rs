use crate::model::address::Address;
use crate::traits::create::Creator;
use bson::{doc, ordered};
use bson::{oid::ObjectId, to_bson, Bson};
use mongodb::{
  error::{Error, ErrorKind},
  results::InsertOneResult,
  results::UpdateResult,
  Collection,
};
use std::vec;

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
    if let Bson::Document(document) = serialized_address {
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
  address: &Address,
) -> Result<UpdateResult, Error> {
  let serialized_address = to_bson(&address).unwrap();
  if let Bson::Document(document) = serialized_address {
    collection.replace_one(
      doc! {"_id": ObjectId::with_string(_id).expect("address id not valid")},
      document,
      None,
    )
  } else {
    Err(Error::from(ErrorKind::OperationError {
      message: String::from("Can not update address"),
    }))
  }
}
