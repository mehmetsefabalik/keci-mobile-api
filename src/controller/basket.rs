use crate::action::basket::{add_to_basket, decrement_product_count};
use crate::model::basket::{Basket, BasketItem};
use crate::service::user;
use actix_web::{http, web, HttpRequest, HttpResponse, Responder};
use bson::oid::ObjectId;
use jsonwebtoken::{encode, EncodingKey, Header};
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
  let product_id = body.product_id.clone();
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
      // TODO: wrap with web::block
      match user::create_anon(app_data.user_collection.clone()) {
        Ok(user_result) => match user_result.inserted_id {
          bson::Bson::ObjectId(id) => {
            let claims = crate::controller::user::Claims {
              sub: id.to_string(),
              user_type: String::from("guest"),
            };
            let token = encode(
              &Header::default(),
              &claims,
              &EncodingKey::from_secret(dotenv!("JWT_SECRET").as_ref()),
            )
            .unwrap();
            let cookie = format!("access_token={}", token);

            let basket_item = BasketItem::new(
              ObjectId::with_string(&product_id).expect("Invalid ObjectId string"),
              1,
            );
            let basket = Basket::new(
              ObjectId::with_string(&id.to_string()).expect("Invalid ObjectId string"),
              vec![basket_item],
              true,
            );

            let create_basket_result =
              web::block(move || app_data.service_container.basket.create(&basket)).await;

            match create_basket_result {
              Ok(basket_result) => HttpResponse::Ok()
                .header(
                  "Set-Cookie",
                  http::header::HeaderValue::from_str(&cookie).unwrap(),
                )
                .json(basket_result.inserted_id),
              Err(e) => {
                println!("Error while creating basket for anon user, {:?}", e);
                HttpResponse::InternalServerError().finish()
              }
            }
          }
          _ => {
            println!("Error: inserted anon user id is not ObjectId");
            HttpResponse::InternalServerError().finish()
          }
        },
        Err(e) => {
          println!("Error while creating anon user: {:?}", e);
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
