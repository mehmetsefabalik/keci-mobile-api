use crate::model::basket::{Basket, BasketItem};
use bson::{doc, oid::ObjectId, ordered::OrderedDocument, to_bson, Bson};
use mongodb::{
  error::{Error, ErrorKind},
  results::{InsertOneResult, UpdateResult},
  Collection,
};

pub fn get_active(
  collection: Collection,
  user_id: String,
) -> Result<(Option<OrderedDocument>, Collection, String), String> {
  let pipeline = vec![
    doc! {
      "$match": doc! {"active": true, "user_id": ObjectId::with_string(&user_id).expect("Id not valid")}
    },
    doc! {
      "$lookup": doc! {"from": "product", "localField": "content.product_id", "foreignField": "_id", "as": "product_info"}
    },
    doc! {
      "$unwind": doc! {"path": "$content.product", "preserveNullAndEmptyArrays": true}
    },
  ];
  match collection.aggregate(pipeline.into_iter(), None) {
    Ok(cursor) => {
      let mut baskets: Vec<OrderedDocument> = vec![];
      for result in cursor {
        if let Ok(document) = result {
          baskets.push(document);
        } else {
          return Err(String::from("Can't find active basket"));
        }
      }
      if baskets.len() > 0 {
        let basket = baskets[0].clone();
        Ok((Some(basket), collection, user_id))
      } else {
        Ok((None, collection, user_id))
      }
    }
    Err(_e) => Err(String::from("Error while getting active basket")),
  }
}

pub fn create(collection: Collection, basket: &Basket) -> Result<InsertOneResult, Error> {
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

pub fn update_product_count(
  collection: &Collection,
  product_id: &String,
  user_id: &String,
  count: i32,
) -> Result<Option<OrderedDocument>, Error> {
  collection.find_one_and_update(doc! {"user_id": ObjectId::with_string(user_id).expect("Id not valid"), "content.product_id": ObjectId::with_string(product_id).expect("Id not valid"), "active": true}, doc!{"$inc": {"content.$.count": count}}, None)
}

pub fn add_item(
  collection: Collection,
  product_id: &str,
  user_id: &str,
) -> Result<UpdateResult, Error> {
  let basket_item = BasketItem::new(
    ObjectId::with_string(product_id).expect("Invalid ObjectId string"),
    1,
  );
  match to_bson(&basket_item) {
    Ok(basket_item_doc) => collection.update_one(
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
  collection: Collection,
  product_id: String,
  user_id: String,
) -> Result<(Option<OrderedDocument>, Collection, String, String), Error> {
  match collection.find_one(
    doc! {"user_id": ObjectId::with_string(&user_id).expect("Id not valid"),"content": {"product_id": ObjectId::with_string(&product_id).expect("Id not valid"), "count": 1}, "active": true},
    None
  ) {
    Ok(doc) => Ok((doc, collection, product_id, user_id)),
    Err(e) => Err(e)
  }
}

pub fn remove_product(
  collection: &Collection,
  product_id: &String,
  user_id: &String,
) -> Result<UpdateResult, Error> {
  collection.update_one(
      doc! {"active": true, "user_id": ObjectId::with_string(&user_id).expect("Id not valid")},
      doc! {"$pull": {"content": {"product_id": ObjectId::with_string(&product_id).expect("Id not valid")}}},
      None,
    )
}
