use crate::model::basket::{Basket, BasketItem};
use crate::service::basket::BasketService;
use bson::oid::ObjectId;
use mongodb::error::Error;

pub fn add_to_basket(
  basket_service: BasketService,
  user_id: String,
  product_id: String,
  seller_id: String,
  listing_id: String,
) -> Result<String, String> {
  match basket_service.get_active(&user_id) {
    Ok(active_basket_option) => match active_basket_option {
      Some(active_basket) => user_has_active_basket(
        &basket_service,
        &active_basket,
        &product_id,
        &seller_id,
        &listing_id,
        &user_id,
      ),
      None => user_does_not_have_active_basket(
        &basket_service,
        &product_id,
        &seller_id,
        &listing_id,
        &user_id,
      ),
    },
    Err(e) => {
      println!("Error while getting active basket, {:?}", e);
      Err("Error while getting active basket".to_string())
    }
  }
}

fn user_has_active_basket(
  basket_service: &BasketService,
  _active_basket: &bson::ordered::OrderedDocument,
  product_id: &str,
  seller_id: &str,
  listing_id: &str,
  user_id: &str,
) -> Result<String, String> {
  match basket_service.update_product_count(product_id, seller_id, user_id, 1) {
    Ok(update) => match update {
      Some(_doc) => Ok("Product count is incremented successfully".to_string()),
      None => {
        // product is not present in basket
        match basket_service.add_item(product_id, seller_id, listing_id, user_id) {
          Ok(_r) => Ok("Product is added to basket successfully".to_string()),
          Err(_e) => Err("Error while adding item".to_string()),
        }
      }
    },
    Err(e) => {
      println!("Error while incrementing product count,  {:?}", e);
      Err("Error while incrementing product count".to_string())
    }
  }
}

fn user_does_not_have_active_basket(
  basket_service: &BasketService,
  product_id: &str,
  seller_id: &str,
  listing_id: &str,
  user_id: &str,
) -> Result<String, String> {
  let basket_item = BasketItem::new(
    ObjectId::with_string(product_id).expect("product_id: Invalid ObjectId string"),
    ObjectId::with_string(seller_id).expect("seller_id: Invalid ObjectId string"),
    ObjectId::with_string(listing_id).expect("listing_id: Invalid ObjectId string"),
    1,
  );
  let basket = Basket::new(
    ObjectId::with_string(user_id).expect("Invalid ObjectId string"),
    vec![basket_item],
    true,
  );

  match basket_service.create(&basket) {
    Ok(_r) => Ok("Basket is created successfully".to_string()),
    Err(_e) => Err("Basket is not created successfully".to_string()),
  }
}

pub fn decrement_product_count(
  basket_service: &BasketService,
  product_id: &str,
  seller_id: &str,
  user_id: &str,
) -> Result<String, Error> {
  match basket_service.get_product_with_count_one(
    product_id.to_string(),
    seller_id.to_string(),
    user_id.to_string(),
  ) {
    Ok(option) => match option {
      Some(_document) => match basket_service.remove_product(product_id, seller_id, user_id) {
        Ok(_update) => Ok("Product is removed successfuly".to_string()),
        Err(e) => {
          println!("product can not be removed: {}", e);
          Err(e)
        }
      },
      None => match basket_service.update_product_count(product_id, seller_id, user_id, -1) {
        Ok(_document) => Ok("Product count is decremented successfuly".to_string()),
        Err(e) => {
          println!("product can not be decremented: {}", e);
          Err(e)
        }
      },
    },
    Err(e) => {
      println!("Error while getting product, {:?}", e);
      Err(e)
    }
  }
}
