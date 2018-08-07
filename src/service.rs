//! TamaWiki HTTP service suitable for serving via Hyper
//!
//! # Example
//!
//! ```
//! extern crate hyper;
//! extern crate tamawiki;
//! extern crate futures;
//!
//! use tamawiki::service::TamaWiki;
//! use futures::future::Future;
//! use hyper::Server;
//!
//! let addr = ([127, 0, 0, 1], 8080).into();
//!
//! let server = Server::bind(&addr)
//!     .serve(TamaWiki::default())
//!     .map_err(|err| eprintln!("Server error: {}", err));
//! ```

use hyper::{Response, Request};
use hyper::body::Body;
use hyper::service::{NewService, Service};
use futures::future::{self, FutureResult};
use std::error::Error;
use std::fmt;


/// Error conditions that could not be handled as a HTTP response
#[derive(Debug)]
pub struct TamaWikiError {}

impl fmt::Display for TamaWikiError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "TamaWikiError")
    }
}

impl Error for TamaWikiError {}

/// Constructs TamaWikiServices
#[derive(Default)]
pub struct TamaWiki {}

impl NewService for TamaWiki {
    type ReqBody = Body;
    type ResBody = Body;
    type Error = TamaWikiError;
    type Service = TamaWikiService;
    type InitError = TamaWikiError;
    type Future = FutureResult<Self::Service, Self::InitError>;
    
    fn new_service(&self) -> Self::Future {
        future::ok(TamaWikiService {})
    }
}

/// Handles a request and returns a response
pub struct TamaWikiService {}

impl Service for TamaWikiService {
    type ReqBody = Body;
    type ResBody = Body;
    type Error = TamaWikiError;
    type Future = FutureResult<Response<Self::ResBody>, Self::Error>;
    
    fn call(&mut self, _req: Request<Self::ReqBody>) -> Self::Future {
        future::ok(Response::new(Body::from("Hello, world!\n")))
    }
}
