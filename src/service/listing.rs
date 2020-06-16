use bson::{doc, ordered};
use mongodb::Collection;
use std::vec;

pub struct ListingService {
  collection: Collection,
}

impl ListingService {
  pub fn new(collection: Collection) -> Self {
    ListingService { collection }
  }
  pub fn get(&self) -> Result<std::vec::Vec<bson::ordered::OrderedDocument>, String> {
    let pipeline = vec![
      doc! {
        "$match": doc! {"visible": true}
      },
      doc! {
        "$lookup": doc! {"from": "product", "localField": "product_id", "foreignField": "_id", "as": "product"}
      },
      doc! {
        "$unwind": doc! {"path": "$product", "preserveNullAndEmptyArrays": true}
      },
      doc! {
        "$project": doc! {"product.name": 1, "product.size": 1, "product.price": 1, "product.old_price": 1, "product._id": 1, "product.image_url": 1, "header": 1, "text": 1, "type": 1}
      },
      doc! {
        "$sort": doc! {"priority": -1}
      },
    ];
    match self.collection.aggregate(pipeline.into_iter(), None) {
      Ok(cursor) => {
        let mut listings: Vec<ordered::OrderedDocument> = vec![];
        for result in cursor {
          if let Ok(document) = result {
            listings.push(document);
          } else {
            return Err(String::from("Can't find listings"));
          }
        }
        Ok(listings)
      }
      Err(_e) => Err(String::from("Error while getting listings")),
    }
  }
}
