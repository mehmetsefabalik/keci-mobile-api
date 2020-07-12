use actix_web::{web, App, HttpServer};
use mongodb::{options::ClientOptions, Client};
use service::address::AddressService;
use service::basket::BasketService;
use service::listing::ListingService;
use service::order::OrderService;
use service::user::UserService;

mod action;
mod controller;
mod middleware;
mod model;
mod service;
mod traits;

pub struct ServiceContainer {
  address: AddressService,
  basket: BasketService,
  listing: ListingService,
  user: UserService,
  order: OrderService,
}

impl ServiceContainer {
  pub fn new(
    address: AddressService,
    basket: BasketService,
    listing: ListingService,
    user: UserService,
    order: OrderService,
  ) -> Self {
    ServiceContainer {
      address,
      basket,
      listing,
      user,
      order,
    }
  }
}

pub struct AppState {
  service_container: ServiceContainer,
}

#[macro_use]
extern crate dotenv_codegen;

#[actix_rt::main]
async fn run(client: Client) -> std::io::Result<()> {
  let db = client.database(dotenv!("DB_NAME"));
  let listing_collection = db.collection(dotenv!("DB_LISTING_COLLECTION"));
  let user_collection = db.collection(dotenv!("DB_USER_COLLECTION"));
  let basket_collection = db.collection(dotenv!("DB_BASKET_COLLECTION"));
  let address_collection = db.collection(dotenv!("DB_ADDRESS_COLLECTION"));
  let order_collection = db.collection(dotenv!("DB_ORDER_COLLECTION"));

  HttpServer::new(move || {
    let service_container = ServiceContainer::new(
      AddressService::new(address_collection.clone()),
      BasketService::new(basket_collection.clone()),
      ListingService::new(listing_collection.clone()),
      UserService::new(user_collection.clone()),
      OrderService::new(order_collection.clone()),
    );
    App::new()
      .data(AppState { service_container })
      .service(
        web::scope("/listings")
          .route("", web::get().to(controller::listing::get))
          .route(
            "{seller}",
            web::get().to(controller::listing::get_for_seller),
          ),
      )
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
          .route("", web::post().to(controller::order::create))
          .route("/{id}", web::get().to(controller::order::find))
          .route("", web::get().to(controller::order::get_all)),
      )
  })
  .bind("0.0.0.0:3003")?
  .run()
  .await
}

fn main() -> std::io::Result<()> {
  println!("number of cpus: {}", num_cpus::get());
  let client_options = ClientOptions::parse(dotenv!("DB_URL")).unwrap();
  let client = Client::with_options(client_options).unwrap();
  run(client)
}
