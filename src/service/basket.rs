use crate::model::basket::{Basket, BasketItem};
use bson::{doc, oid::ObjectId, ordered::OrderedDocument, to_bson, Bson};
use mongodb::{
  error::{Error, ErrorKind},
  results::{InsertOneResult, UpdateResult},
  Collection,
};

#[derive(Clone)]
pub struct BasketService {
  collection: Collection,
}

impl BasketService {
  pub fn new(collection: Collection) -> Self {
    BasketService { collection }
  }

  pub fn get_active(&self, user_id: &str) -> Result<Option<OrderedDocument>, Error> {
    let pipeline = vec![
      doc! {
        "$match": doc! {"active": true, "user_id": ObjectId::with_string(user_id).expect("Id not valid")}
      },
      doc! {
        "$lookup": doc! {"from": "product", "localField": "content.product_id", "foreignField": "_id", "as": "product_info"}
      },
      doc! {
        "$unwind": doc! {"path": "$content.product", "preserveNullAndEmptyArrays": true}
      },
    ];
    match self.collection.aggregate(pipeline.into_iter(), None) {
      Ok(cursor) => {
        let mut baskets: Vec<OrderedDocument> = vec![];
        for result in cursor {
          if let Ok(document) = result {
            baskets.push(document);
          } else {
            return Err(Error::from(ErrorKind::OperationError {
              message: String::from("Can't find active basket"),
            }));
          }
        }
        if baskets.len() > 0 {
          let basket = baskets[0].clone();
          Ok(Some(basket))
        } else {
          Ok(None)
        }
      }
      Err(e) => Err(e),
    }
  }

  pub fn create(&self, basket: &Basket) -> Result<InsertOneResult, Error> {
    let serialized_basket = to_bson(&basket).unwrap();
    if let Bson::Document(mut document) = serialized_basket {
      document.insert("created_at", chrono::Utc::now());
      match self.collection.insert_one(document, None) {
        Ok(insert_result) => Ok(insert_result),
        Err(e) => Err(e),
      }
    } else {
      Err(Error::from(ErrorKind::OperationError {
        message: String::from("Can not create basket"),
      }))
    }
  }

  pub fn update_product_count(
    &self,
    product_id: &str,
    seller_id: &str,
    user_id: &str,
    count: i32,
  ) -> Result<Option<OrderedDocument>, Error> {
    let query = doc! {
      "user_id": ObjectId::with_string(user_id).expect("Id not valid"),
      "content.product_id": ObjectId::with_string(product_id).expect("product_id not valid"),
      "content.seller_id": ObjectId::with_string(seller_id).expect("seller_id not valid"),
      "active": true
    };
    let update = doc! {"$inc": {"content.$.count": count}};
    self.collection.find_one_and_update(query, update, None)
  }

  pub fn add_item(
    &self,
    product_id: &str,
    seller_id: &str,
    listing_id: &str,
    user_id: &str,
  ) -> Result<UpdateResult, Error> {
    let basket_item = BasketItem::new(
      ObjectId::with_string(product_id).expect("product_id not valid"),
      ObjectId::with_string(seller_id).expect("seller_id not valid"),
      ObjectId::with_string(listing_id).expect("listing_id not valid"),
      1,
    );
    match to_bson(&basket_item) {
      Ok(basket_item_doc) => self.collection.update_one(
        doc! {"active": true, "user_id": ObjectId::with_string(&user_id).expect("Id not valid")},
        doc! {"$push": {"content": basket_item_doc}},
        None,
      ),
      _ => Err(Error::from(ErrorKind::OperationError {
        message: String::from("Can not create basket"),
      })),
    }
  }

  pub fn get_product_with_count_one(
    &self,
    product_id: String,
    seller_id: String,
    user_id: String,
  ) -> Result<Option<OrderedDocument>, Error> {
    self.collection.find_one(
      doc! {"user_id": ObjectId::with_string(&user_id).expect("user_id not valid"),"content": {"product_id": ObjectId::with_string(&product_id).expect("product_id not valid"), "seller_id": ObjectId::with_string(&seller_id).expect("seller_id not valid"), "count": 1}, "active": true},
      None
    )
  }

  pub fn remove_product(
    &self,
    product_id: &str,
    seller_id: &str,
    user_id: &str,
  ) -> Result<UpdateResult, Error> {
    self.collection.update_one(
      doc! {"active": true, "user_id": ObjectId::with_string(&user_id).expect("Id not valid")},
      doc! {"$pull": {"content": {"product_id": ObjectId::with_string(&product_id).expect("product_id not valid"), "seller_id": ObjectId::with_string(&seller_id).expect("seller_id not valid")}}},
      None,
    )
  }

  pub fn delete(&self, user_id: &str) -> Result<Option<OrderedDocument>, Error> {
    let query = doc! {
      "user_id": ObjectId::with_string(user_id).expect("Id not valid"),
      "active": true
    };
    let update = doc! {"$set": {"active": false}};
    self.collection.find_one_and_update(query, update, None)
  }
}
