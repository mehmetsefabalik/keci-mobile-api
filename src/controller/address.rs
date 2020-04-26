use crate::model::address::Address;
use crate::service::address;
use actix_web::{web, HttpRequest, HttpResponse, Responder};
use bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug, Clone)]
pub struct CreateAddressBody {
  name: String,
  surname: String,
  title: String,
  text: String,
  district_id: i32,
  neighborhood_id: i32,
}

#[derive(Deserialize, Debug, Serialize)]
pub struct CreatedResponse {
  id: bson::Bson,
  message: String,
}

pub async fn create(
  request: HttpRequest,
  app_data: web::Data<crate::AppState>,
  body: web::Json<CreateAddressBody>,
) -> impl Responder {
  match request.headers().get("user_id") {
    Some(user_id_header) => match user_id_header.to_str() {
      Ok(user_id) => {
        let address = Address::new(
          ObjectId::with_string(user_id).expect("Invalid ObjectId string"),
          &body.name,
          &body.surname,
          &body.title,
          &body.text,
          body.district_id,
          body.neighborhood_id,
        );
        let create_address_result =
          web::block(move || address::create(&app_data.address_collection, &address)).await;
        match create_address_result {
          Ok(response) => {
            let response = CreatedResponse {
              id: response.inserted_id,
              message: String::from("Address has been successfully created"),
            };
            HttpResponse::Created().json(response)
          }
          Err(e) => {
            println!("Error while creating address, {:?}", e);
            HttpResponse::InternalServerError().finish()
          }
        }
      }
      Err(_e) => {
        println!(
          "Error while stringifying user_id header, {:?}",
          user_id_header
        );
        HttpResponse::BadRequest().finish()
      }
    },
    None => HttpResponse::Unauthorized().finish(),
  }
}

pub async fn get_all(request: HttpRequest, app_data: web::Data<crate::AppState>) -> impl Responder {
  match request.headers().get("user_id") {
    Some(user_id_header) => match user_id_header.to_str() {
      Ok(user_id_str) => {
        let user_id = String::from(user_id_str);
        let create_address_result = web::block(move || {
          address::get_all(&app_data.address_collection, &user_id)
        })
        .await;
        match create_address_result {
          Ok(response) => HttpResponse::Ok().json(response),
          Err(e) => {
            println!("Error while creating address, {:?}", e);
            HttpResponse::InternalServerError().finish()
          }
        }
      }
      Err(_e) => {
        println!(
          "Error while stringifying user_id header, {:?}",
          user_id_header
        );
        HttpResponse::BadRequest().finish()
      }
    },
    None => HttpResponse::Unauthorized().finish(),
  }
}

#[derive(Deserialize)]
pub struct UpdatePath {
  address_id: String,
}

pub async fn update(
  request: HttpRequest,
  app_data: web::Data<crate::AppState>,
  body: web::Json<CreateAddressBody>,
  path: web::Path<UpdatePath>,
) -> impl Responder {
  match request.headers().get("user_id") {
    Some(user_id_header) => match user_id_header.to_str() {
      Ok(user_id) => {
        let address = Address::new(
          ObjectId::with_string(user_id).expect("Invalid ObjectId string"),
          &body.name,
          &body.surname,
          &body.title,
          &body.text,
          body.district_id,
          body.neighborhood_id,
        );
        let address_result = web::block(move || {
          address::update(&app_data.address_collection, &path.address_id, &address)
        })
        .await;
        match address_result {
          Ok(_response) => HttpResponse::Ok().finish(),
          Err(e) => {
            println!("Error while creating address, {:?}", e);
            HttpResponse::InternalServerError().finish()
          }
        }
      }
      Err(_e) => {
        println!(
          "Error while stringifying user_id header, {:?}",
          user_id_header
        );
        HttpResponse::BadRequest().finish()
      }
    },
    None => HttpResponse::Unauthorized().finish(),
  }
}
