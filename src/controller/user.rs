use actix_web::{error, http, web, HttpRequest, HttpResponse, Responder};
use bcrypt::{hash, verify};
use bson::{from_bson, to_bson};
use jsonwebtoken::{encode, EncodingKey, Header};
use serde::{Deserialize, Serialize};
use crate::service::user;
use crate::model::user::User;

#[derive(Deserialize, Debug, Clone)]
pub struct CreateUserBody {
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
  body: web::Json<CreateUserBody>,
) -> impl Responder {
  let phone = body.phone.clone();
  web::block(move || user::get(app_data.user_collection.clone(), &phone))
    .await
    .map(|(get_user_result, user_collection)| match get_user_result {
      Some(_) => HttpResponse::BadRequest().json(UserAlreadyRegisteredResponse {
        user_already_registered: true,
      }),
      None => {
        // TODO: CPU intensive task, wrap with web::block
        let hashed = hash(&body.password, 4).unwrap();

        match request.headers().get("user_id") {
          Some(user_id_header) => {
            // user
            match user_id_header.to_str() {
              Ok(user_id_str) => {
                let user_id = String::from(user_id_str);
                // TODO: call with web::block
                match user::register(
                  user_collection,
                  &user_id,
                  &body.phone,
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
            let user = User::new(&body.phone, &hashed);
            // TODO: call with web::block
            match user::create(user_collection, &user) {
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
    })
    .map_err(|err| match err {
      error::BlockingError::Error(error) => {
        println!("error, {:?}", error);
        HttpResponse::BadRequest().body("Error while searching user")
      }
      error::BlockingError::Canceled => HttpResponse::new(http::StatusCode::INTERNAL_SERVER_ERROR),
    })
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserNotExistResponse {
  pub user_not_exist: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WrongPasswordResponse {
  pub wrong_password: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AlreadyGuestResponse {
  already_guest: bool,
  guest_id: String,
  registered_id: String,
}

#[derive(Deserialize, Debug, Clone)]
struct UserJson {
  _id: bson::oid::ObjectId,
  phone: String,
  password: String,
}

pub async fn login(
  request: HttpRequest,
  app_data: web::Data<crate::AppState>,
  body: web::Json<CreateUserBody>,
) -> impl Responder {
  let phone = body.phone.clone();
  web::block(move || user::get(app_data.user_collection.clone(), &phone))
    .await
    .map(
      |(get_user_result, _user_collection)| match get_user_result {
        Some(user_document) => {
          let user_bson = to_bson(&user_document).unwrap();
          let user = from_bson::<UserJson>(user_bson).unwrap();
          let verify_result = verify(&body.password, &user.password);
          match verify_result {
            Ok(verified) => {
              if verified == true {
                match request.headers().get("user_id") {
                  Some(user_id_header) => {
                    // user
                    match user_id_header.to_str() {
                      Ok(guest_id_str) => {
                        let user_type = request
                          .headers()
                          .get("user_type")
                          .unwrap()
                          .to_str()
                          .unwrap();
                        if user_type == "guest" {
                          // logging in while s/he is guest
                          let claims = Claims {
                            sub: user._id.to_string(),
                            user_type: String::from("registered"),
                          };
                          let token = encode(
                            &Header::default(),
                            &claims,
                            &EncodingKey::from_secret(dotenv!("JWT_SECRET").as_ref()),
                          )
                          .unwrap();
                          let cookie = format!("access_token={}; path=/", token);
                          HttpResponse::Ok()
                            .header(
                              "Set-Cookie",
                              http::header::HeaderValue::from_str(&cookie).unwrap(),
                            )
                            .json(AlreadyGuestResponse {
                              already_guest: true,
                              guest_id: String::from(guest_id_str),
                              registered_id: user._id.to_string()
                            })
                        } else {
                          // logging in while s/he is registered
                          let claims = Claims {
                            sub: user._id.to_string(),
                            user_type: String::from("registered"),
                          };
                          let token = encode(
                            &Header::default(),
                            &claims,
                            &EncodingKey::from_secret(dotenv!("JWT_SECRET").as_ref()),
                          )
                          .unwrap();
                          let cookie = format!("access_token={}; path=/", token);
                          HttpResponse::Ok()
                            .header(
                              "Set-Cookie",
                              http::header::HeaderValue::from_str(&cookie).unwrap(),
                            )
                            .json(user._id.to_string())
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
                    // not a user before
                    let claims = Claims {
                      sub: user._id.to_string(),
                      user_type: String::from("registered"),
                    };
                    let token = encode(
                      &Header::default(),
                      &claims,
                      &EncodingKey::from_secret(dotenv!("JWT_SECRET").as_ref()),
                    )
                    .unwrap();
                    let cookie = format!("access_token={}; path=/", token);
                    HttpResponse::Ok()
                      .header(
                        "Set-Cookie",
                        http::header::HeaderValue::from_str(&cookie).unwrap(),
                      )
                      .json(user._id.to_string())
                  }
                }
              } else {
                HttpResponse::BadRequest().json(WrongPasswordResponse {
                  wrong_password: true,
                })
              }
            }
            Err(_e) => HttpResponse::new(http::StatusCode::INTERNAL_SERVER_ERROR),
          }
        }
        None => HttpResponse::BadRequest().json(UserNotExistResponse {
          user_not_exist: true,
        }),
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
