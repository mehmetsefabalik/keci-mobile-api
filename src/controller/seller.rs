use actix_web::{web, HttpResponse, Responder};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct GetPath {
  name: String,
}

pub async fn get(app_data: web::Data<crate::AppState>, path: web::Path<GetPath>) -> impl Responder {
  let result = web::block(move || app_data.service_container.seller.get(&path.name)).await;
  match result {
    Ok(result) => HttpResponse::Ok().json(result),
    Err(e) => {
      println!("Error while getting seller, {:?}", e);
      HttpResponse::InternalServerError().finish()
    }
  }
}
