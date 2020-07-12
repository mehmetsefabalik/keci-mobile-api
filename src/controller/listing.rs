use actix_web::{web, HttpResponse, Responder};
use serde::Deserialize;

pub async fn get(app_data: web::Data<crate::AppState>) -> impl Responder {
  let result = web::block(move || app_data.service_container.listing.get()).await;
  match result {
    Ok(result) => HttpResponse::Ok().json(result),
    Err(e) => {
      println!("Error while getting listings, {:?}", e);
      HttpResponse::InternalServerError().finish()
    }
  }
}

#[derive(Deserialize)]
pub struct GetForSellerPath {
  seller: String,
}

pub async fn get_for_seller(
  app_data: web::Data<crate::AppState>,
  path: web::Path<GetForSellerPath>,
) -> impl Responder {
  let result = web::block(move || {
    app_data
      .service_container
      .listing
      .get_for_seller(&path.seller)
  })
  .await;
  match result {
    Ok(result) => HttpResponse::Ok().json(result),
    Err(e) => {
      println!("Error while getting listings, {:?}", e);
      HttpResponse::InternalServerError().finish()
    }
  }
}
