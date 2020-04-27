use actix_web::{web, App, HttpServer};
use mongodb::{options::ClientOptions, Client, Collection};
use service::address::AddressService;
use service::basket::BasketService;

mod controller;
mod middleware;
mod model;
mod service;
mod traits;
mod action;

pub struct ServiceContainer {
  address: AddressService,
  basket: BasketService,
}

impl ServiceContainer {
  pub fn new(address: AddressService, basket: BasketService) -> Self {
    ServiceContainer { address, basket }
  }
}

pub struct AppState {
  listing_collection: Collection,
  user_collection: Collection,
  service_container: ServiceContainer,
}

#[macro_use]
extern crate dotenv_codegen;

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
  let client_options = ClientOptions::parse(dotenv!("DB_URL")).unwrap();
  let client = Client::with_options(client_options).unwrap();
  let db = client.database(dotenv!("DB_NAME"));
  let listing_collection = db.collection(dotenv!("DB_LISTING_COLLECTION"));
  let user_collection = db.collection(dotenv!("DB_USER_COLLECTION"));
  let basket_collection = db.collection(dotenv!("DB_BASKET_COLLECTION"));
  let address_collection = db.collection(dotenv!("DB_ADDRESS_COLLECTION"));

  HttpServer::new(move || {
    let service_container = ServiceContainer::new(
      AddressService::new(address_collection.clone()),
      BasketService::new(basket_collection.clone()),
    );
    App::new()
      .data(AppState {
        listing_collection: listing_collection.clone(),
        user_collection: user_collection.clone(),
        service_container,
      })
      .wrap(
        actix_cors::Cors::new()
          .allowed_origin(dotenv!("ALLOWED_ORIGIN"))
          .finish(),
      )
      .service(web::scope("/listings").route("", web::get().to(controller::listing::get)))
      .service(
        web::scope("/basket")
          .wrap(middleware::user::Resolve)
          .route("", web::post().to(controller::basket::add))
          .route("", web::get().to(controller::basket::get_active))
          .route("", web::patch().to(controller::basket::update)),
      )
      .service(
        web::scope("/users")
          .wrap(middleware::user::Resolve)
          .route("", web::post().to(controller::user::create))
          .route("/validate", web::post().to(controller::user::login)),
      )
      .service(
        web::scope("/addresses")
          .wrap(middleware::user::Resolve)
          .route("", web::post().to(controller::address::create))
          .route(
            "/{address_id}",
            web::patch().to(controller::address::update),
          )
          .route("", web::get().to(controller::address::get_all)),
      )
      .service(
        web::scope("/orders")
          .wrap(middleware::user::Resolve)
          .route("", web::post().to(controller::address::create)),
      )
  })
  .bind("0.0.0.0:3003")?
  .run()
  .await
}
