use actix_service::Service;
use actix_web::http::{header, HeaderValue};
use actix_web::{web, App, HttpServer};
use mongodb::{options::ClientOptions, Client, Collection};

mod controller;
mod service;

pub struct AppState {
  listing_collection: Collection,
  product_collection: Collection,
  content_collection: Collection,
}

#[macro_use]
extern crate dotenv_codegen;

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
  let client_options = ClientOptions::parse(dotenv!("DB_URL")).unwrap();
  let client = Client::with_options(client_options).unwrap();
  let db = client.database(dotenv!("DB_NAME"));
  let listing_collection = db.collection(dotenv!("DB_LISTING_COLLECTION"));
  let product_collection = db.collection(dotenv!("DB_PRODUCT_COLLECTION"));
  let content_collection = db.collection(dotenv!("DB_CONTENT_COLLECTION"));

  HttpServer::new(move || {
    App::new()
      .data(AppState {
        listing_collection: listing_collection.clone(),
        product_collection: product_collection.clone(),
        content_collection: content_collection.clone(),
      })
      .wrap_fn(|mut req, srv| {
        let mut token = "";
        let headers = req.headers().clone();
        match headers.get("cookie") {
          Some(cookie) => {
            let split = cookie.to_str().unwrap().split(";");
            let cookies: Vec<&str> = split.collect();
            for cookie in &cookies {
              if cookie.contains("access_token=") {
                let token_cookie: Vec<&str> = cookie.split("=").collect();
                token = token_cookie[1];
                println!("token: {}", token);
              }
            }
          }
          None => println!("none o.o",),
        };
        if token.len() > 0 {
          req.headers_mut().insert(
            header::HeaderName::from_static("user_id"),
            header::HeaderValue::from_str(token).unwrap(),
          );
        }
        let fut = srv.call(req);
        async {
          let res = fut.await?;
          Ok(res)
        }
      })
      .service(web::scope("/listings").route("", web::get().to(controller::listing::get)))
      .service(web::scope("/contents").route("", web::get().to(controller::content::get)))
  })
  .bind("0.0.0.0:3003")?
  .run()
  .await
}
