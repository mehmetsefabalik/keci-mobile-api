use crate::model::order::Order;
use crate::traits::service::{Creator, Getter};
use bson::{doc, oid::ObjectId, ordered, to_bson, Bson};
use mongodb::error::{Error, ErrorKind};
use mongodb::results::InsertOneResult;
use mongodb::Collection;

#[derive(Clone)]
pub struct OrderService {
  collection: Collection,
}

impl OrderService {
  pub fn new(collection: Collection) -> Self {
    OrderService { collection }
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

impl Getter for OrderService {
  fn get_all(&self, id: &str) -> Result<std::vec::Vec<bson::ordered::OrderedDocument>, String> {
    match self.collection.find(
      doc! {"user_id": ObjectId::with_string(id).expect("user_id is not valid")},
      None,
    ) {
      Ok(cursor) => {
        let mut orders: Vec<ordered::OrderedDocument> = vec![];
        for result in cursor {
          if let Ok(document) = result {
            orders.push(document);
          } else {
            return Err(String::from("Can't find orders"));
          }
        }
        Ok(orders)
      }
      Err(_e) => Err(String::from("Error while getting orders")),
    }
  }
}
