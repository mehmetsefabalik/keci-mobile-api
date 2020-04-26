use mongodb::error::Error;
use mongodb::results::InsertOneResult;

pub trait Creator<T> {
  fn create(&self, model: &T) -> Result<InsertOneResult, Error>;
}
