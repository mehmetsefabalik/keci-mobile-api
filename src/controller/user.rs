use actix_web::{error, http, web, HttpResponse, Responder};
use serde::Deserialize;
use bcrypt::{DEFAULT_COST, hash};

#[derive(Deserialize, Clone)]
pub struct CreateUser {
    pub phone: String,
    password: String,
}

pub async fn create(app_data: web::Data<crate::AppState>, body: web::Json<CreateUser>) -> impl Responder {
  web::block(move || crate::service::user::get(app_data.user_collection.clone(), body))
    .await
    .map(|(get_user_result, user_collection, user)| {
      match get_user_result {
        Some(_) => {
          HttpResponse::BadRequest().json({})
        },
        None => {
          let hashed = hash(&user.password, DEFAULT_COST).unwrap();
          match crate::service::user::create(user_collection, &user.phone, &hashed) {
            Ok(create_user_result) => {
              HttpResponse::Ok().json(create_user_result.inserted_id)
            },
            Err(_) => HttpResponse::new(http::StatusCode::INTERNAL_SERVER_ERROR)
          }
        }
      }
    })
    .map_err(|err| match err {
      error::BlockingError::Error(error) => {
        println!("error, {:?}", error);
        HttpResponse::BadRequest().body("Error while searching user")
      },
      error::BlockingError::Canceled => HttpResponse::new(http::StatusCode::INTERNAL_SERVER_ERROR),
    })
}
