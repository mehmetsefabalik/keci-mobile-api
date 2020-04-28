use crate::action;
use crate::action::user::{LoginResult, UserCreateResult};
use actix_web::{http, web, HttpRequest, HttpResponse, Responder};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug, Clone)]
pub struct CreateUserBody {
  pub phone: String,
  password: String,
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
  let user_id;
  match request.headers().get("user_id") {
    Some(user_id_header) => {
      user_id = Some(user_id_header.to_str().unwrap().to_string());
    }
    None => user_id = None,
  }

  let result = web::block(move || {
    action::user::create(
      app_data.service_container.user.clone(),
      body.phone.clone(),
      body.password.clone(),
      user_id,
    )
  })
  .await;

  match result {
    Ok(response) => match response {
      UserCreateResult::UserCreated(cookie) => HttpResponse::Ok()
        .header(
          "Set-Cookie",
          http::header::HeaderValue::from_str(&cookie).unwrap(),
        )
        .finish(),
      UserCreateResult::GuestUserNotRegistered => HttpResponse::BadRequest().finish(),
      UserCreateResult::UserAlreadyExists => HttpResponse::BadRequest().finish(),
    },
    Err(e) => {
      println!("Error while creating user: {:?}", e);
      HttpResponse::InternalServerError().finish()
    }
  }
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
  let user_id;
  let user_type;
  match request.headers().get("user_id") {
    Some(user_id_header) => {
      user_id = Some(user_id_header.to_str().unwrap().to_string());
      user_type = Some(
        request
          .headers()
          .get("user_type")
          .unwrap()
          .to_str()
          .unwrap()
          .to_string(),
      );
    }
    None => {
      user_id = None;
      user_type = None;
    }
  }
  let result = web::block(move || {
    action::user::login(
      app_data.service_container.user.clone(),
      body.phone.clone(),
      body.password.clone(),
      user_id,
      user_type,
    )
  })
  .await;

  match result {
    Ok(login_result) => match login_result {
      LoginResult::Verified(cookie) => HttpResponse::Ok()
        .header(
          "Set-Cookie",
          http::header::HeaderValue::from_str(&cookie).unwrap(),
        )
        .finish(),
      LoginResult::NotVerified => HttpResponse::BadRequest().finish(),
      LoginResult::UserNotExists => HttpResponse::BadRequest().finish(),
    },
    Err(e) => {
      println!("Error while creating user: {:?}", e);
      HttpResponse::InternalServerError().finish()
    }
  }
}
