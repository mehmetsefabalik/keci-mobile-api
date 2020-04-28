use crate::model::basket::{Basket, BasketItem};
use crate::model::user::{Claims, User};
use crate::service::basket::BasketService;
use crate::service::user::UserService;
use crate::traits::service::Creator;
use bcrypt::hash;
use bcrypt::verify;
use bson::oid::ObjectId;
use bson::{from_bson, to_bson};
use jsonwebtoken::{encode, EncodingKey, Header};
use serde::Deserialize;

pub fn create_anon_with_basket(
  user_service: UserService,
  basket_service: BasketService,
  product_id: String,
) -> Result<String, String> {
  match user_service.create_anon() {
    Ok(user_result) => match user_result.inserted_id {
      bson::Bson::ObjectId(id) => {
        let token = get_guest_user_token(id.to_string());
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
    Err(_e) => Err("Error while creating anon user: {:?}".to_string()),
  }
}

fn get_registered_user_token(id: String) -> String {
  let claims = Claims {
    sub: id,
    user_type: String::from("registered"),
  };
  let token = encode(
    &Header::default(),
    &claims,
    &EncodingKey::from_secret(dotenv!("JWT_SECRET").as_ref()),
  )
  .unwrap();
  token
}

fn get_guest_user_token(id: String) -> String {
  let claims = Claims {
    sub: id,
    user_type: String::from("guest"),
  };
  let token = encode(
    &Header::default(),
    &claims,
    &EncodingKey::from_secret(dotenv!("JWT_SECRET").as_ref()),
  )
  .unwrap();
  token
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
  user_id_option: Option<String>,
) -> Result<UserCreateResult, String> {
  match user_service.get(&phone) {
    Ok(user_result) => match user_result {
      Some(_user) => Ok(UserCreateResult::UserAlreadyExists),
      None => match user_id_option {
        Some(user_id) => match user_service.register(&user_id, &phone, &password) {
          Ok(user_result) => {
            if user_result.modified_count == 1 {
              let token = get_registered_user_token(user_id.to_string());
              let cookie = format!("access_token={}", token);
              Ok(UserCreateResult::UserCreated(cookie))
            } else {
              Ok(UserCreateResult::GuestUserNotRegistered)
            }
          }
          Err(_e) => Err("Error while registering user".to_string()),
        },
        None => {
          let hashed = hash(&password, 4).unwrap();
          let user = User::new(&phone, &hashed);
          match user_service.create(&user) {
            Ok(user_result) => match user_result.inserted_id {
              bson::Bson::ObjectId(id) => {
                let token = get_registered_user_token(id.to_string());
                let cookie = format!("access_token={}", token);
                Ok(UserCreateResult::UserCreated(cookie))
              }
              _ => Err("Error: inserted user id is not type of ObjectId".to_string()),
            },
            Err(_e) => Err("Error while creating user".to_string()),
          }
        }
      },
    },
    Err(_e) => Err("Error while getting user".to_string()),
  }
}

pub enum LoginResult {
  Verified(String),
  UserNotExists,
  NotVerified,
}

#[derive(Deserialize, Debug, Clone)]
struct UserJson {
  _id: bson::oid::ObjectId,
  phone: String,
  password: String,
}

pub fn login(
  user_service: UserService,
  phone: String,
  password: String,
  user_id_option: Option<String>,
  user_type_option: Option<String>,
) -> Result<LoginResult, String> {
  match user_service.get(&phone) {
    Ok(user_option) => match user_option {
      Some(user_document) => {
        let user_bson = to_bson(&user_document).unwrap();
        let user = from_bson::<UserJson>(user_bson).unwrap();
        let verify_result = verify(&password, &user.password);

        match verify_result {
          Ok(verified) => {
            if verified == true {
              match user_id_option {
                Some(_guest_id) => {
                  if user_type_option.unwrap() == "guest" {
                    // TODO: merge basket
                  }
                  let token = get_registered_user_token(user._id.to_string());
                  let cookie = format!("access_token={}; path=/", token);
                  Ok(LoginResult::Verified(cookie))
                }
                None => {
                  let token = get_registered_user_token(user._id.to_string());
                  let cookie = format!("access_token={}; path=/", token);
                  Ok(LoginResult::Verified(cookie))
                }
              }
            } else {
              Ok(LoginResult::NotVerified)
            }
          }
          Err(_e) => Err("Error while verifying".to_string()),
        }
      }
      None => Ok(LoginResult::UserNotExists),
    },
    Err(_e) => Err("Error while getting user".to_string()),
  }
}
