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
                match crate::service::basket::increment_product_count(
                  collection,
                  product_id,
                  user_id,
                ) {
                  Ok((update, collection, product_id, user_id)) => match update {
                    Some(doc) => {
                      HttpResponse::Ok().json(doc)
                    },
                    None => {
                      // product is not present in basket
                      match crate::service::basket::add_item(collection, &product_id, &user_id) {
                        Ok(_update) => {
                          HttpResponse::Ok().json(active_basket)
                        },
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
      Ok(HttpResponse::Ok().json("anon"))
    }
  }
}
