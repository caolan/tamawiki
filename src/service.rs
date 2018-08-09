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
//! use tamawiki::store::memory::MemoryStore;
//! use futures::future::Future;
//! use hyper::Server;
//!
//! let addr = ([127, 0, 0, 1], 8080).into();
//! let store = MemoryStore::default();
//!
//! let server = Server::bind(&addr)
//!     .serve(TamaWiki {store})
//!     .map_err(|err| eprintln!("Server error: {}", err));
//! ```

use hyper::{Response, Request};
use hyper::body::Body;
use hyper::service::{NewService, Service};
use futures::future::{self, Future, FutureResult};
use http::StatusCode;
use tera::Tera;
use std::fmt::{self, Display};
use std::error::Error;
use std::path::PathBuf;

use store::{Store, StoreError};


lazy_static! {
    static ref TERA: Tera = {
        compile_templates!("templates/**/*")
    };
}

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
pub struct TamaWiki<T: Store> {
    /// A cloneable interface to the backing store used by TamaWiki to
    /// persist document updates
    pub store: T
}

impl<T: Store> NewService for TamaWiki<T> {
    type ReqBody = Body;
    type ResBody = Body;
    type Error = TamaWikiError;
    type Service = TamaWikiService<T>;
    type InitError = TamaWikiError;
    type Future = FutureResult<Self::Service, Self::InitError>;
    
    fn new_service(&self) -> Self::Future {
        future::ok(TamaWikiService {
            store: self.store.clone()
        })
    }
}

/// Handles a request and returns a response
pub struct TamaWikiService<T: Store> {
    /// A cloneable interface to the backing store used by TamaWiki to
    /// persist document updates
    pub store: T
}

#[derive(Debug)]
enum HttpError {
    InternalServerError(String)
}

impl Display for HttpError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            HttpError::InternalServerError(ref err) => 
                write!(f, "InternalServerError: {}", err),
        }
    }
}

impl Error for HttpError {
    fn description(&self) -> &str {
        match *self {
            HttpError::InternalServerError(ref err) => err,
        }
    }
}

impl<T: Store> TamaWikiService<T> {
    fn serve_document(&mut self, req: &Request<Body>) ->
        Box<Future<Item=Response<Body>, Error=HttpError> + Send>
    {
        let path = PathBuf::from(&req.uri().path()[1..]);
        Box::new(
            self.store.content(&path.as_path()).then(|result| {
                match result {
                    Ok((seq, doc)) => {
                        let ctx = json!({
                            "title": "Document",
                            "content": doc.content,
                            "seq": seq
                        });
                        let text = TERA.render("document.html", &ctx).unwrap();
                        Ok(Response::builder()
                           .body(Body::from(text))
                           .unwrap())
                    }
                    Err(StoreError::NotFound) => {
                        let ctx = json!({
                            "title": "Document",
                        });
                        let text = TERA.render("empty_document.html", &ctx).unwrap();
                        Ok(Response::builder()
                           .status(StatusCode::NOT_FOUND)
                           .body(Body::from(text))
                           .unwrap())
                    },
                    Err(err) => {
                        Err(HttpError::InternalServerError(format!("{}", err)))
                    }
                }
            })
        )
    }
}

impl<T: Store> Service for TamaWikiService<T> {
    type ReqBody = Body;
    type ResBody = Body;
    type Error = TamaWikiError;
    type Future = Box<Future<Item=Response<Self::ResBody>,
                             Error=Self::Error> +
                      Send>;
    
    fn call(&mut self, req: Request<Self::ReqBody>) -> Self::Future {
        Box::new(
            self.serve_document(&req).then(|result| {
                match result {
                    Ok(response) => future::ok(response),
                    Err(HttpError::InternalServerError(err)) => {
                        let ctx = json!({
                            "title": "Internal Server Error",
                            "error": err
                        });
                        let text = TERA.render("500.html", &ctx).unwrap();
                        future::ok(Response::builder()
                                   .status(StatusCode::INTERNAL_SERVER_ERROR)
                                   .body(Body::from(text))
                                   .unwrap())
                    }
                }
            }).map_err(|_err: HttpError| {
                TamaWikiError {}
            })
        )
    }
}
