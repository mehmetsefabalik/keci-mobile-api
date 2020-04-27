use actix_web::{web, HttpResponse, Responder};

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
