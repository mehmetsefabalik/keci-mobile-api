use mongodb::error::Error;
use mongodb::results::{InsertOneResult, UpdateResult};

pub trait Creator<T> {
  fn create(&self, model: &T) -> Result<InsertOneResult, Error>;
}

pub trait Getter {
  fn get_all(&self, id: &str) -> Result<std::vec::Vec<bson::ordered::OrderedDocument>, String>;
}


pub trait Updater<T> {
  fn update(&self, model: &T, id: &str) -> Result<UpdateResult, Error>;
}
