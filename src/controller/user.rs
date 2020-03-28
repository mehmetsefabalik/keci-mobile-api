use actix_web::{error, http, web, HttpResponse, Responder, HttpRequest};

pub async fn get(req: HttpRequest, app_data: web::Data<crate::AppState>) -> impl Responder {
  web::block(move || crate::service::listing::get(app_data.listing_collection.clone()))
    .await
    .map(|result| HttpResponse::Ok().json(result))
    .map_err(|err| match err {
      error::BlockingError::Error(result) => HttpResponse::NotFound().body(result),
      error::BlockingError::Canceled => HttpResponse::new(http::StatusCode::INTERNAL_SERVER_ERROR),
    })
}
