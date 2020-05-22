use bson::ordered::OrderedDocument;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Order {
  user_id: bson::oid::ObjectId,
  address: OrderedDocument,
  basket: OrderedDocument,
  status: i32,
}

impl Order {
  pub fn new(
    user_id: bson::oid::ObjectId,
    basket: OrderedDocument,
    address: OrderedDocument,
    status: Status,
  ) -> Self {
    Order {
      user_id,
      address,
      basket,
      status: status as i32,
    }
  }
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Status {
  Cancelled,
  Taken,
  Preparing,
  Shipping,
  Shipped,
}
