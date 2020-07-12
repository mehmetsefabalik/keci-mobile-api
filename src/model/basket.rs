use bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct BasketItem {
  product_id: ObjectId,
  seller_id: ObjectId,
  listing_id: ObjectId,
  count: i16,
}

impl BasketItem {
  pub fn new(
    product_id: ObjectId,
    seller_id: ObjectId,
    listing_id: ObjectId,
    count: i16,
  ) -> BasketItem {
    BasketItem {
      product_id,
      seller_id,
      listing_id,
      count,
    }
  }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Basket {
  user_id: ObjectId,
  content: Vec<BasketItem>,
  active: bool,
}

impl Basket {
  pub fn new(user_id: ObjectId, content: Vec<BasketItem>, active: bool) -> Basket {
    Basket {
      user_id,
      content,
      active,
    }
  }
}
