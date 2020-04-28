use crate::model::basket::{Basket, BasketItem};
use crate::service::basket::BasketService;
use crate::service::user::UserService;
use crate::model::user::Claims;
use actix_web::http::header::HeaderValue;
use bcrypt::hash;
use bson::oid::ObjectId;
use jsonwebtoken::{encode, EncodingKey, Header};

pub fn create_anon_with_basket(
  user_service: UserService,
  basket_service: BasketService,
  product_id: String,
) -> Result<String, String> {
  match user_service.create_anon() {
    Ok(user_result) => match user_result.inserted_id {
      bson::Bson::ObjectId(id) => {
        let claims = crate::controller::user::Claims {
          sub: id.to_string(),
          user_type: String::from("guest"),
        };
        let token = encode(
          &Header::default(),
          &claims,
          &EncodingKey::from_secret(dotenv!("JWT_SECRET").as_ref()),
        )
        .unwrap();
        let cookie = format!("access_token={}", token);

        let basket_item = BasketItem::new(
          ObjectId::with_string(&product_id).expect("Invalid ObjectId string"),
          1,
        );
        let basket = Basket::new(
          ObjectId::with_string(&id.to_string()).expect("Invalid ObjectId string"),
          vec![basket_item],
          true,
        );

        match basket_service.create(&basket) {
          Ok(_basket_result) => Ok(cookie),
          Err(_e) => Err("Error while creating basket for anon user, {:?}".to_string()),
        }
      }
      _ => Err("Error: inserted anon user id is not ObjectId".to_string()),
    },
    Err(e) => Err("Error while creating anon user: {:?}".to_string()),
  }
}

pub enum UserCreateResult {
  UserAlreadyExists,
  UserCreated(String),
  GuestUserNotRegistered,

}

pub fn create(
  user_service: UserService,
  phone: String,
  password: String,
  user_id_header: Option<&HeaderValue>,
) -> Result<UserCreateResult, String> {
  match user_service.get(&phone) {
    Ok(user_result) => match user_result {
      Some(user) => Ok(UserCreateResult::UserAlreadyExists),
      None => {
        let hashed = hash(&password, 4).unwrap();

        match user_id_header {
          Some(user_id_header_value) => match user_id_header_value.to_str() {
            Ok(user_id) => match user_service.register(&user_id, &phone, &password) {
              Ok(user_result) => {
                if user_result.modified_count == 1 {
                  let claims = Claims {
                    sub: user_id.to_string(),
                    user_type: String::from("registered"),
                  };
                  let token = encode(
                    &Header::default(),
                    &claims,
                    &EncodingKey::from_secret(dotenv!("JWT_SECRET").as_ref()),
                  )
                  .unwrap();
                  let cookie = format!("access_token={}", token);
                  Ok(UserCreateResult::UserCreated(cookie))
                } else {
                  Ok(UserCreateResult::GuestUserNotRegistered)
                }
              }
              Err(_e) => Err("Error while registering user".to_string()),
            },
            Err(e) => Err("Error while parsing user_id header value".to_string()),
          },
          None => {
            
          }
        }
      }
    },
    Err(_e) => Err("Error while getting user".to_string()),
  }
}
