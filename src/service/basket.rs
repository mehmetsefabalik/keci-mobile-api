use bson::{doc, oid::ObjectId, ordered::OrderedDocument, to_bson, Bson};
use mongodb::{
  error::{Error, ErrorKind},
  results::{InsertOneResult, UpdateResult},
  Collection,
};
use serde::{Deserialize, Serialize};

pub fn get_active(
  collection: Collection,
  user_id: String,
) -> Result<(Option<OrderedDocument>, Collection, String), Error> {
  match collection.find_one(
    doc! {"active": true, "user_id": ObjectId::with_string(&user_id).expect("Id not valid")},
    None,
  ) {
    Ok(result) => Ok((result, collection, user_id)),
    Err(e) => Err(e),
  }
}

#[derive(Serialize, Deserialize, Debug)]
struct BasketItem {
  product_id: ObjectId,
  count: i16,
}

#[derive(Serialize, Deserialize, Debug)]
struct Basket {
  user_id: ObjectId,
  content: Vec<BasketItem>,
  active: bool,
}

pub fn create(
  collection: Collection,
  product_id: &str,
  user_id: &str,
) -> Result<InsertOneResult, Error> {
  let basket_item = BasketItem {
    product_id: ObjectId::with_string(product_id).expect("Invalid ObjectId string"),
    count: 1,
  };
  let basket = Basket {
    user_id: ObjectId::with_string(user_id).expect("Invalid ObjectId string"),
    content: vec![basket_item],
    active: true,
  };
  let serialized_basket = to_bson(&basket).unwrap();
  if let Bson::Document(document) = serialized_basket {
    match collection.insert_one(document, None) {
      Ok(insert_result) => Ok(insert_result),
      Err(e) => Err(e),
    }
  } else {
    Err(Error::from(ErrorKind::OperationError {
      message: String::from("Can not create basket"),
    }))
  }
}

pub fn increment_product_count(collection: Collection, product_id: &str, user_id: &str) -> Result<UpdateResult, Error> {
  collection.update_one(doc! {"user_id": ObjectId::with_string(&user_id).expect("Id not valid"), "content.product_id": ObjectId::with_string(&product_id).expect("Id not valid"), "active": true}, doc!{"$inc": {"content.$.count": 1}}, None)
}

pub fn get_active_with_product_id(collection: Collection, product_id: &str, user_id: &str) -> Result<Option<OrderedDocument>, Error> {
  collection.find_one(doc! {"user_id": ObjectId::with_string(&user_id).expect("Id not valid"), "content.product_id": ObjectId::with_string(&product_id).expect("Id not valid"), "active": true}, None)
}
