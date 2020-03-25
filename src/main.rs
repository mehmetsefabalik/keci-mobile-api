use actix_web::{web, App, HttpServer};
use mongodb::{options::ClientOptions, Client, Collection};

mod controller;
mod service;

pub struct AppState {
  listing_collection: Collection,
}

#[macro_use]
extern crate dotenv_codegen;

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
  let client_options = ClientOptions::parse(dotenv!("DB_URL")).unwrap();
  let client = Client::with_options(client_options).unwrap();
  let db = client.database(dotenv!("DB_NAME"));
  let listing_collection = db.collection(dotenv!("DB_LISTING_COLLECTION"));

  HttpServer::new(move || {
    App::new()
      .data(AppState {
        listing_collection: listing_collection.clone(),
      })
      .service(web::scope("/listing").route("", web::get().to(controller::listing::get)))
  })
  .bind("0.0.0.0:3003")?
  .run()
  .await
}
