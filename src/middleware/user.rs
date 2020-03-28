use std::pin::Pin;
use std::task::{Context, Poll};

use actix_service::{Service, Transform};
use actix_web::{dev::ServiceRequest, dev::ServiceResponse, http::header, Error};
use futures::future::{ok, Ready};
use futures::Future;
use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};

pub struct ResolveToken;

impl<S, B> Transform<S> for ResolveToken
where
  S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
  S::Future: 'static,
  B: 'static,
{
  type Request = ServiceRequest;
  type Response = ServiceResponse<B>;
  type Error = Error;
  type InitError = ();
  type Transform = ResolveTokenMiddleware<S>;
  type Future = Ready<Result<Self::Transform, Self::InitError>>;

  fn new_transform(&self, service: S) -> Self::Future {
    ok(ResolveTokenMiddleware { service })
  }
}

pub struct ResolveTokenMiddleware<S> {
  service: S,
}

impl<S, B> Service for ResolveTokenMiddleware<S>
where
  S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
  S::Future: 'static,
  B: 'static,
{
  type Request = ServiceRequest;
  type Response = ServiceResponse<B>;
  type Error = Error;
  type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

  fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
    self.service.poll_ready(cx)
  }

  fn call(&mut self, mut req: ServiceRequest) -> Self::Future {
    req
      .headers_mut()
      .remove(header::HeaderName::from_static("token"));
    let mut token = "";
    let headers = req.headers().clone();
    match headers.get("cookie") {
      Some(cookie) => {
        let split = cookie.to_str().unwrap().split(";");
        let cookies: Vec<&str> = split.collect();
        for cookie in &cookies {
          if cookie.contains("access_token=") {
            let token_cookie: Vec<&str> = cookie.split("=").collect();
            token = token_cookie[1];
          }
        }
      }
      None => {}
    };
    if token.len() > 0 {
      req.headers_mut().insert(
        header::HeaderName::from_static("token"),
        header::HeaderValue::from_str(token).unwrap(),
      );
    }

    let fut = self.service.call(req);

    Box::pin(async move {
      let res = fut.await?;
      Ok(res)
    })
  }
}

pub struct ResolveId;

impl<S, B> Transform<S> for ResolveId
where
  S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
  S::Future: 'static,
  B: 'static,
{
  type Request = ServiceRequest;
  type Response = ServiceResponse<B>;
  type Error = Error;
  type InitError = ();
  type Transform = ResolveIdMiddleware<S>;
  type Future = Ready<Result<Self::Transform, Self::InitError>>;

  fn new_transform(&self, service: S) -> Self::Future {
    ok(ResolveIdMiddleware { service })
  }
}

pub struct ResolveIdMiddleware<S> {
  service: S,
}

impl<S, B> Service for ResolveIdMiddleware<S>
where
  S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
  S::Future: 'static,
  B: 'static,
{
  type Request = ServiceRequest;
  type Response = ServiceResponse<B>;
  type Error = Error;
  type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

  fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
    self.service.poll_ready(cx)
  }

  fn call(&mut self, mut req: ServiceRequest) -> Self::Future {
    req
      .headers_mut()
      .remove(header::HeaderName::from_static("user_id"));
    let headers = req.headers().clone();
    match headers.get("token") {
      Some(token) => match token.to_str() {
        Ok(token_str) => {
          let split = token_str.split(".");
          let token_content: Vec<&str> = split.collect();
          if token_content.len() == 3 {
            let validation = Validation {
              leeway: 0,
              validate_exp: false,
              validate_nbf: false,
              iss: None,
              sub: None,
              aud: None,
              algorithms: vec![Algorithm::HS256],
            };
            println!("jwt secret: {:?}", dotenv!("JWT_SECRET"));
            match decode::<crate::controller::user::Claims>(
              token_str,
              &DecodingKey::from_secret(dotenv!("JWT_SECRET").as_ref()),
              &validation,
            ) {
              Ok(decoded_token) => {
                req.headers_mut().insert(
                  header::HeaderName::from_static("user_id"),
                  header::HeaderValue::from_str(&decoded_token.claims.sub).unwrap(),
                );
              }
              Err(e) => println!("Error while decoding token: {:?}", e),
            }
          }
        }
        Err(e) => println!("Error while stringifying token: {:?}", e),
      },
      None => {}
    };

    let fut = self.service.call(req);

    Box::pin(async move {
      let res = fut.await?;
      Ok(res)
    })
  }
}
