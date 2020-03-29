use actix_web::{error, http, web, HttpRequest, HttpResponse, Responder};
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
          web::block(move || {
            crate::service::basket::get_active(app_data.basket_collection.clone(), user_id)
          })
          .await
          .map(|(active_basket_option, collection, user_id)| {
            match active_basket_option {
              Some(active_basket) => {
                // user has active basket

                // check whether product is already in active basket
                // TODO: wrap with web block
                match crate::service::basket::increment_product_count(
                  collection, product_id, user_id,
                ) {
                  Ok((update, collection, product_id, user_id)) => match update {
                    Some(doc) => HttpResponse::Ok().json(doc),
                    None => {
                      // product is not present in basket
                      // TODO: wrap with web::block
                      match crate::service::basket::add_item(collection, &product_id, &user_id) {
                        Ok(_update) => HttpResponse::Ok().json(active_basket),
                        Err(e) => {
                          println!("Error while adding item to basket, {:?}", e);
                          HttpResponse::new(http::StatusCode::INTERNAL_SERVER_ERROR)
                        }
                      }
                    }
                  },
                  Err(e) => {
                    println!("Error while incrementing product count,  {:?}", e);
                    HttpResponse::new(http::StatusCode::INTERNAL_SERVER_ERROR)
                  }
                }
              }
              None => {
                // user does not have active basket
                // TODO: wrap with web::block
                let basket_result =
                  crate::service::basket::create(collection, &product_id, &user_id);
                match basket_result {
                  Ok(basket) => {
                    let response = Response {
                      id: basket.inserted_id,
                      message: String::from("created active basket for user"),
                    };
                    HttpResponse::Ok().json(response)
                  }
                  Err(_e) => HttpResponse::new(http::StatusCode::INTERNAL_SERVER_ERROR),
                }
              }
            }
          })
          .map_err(|err| match err {
            error::BlockingError::Error(error) => {
              println!("error, {:?}", error);
              HttpResponse::BadRequest().body("Error while getting active basket")
            }
            error::BlockingError::Canceled => {
              HttpResponse::new(http::StatusCode::INTERNAL_SERVER_ERROR)
            }
          })
        }
        Err(e) => {
          println!(
            "Error while getting string of user_id header value, {:?}",
            e
          );
          Ok(HttpResponse::new(http::StatusCode::INTERNAL_SERVER_ERROR))
        }
      }
    }
    None => {
      // anon
      // TODO: wrap with web::block
      match crate::service::user::create_anon(app_data.user_collection.clone()) {
        Ok(user_result) => match user_result.inserted_id {
          bson::Bson::ObjectId(id) => {
            let claims = crate::controller::user::Claims {
              sub: id.to_string(),
              user_type: String::from("registered"),
            };
            let token = encode(
              &Header::default(),
              &claims,
              &EncodingKey::from_secret(dotenv!("JWT_SECRET").as_ref()),
            )
            .unwrap();
            let cookie = format!("access_token={}", token);
            // TODO: wrap with web::block
            match crate::service::basket::create(
              app_data.basket_collection.clone(),
              &product_id,
              &id.to_string(),
            ) {
              Ok(basket_result) => Ok(
                HttpResponse::Ok()
                  .header(
                    "Set-Cookie",
                    http::header::HeaderValue::from_str(&cookie).unwrap(),
                  )
                  .json(basket_result.inserted_id),
              ),
              Err(e) => {
                println!("Error while creating basket for anon user, {:?}", e);
                Ok(HttpResponse::new(http::StatusCode::INTERNAL_SERVER_ERROR))
              }
            }
          }
          _ => Ok(HttpResponse::new(http::StatusCode::INTERNAL_SERVER_ERROR)),
        },
        Err(e) => {
          println!("Error while creating anon user: {:?}", e);
          Ok(HttpResponse::new(http::StatusCode::INTERNAL_SERVER_ERROR))
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
        web::block(move || {
          crate::service::basket::get_active(
            app_data.basket_collection.clone(),
            String::from(user_id),
          )
        })
        .await
        .map(|(option, _collection, _user_id)| match option {
          Some(document) => HttpResponse::Ok().json(document),
          None => HttpResponse::NotFound().body("active basket not exists"),
        })
        .map_err(|err| match err {
          error::BlockingError::Error(error) => {
            println!("error, {:?}", error);
            HttpResponse::BadRequest().body("Error while getting active basket")
          }
          error::BlockingError::Canceled => {
            HttpResponse::new(http::StatusCode::INTERNAL_SERVER_ERROR)
          }
        })
      }
      Err(e) => {
        println!("Error while stringifying user_id header, {:?}", e);
        Ok(HttpResponse::NotFound().body("user is not type of string"))
      }
    },
    None => Ok(HttpResponse::NotFound().body("user does not exist")),
  }
}
