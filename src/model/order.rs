use serde::{Deserialize, Serialize};
use bson::ordered::OrderedDocument;

#[derive(Serialize, Deserialize, Debug)]
pub struct Order {
  user_id: bson::oid::ObjectId,
  address: OrderedDocument,
  basket: OrderedDocument,
  total_amount: i32,
}

impl Order {
  pub fn new(
    user_id: bson::oid::ObjectId,
    basket: OrderedDocument,
    address: OrderedDocument,
    total_amount: i32,
  ) -> Self {
    Order {
      user_id,
      address,
      basket,
      total_amount,
    }
  }
}
