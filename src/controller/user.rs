use actix_web::{error, http, web, HttpRequest, HttpResponse, Responder};
use bcrypt::hash;
use jsonwebtoken::{encode, EncodingKey, Header};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Clone)]
pub struct CreateUser {
  pub phone: String,
  password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
  pub sub: String,
  pub user_type: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserAlreadyRegisteredResponse {
  pub user_already_registered: bool,
}

pub async fn create(
  request: HttpRequest,
  app_data: web::Data<crate::AppState>,
  body: web::Json<CreateUser>,
) -> impl Responder {
  web::block(move || crate::service::user::get(app_data.user_collection.clone(), body))
    .await
    .map(
      |(get_user_result, user_collection, user)| match get_user_result {
        Some(_) => HttpResponse::BadRequest().json(UserAlreadyRegisteredResponse {
          user_already_registered: true,
        }),
        None => {
          // TODO: CPU intensive task, wrap with web::block
          let hashed = hash(&user.password, 4).unwrap();

          match request.headers().get("user_id") {
            Some(user_id_header) => {
              // user
              match user_id_header.to_str() {
                Ok(user_id_str) => {
                  let user_id = String::from(user_id_str);
                  // TODO: call with web::block
                  match crate::service::user::register(
                    user_collection,
                    &user_id,
                    &user.phone,
                    &hashed,
                  ) {
                    Ok(register_user_result) => {
                      if register_user_result.modified_count == 1 {
                        let claims = Claims {
                          sub: user_id.clone(),
                          user_type: String::from("registered"),
                        };
                        let token = encode(
                          &Header::default(),
                          &claims,
                          &EncodingKey::from_secret(dotenv!("JWT_SECRET").as_ref()),
                        )
                        .unwrap();
                        let cookie = format!("access_token={}", token);
                        HttpResponse::Ok()
                          .header(
                            "Set-Cookie",
                            http::header::HeaderValue::from_str(&cookie).unwrap(),
                          )
                          .json(user_id)
                      } else {
                        HttpResponse::BadRequest().json({})
                      }
                    }
                    Err(_) => HttpResponse::new(http::StatusCode::INTERNAL_SERVER_ERROR),
                  }
                }
                Err(e) => {
                  println!(
                    "Error while getting string of user_id header value, {:?}",
                    e
                  );
                  HttpResponse::new(http::StatusCode::INTERNAL_SERVER_ERROR)
                }
              }
            }
            None => {
              // anon
              // TODO: call with web::block
              match crate::service::user::create(user_collection, &user.phone, &hashed) {
                Ok(create_user_result) => match create_user_result.inserted_id {
                  bson::Bson::ObjectId(id) => {
                    let claims = Claims {
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
                    HttpResponse::Ok()
                      .header(
                        "Set-Cookie",
                        http::header::HeaderValue::from_str(&cookie).unwrap(),
                      )
                      .json(id)
                  }
                  _ => HttpResponse::new(http::StatusCode::INTERNAL_SERVER_ERROR),
                },
                Err(_) => HttpResponse::new(http::StatusCode::INTERNAL_SERVER_ERROR),
              }
            }
          }
        }
      },
    )
    .map_err(|err| match err {
      error::BlockingError::Error(error) => {
        println!("error, {:?}", error);
        HttpResponse::BadRequest().body("Error while searching user")
      }
      error::BlockingError::Canceled => HttpResponse::new(http::StatusCode::INTERNAL_SERVER_ERROR),
    })
}
