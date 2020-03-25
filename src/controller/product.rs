use actix_web::{error, http, web, HttpResponse, Responder};

pub async fn get(app_data: web::Data<crate::AppState>) -> impl Responder {
  web::block(move || crate::service::product::get(app_data.product_collection.clone()))
    .await
    .map(|result| HttpResponse::Ok().json(result))
    .map_err(|err| match err {
      error::BlockingError::Error(result) => HttpResponse::NotFound().body(result),
      error::BlockingError::Canceled => HttpResponse::new(http::StatusCode::INTERNAL_SERVER_ERROR),
    })
}
