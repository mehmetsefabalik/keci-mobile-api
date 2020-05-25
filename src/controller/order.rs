use crate::action::order::{create_order, CreateOrderResponse};
use crate::traits::service::Getter;
use actix_web::{web, HttpRequest, HttpResponse, Responder};
use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub struct CreateOrderBody {
  address_id: String,
}

pub async fn create(
  request: HttpRequest,
  app_data: web::Data<crate::AppState>,
  body: web::Json<CreateOrderBody>,
) -> impl Responder {
  match request.headers().get("user_id") {
    Some(user_id_header) => {
      // user
      match user_id_header.to_str() {
        Ok(user_id_str) => {
          let user_id = String::from(user_id_str);
          let result = web::block(move || {
            create_order(
              app_data.service_container.order.clone(),
              app_data.service_container.basket.clone(),
              app_data.service_container.address.clone(),
              user_id.clone(),
              body.address_id.clone(),
            )
          })
          .await;

          match result {
            Ok(response) => match response {
              CreateOrderResponse::OrderCreated(id) => HttpResponse::Ok().json(id),
              CreateOrderResponse::ActiveBasketNotFound => {
                HttpResponse::BadRequest().body("Active Basket Not Found")
              }
              CreateOrderResponse::AddressNotFound => {
                HttpResponse::BadRequest().body("Address Not Found")
              }
              CreateOrderResponse::BasketToDeleteNotFound => {
                HttpResponse::BadRequest().body("Basket To Delete Not Found")
              }
            },
            Err(_e) => HttpResponse::InternalServerError().finish(),
          }
        }
        Err(e) => {
          println!(
            "Error while getting string of user_id header value, {:?}",
            e
          );
          HttpResponse::InternalServerError().finish()
        }
      }
    }
    None => HttpResponse::Unauthorized().finish(),
  }
}

pub async fn get_all(request: HttpRequest, app_data: web::Data<crate::AppState>) -> impl Responder {
  match request.headers().get("user_id") {
    Some(user_id_header) => match user_id_header.to_str() {
      Ok(user_id_str) => {
        let user_id = String::from(user_id_str);
        let orders = web::block(move || app_data.service_container.order.get_all(&user_id)).await;
        match orders {
          Ok(response) => HttpResponse::Ok().json(response),
          Err(e) => {
            println!("Error whil getting orders, {:?}", e);
            HttpResponse::InternalServerError().finish()
          }
        }
      }
      Err(_e) => {
        println!(
          "Error while stringifying user_id header, {:?}",
          user_id_header
        );
        HttpResponse::BadRequest().finish()
      }
    },
    None => HttpResponse::Unauthorized().finish(),
  }
}

#[derive(Deserialize)]
pub struct FindPath {
  pub id: String,
}

pub async fn find(
  request: HttpRequest,
  path: web::Path<FindPath>,
  app_data: web::Data<crate::AppState>,
) -> impl Responder {
  match request.headers().get("user_id") {
    Some(user_id_header) => match user_id_header.to_str() {
      Ok(user_id_str) => {
        let user_id = String::from(user_id_str);
        let order =
          web::block(move || app_data.service_container.order.find(&path.id, &user_id)).await;
        match order {
          Ok(response) => HttpResponse::Ok().json(response),
          Err(e) => {
            println!("Error while finding order, {:?}", e);
            HttpResponse::InternalServerError().finish()
          }
        }
      }
      Err(_e) => {
        println!(
          "Error while stringifying user_id header, {:?}",
          user_id_header
        );
        HttpResponse::BadRequest().finish()
      }
    },
    None => HttpResponse::Unauthorized().finish(),
  }
}
