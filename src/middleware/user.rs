use std::pin::Pin;
use std::task::{Context, Poll};

use actix_service::{Service, Transform};
use actix_web::{dev::ServiceRequest, dev::ServiceResponse, Error, http::{header}};
use futures::future::{ok, Ready};
use futures::Future;

pub struct Resolve;

impl<S, B> Transform<S> for Resolve
where
  S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
  S::Future: 'static,
  B: 'static,
{
  type Request = ServiceRequest;
  type Response = ServiceResponse<B>;
  type Error = Error;
  type InitError = ();
  type Transform = ResolveMiddleware<S>;
  type Future = Ready<Result<Self::Transform, Self::InitError>>;

  fn new_transform(&self, service: S) -> Self::Future {
    ok(ResolveMiddleware { service })
  }
}

pub struct ResolveMiddleware<S> {
  service: S,
}

impl<S, B> Service for ResolveMiddleware<S>
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
    println!("Hi from start. You requested: {}", req.path());

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
            println!("token: {}", token);
          }
        }
      }
      None => println!("none o.o",),
    };
    if token.len() > 0 {
      req.headers_mut().insert(
        header::HeaderName::from_static("user_id"),
        header::HeaderValue::from_str(token).unwrap(),
      );
    }

    let fut = self.service.call(req);

    Box::pin(async move {
      let res = fut.await?;

      println!("Hi from response");
      Ok(res)
    })
  }
}
