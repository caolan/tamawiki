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
//! use std::path::PathBuf;
//!
//! let addr = ([127, 0, 0, 1], 8080).into();
//! let store = MemoryStore::default();
//! let static_path = PathBuf::from("static");
//!
//! let server = Server::bind(&addr)
//!     .serve(TamaWiki {store, static_path})
//!     .map_err(|err| eprintln!("Server error: {}", err));
//! ```

use hyper::{Response, Request};
use hyper::body::Body;
use hyper::service::{NewService, Service};
use futures::future::{self, Future, FutureResult};
use http::StatusCode;
use tera::Tera;
use hyper_staticfile::{self, resolve};
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
    pub store: T,
    /// Path to serve static files from
    pub static_path: PathBuf,
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
            store: self.store.clone(),
            static_path: self.static_path.clone()
        })
    }
}

#[derive(Debug)]
enum HttpError {
    InternalServerError(String),
    MethodNotAllowed,
    BadRequest,
    NotFound,
    Unauthorized,
}

impl Display for HttpError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            HttpError::MethodNotAllowed => write!(f, "MethodNotAllowed"),
            HttpError::BadRequest => write!(f, "BadRequest"),
            HttpError::NotFound => write!(f, "NotFound"),
            HttpError::Unauthorized => write!(f, "Unauthorized"),
            HttpError::InternalServerError(ref err) => 
                write!(f, "InternalServerError: {}", err),
        }
    }
}

impl Error for HttpError {
    fn description(&self) -> &str {
        match *self {
            HttpError::MethodNotAllowed => "method not allowed",
            HttpError::BadRequest => "bad request",
            HttpError::NotFound => "not found",
            HttpError::Unauthorized => "unauthorized",
            HttpError::InternalServerError(ref err) => err,
        }
    }
}

/// Handles a request and returns a response
pub struct TamaWikiService<T: Store> {
    /// A cloneable interface to the backing store used by TamaWiki to
    /// persist document updates
    pub store: T,
    /// Path to serve static files from
    pub static_path: PathBuf,
}

impl<T: Store> TamaWikiService<T> {
    
    fn serve_static(&mut self, req: Request<Body>) -> 
        Box<Future<Item=Response<Body>, Error=HttpError> + Send>
    {
        // First, resolve the request. Returns a `ResolveFuture` for a `ResolveResult`.
        let result = resolve(&self.static_path.as_path(), &req).map_err(|err| {
            println!("{}", err);
            HttpError::InternalServerError(format!("{}", err))
        }).and_then(|res| {
            use hyper_staticfile::ResolveResult::*;
            match res {
                // The request was not `GET` or `HEAD` request,
                MethodNotMatched => Err(HttpError::MethodNotAllowed),
                // The request URI was not just a path.
                UriNotMatched => Err(HttpError::BadRequest),
                // The requested file does not exist.
                NotFound => Err(HttpError::NotFound),
                // The requested file could not be accessed.
                PermissionDenied => Err(HttpError::Unauthorized),
                // A directory was requested as a file.
                IsDirectory => Ok(IsDirectory),
                // The requested file was found.
                Found(file, metadata) => Ok(Found(file, metadata)),
            }
        });

        // Then, build a response based on the result.
        // The `ResponseBuilder` is typically a short-lived, per-request instance.
        Box::new(
            result.map(move |result| {
                hyper_staticfile::ResponseBuilder::new()
                    .build(&req, result)
                    .unwrap()
            })
        )
    }
    
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
        let res = if req.uri().path().starts_with("/_static/") {
            self.serve_static(req)
        } else {
            self.serve_document(&req)
        };
        let res = res.then(|result| -> FutureResult<Response<Body>, TamaWikiError> {
            match result {
                Ok(response) => future::ok(response),
                Err(HttpError::InternalServerError(err)) => {
                    let ctx = json!({
                        "title": "Internal Server Error",
                        "error": err
                    });
                    let text = TERA.render("500.html", &ctx).unwrap();
                    future::ok(
                        Response::builder()
                            .status(StatusCode::INTERNAL_SERVER_ERROR)
                            .body(Body::from(text))
                            .unwrap())
                },
                Err(HttpError::NotFound) => {
                    let ctx = json!({
                        "title": "Not Found"
                    });
                    let text = TERA.render("404.html", &ctx).unwrap();
                    future::ok(
                        Response::builder()
                            .status(StatusCode::NOT_FOUND)
                            .body(Body::from(text))
                            .unwrap())
                },
                Err(HttpError::MethodNotAllowed) => {
                    future::ok(
                        Response::builder()
                            .status(StatusCode::METHOD_NOT_ALLOWED)
                            .body(Body::from(""))
                            .unwrap())
                },
                Err(HttpError::Unauthorized) => {
                    future::ok(
                        Response::builder()
                            .status(StatusCode::UNAUTHORIZED)
                            .body(Body::from(""))
                            .unwrap())
                },
                Err(HttpError::BadRequest) => {
                    future::ok(
                        Response::builder()
                            .status(StatusCode::BAD_REQUEST)
                            .body(Body::from(""))
                            .unwrap())
                },
            }
        });
        Box::new(res)
    }
}
