use crate::service::address::AddressService;
use crate::service::basket::BasketService;
use crate::service::order::OrderService;
use crate::traits::service::{Finder, Creator};
use crate::model::order::Order;

pub enum CreateOrderResponse {
  OrderCreated,
  ActiveBasketNotFound,
  AddressNotFound,
  BasketToDeleteNotFound,
}

pub fn create_order(
  order_service: OrderService,
  basket_service: BasketService,
  address_service: AddressService,
  user_id: String,
  address_id: String,
) -> Result<CreateOrderResponse, String> {
  let address_result = address_service.find(&address_id);

  match address_result {
    Ok(address_option) => match address_option {
      Some(address) => {
        let basket_result = basket_service.get_active(&user_id);

        match basket_result {
          Ok(basket_option) => match basket_option {
            Some(basket) => {
              let order = Order::new(
                bson::oid::ObjectId::with_string(&user_id).expect("Invalid ObjectId string"),
                basket,
                address,
                0,
                );

              let order_result = order_service.create(&order);

              match order_result {
                Ok(_order) => {
                  let delete_basket_result = basket_service.delete(&user_id);

                  match delete_basket_result {
                    Ok(delete_basket_option) => {
                      match delete_basket_option {
                        Some(_basket) => Ok(CreateOrderResponse::OrderCreated),
                        None => Ok(CreateOrderResponse::BasketToDeleteNotFound)
                      }
                    },
                    Err(_e) => Err("Error while deleting basket".to_string())
                  }
                },
                Err(_e) => Err("Error while creating order".to_string())
              }
            }
            None => Ok(CreateOrderResponse::ActiveBasketNotFound),
          },
          Err(_e) => Err("Error while getting active basket".to_string()),
        }
      }
      None => Ok(CreateOrderResponse::AddressNotFound),
    },
    Err(_e) => Err("Error while getting address".to_string()),
  }
}
