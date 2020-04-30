use mongodb::Collection;
use crate::model::order::Order;
use crate::traits::service::Creator;
use mongodb::error::{Error, ErrorKind};
use mongodb::results::InsertOneResult;
use bson::{to_bson, Bson};

#[derive(Clone)]
pub struct OrderService {
  collection: Collection
}

impl OrderService {
  pub fn new(collection: Collection) -> Self {
    OrderService {
      collection
    }
  }
}

impl Creator<Order> for OrderService {
  fn create(&self, order: &Order) -> Result<InsertOneResult, Error> {
    let serialized_order = to_bson(&order).unwrap();
    if let Bson::Document(document) = serialized_order {
      match self.collection.insert_one(document, None) {
        Ok(insert_result) => Ok(insert_result),
        Err(e) => Err(e),
      }
    } else {
      Err(Error::from(ErrorKind::OperationError {
        message: String::from("Can not create order"),
      }))
    }
  }
}