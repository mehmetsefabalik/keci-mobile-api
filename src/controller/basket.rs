use crate::action::basket::{add_to_basket, decrement_product_count};
use crate::action::user::create_anon_with_basket;
use actix_web::{http, web, HttpRequest, HttpResponse, Responder};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
pub struct Body {
  pub product_id: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Response {
  pub id: bson::Bson,
  pub message: String,
}

pub async fn add(
  request: HttpRequest,
  app_data: web::Data<crate::AppState>,
  body: web::Json<Body>,
) -> impl Responder {
  match request.headers().get("user_id") {
    Some(user_id_header) => {
      // user
      match user_id_header.to_str() {
        Ok(user_id_str) => {
          let user_id = String::from(user_id_str);
          let result = web::block(move || {
            add_to_basket(
              app_data.service_container.basket.clone(),
              user_id.clone(),
              body.product_id.clone(),
            )
          })
          .await;

          match result {
            Ok(_response) => HttpResponse::Ok().finish(),
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
      // anon
      let result = web::block(move || {
        create_anon_with_basket(
          app_data.service_container.user.clone(),
          app_data.service_container.basket.clone(),
          body.product_id.clone(),
        )
      })
      .await;

      match result {
        Ok(cookie) => HttpResponse::Ok()
          .header(
            "Set-Cookie",
            http::header::HeaderValue::from_str(&cookie).unwrap(),
          )
          .finish(),
          Err(e) => {
            println!("{:?}", e);
            HttpResponse::InternalServerError().finish()
          }
      }
    }
  }
}

pub async fn get_active(
  request: HttpRequest,
  app_data: web::Data<crate::AppState>,
) -> impl Responder {
  match request.headers().get("user_id") {
    Some(user_id_header) => match user_id_header.to_str() {
      Ok(user_id_str) => {
        let user_id = String::from(user_id_str);
        let active_basket_result =
          web::block(move || app_data.service_container.basket.get_active(&user_id)).await;
        match active_basket_result {
          Ok(active_basket_option) => match active_basket_option {
            Some(document) => HttpResponse::Ok().json(document),
            None => HttpResponse::NotFound().body("active basket not exists"),
          },
          Err(e) => {
            println!("Error while creating anon user: {:?}", e);
            HttpResponse::new(http::StatusCode::INTERNAL_SERVER_ERROR)
          }
        }
      }
      Err(e) => {
        println!("Error while stringifying user_id header, {:?}", e);
        HttpResponse::Unauthorized().body("user id is not type of string")
      }
    },
    None => HttpResponse::Unauthorized().finish(),
  }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct UpdateBody {
  pub product_id: String,
  pub count: i32,
}

pub async fn update(
  request: HttpRequest,
  body: web::Json<UpdateBody>,
  app_data: web::Data<crate::AppState>,
) -> impl Responder {
  match request.headers().get("user_id") {
    Some(user_id_header) => match user_id_header.to_str() {
      Ok(user_id_str) => {
        let user_id = String::from(user_id_str);
        if body.count > 0 {
          let update_product_count_response = web::block(move || {
            app_data
              .service_container
              .basket
              .update_product_count(&body.product_id, &user_id, 1)
          })
          .await;

          match update_product_count_response {
            Ok(option) => match option {
              Some(document) => HttpResponse::Ok().json(document),
              None => HttpResponse::NotFound().body("product does not exist in basket"),
            },
            Err(e) => {
              println!("Error while updating product count, {:?}", e);
              HttpResponse::InternalServerError().finish()
            }
          }
        } else {
          let decrement_product_count_result = web::block(move || {
            decrement_product_count(
              &app_data.service_container.basket,
              &body.product_id,
              &user_id,
            )
          })
          .await;

          match decrement_product_count_result {
            Ok(_response) => HttpResponse::Ok().finish(),
            Err(_e) => HttpResponse::InternalServerError().finish(),
          }
        }
      }
      Err(e) => {
        println!("Error while stringifying user_id header, {:?}", e);
        HttpResponse::NotFound().body("user id is not type of string")
      }
    },
    None => HttpResponse::NotFound().body("user does not exist"),
  }
}
