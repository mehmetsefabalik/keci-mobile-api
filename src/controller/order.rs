use crate::action::order::{create_order, CreateOrderResponse};
use actix_web::{web, HttpRequest, HttpResponse, Responder};
use serde::{Deserialize};

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
            Ok(response) => {
              match response {
                CreateOrderResponse::OrderCreated => {
                  HttpResponse::Ok().finish()
                },
                CreateOrderResponse::ActiveBasketNotFound => {
                  HttpResponse::BadRequest().body("Active Basket Not Found")
                },
                CreateOrderResponse::AddressNotFound => {
                  HttpResponse::BadRequest().body("Address Not Found")
                }
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
    None => {
      HttpResponse::Unauthorized().finish()
    }
  }
}