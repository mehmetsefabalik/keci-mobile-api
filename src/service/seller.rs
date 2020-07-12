use bson::{doc, ordered};
use mongodb::{error::Error, Collection};

pub struct SellerService {
  collection: Collection,
}

impl SellerService {
  pub fn new(collection: Collection) -> Self {
    SellerService { collection }
  }

  pub fn get(&self, name: &str) -> Result<Option<ordered::OrderedDocument>, Error> {
    self.collection.find_one(doc! {"name": name}, None)
  }
}
