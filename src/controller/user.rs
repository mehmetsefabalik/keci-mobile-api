use actix_web::{error, http, web, HttpResponse, Responder};
use bcrypt::{hash, DEFAULT_COST};
use jsonwebtoken::{encode, EncodingKey, Header};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Clone)]
pub struct CreateUser {
  pub phone: String,
  password: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
  sub: String,
}

pub async fn create(
  app_data: web::Data<crate::AppState>,
  body: web::Json<CreateUser>,
) -> impl Responder {
  web::block(move || crate::service::user::get(app_data.user_collection.clone(), body))
    .await
    .map(
      |(get_user_result, user_collection, user)| match get_user_result {
        Some(_) => HttpResponse::BadRequest().json({}),
        None => {
          let hashed = hash(&user.password, DEFAULT_COST).unwrap();
          match crate::service::user::create(user_collection, &user.phone, &hashed) {
            Ok(create_user_result) => match create_user_result.inserted_id {
              bson::Bson::ObjectId(id) => {
                let claims = Claims {
                  sub: id.to_string(),
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
