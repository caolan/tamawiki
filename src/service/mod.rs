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
//!     .serve(TamaWiki::new(store, "static"))
//!     .map_err(|err| eprintln!("Server error: {}", err));
//! ```

use hyper::{Response, Request};
use hyper::body::Body;
use hyper::service::{NewService, Service};
use futures::future::{self, Future, FutureResult};
use http::StatusCode;
use hyper_staticfile::{self, resolve};
use std::path::PathBuf;
use futures::stream::Stream;
use futures::sink::Sink;

use templates::TERA;
use store::{Store, StoreError};
use session::connection::{WebSocketConnection, ConnectionError};
use session::message::{ServerMessage, ConnectedMessage, JoinMessage, LeaveMessage, EditMessage};
use session::DocumentSessionManager;

mod error;
mod request;
mod upgrade;

use service::error::{TamaWikiError, HttpError};
use service::request::query_params;
use service::upgrade::{websocket_upgrade, is_websocket_upgrade_request};
use document::{Event, Join, Leave, Edit};
use store::SequenceId;


/// Handles a request and returns a response
#[derive(Clone)]
pub struct TamaWiki<T: Store + Sync> {
    // A cloneable interface to the backing store used by TamaWiki to
    // persist document updates
    store: T,
    // Path to serve static files from
    static_path: PathBuf,
    // Co-ordinates document access for editors
    document_sessions: DocumentSessionManager<T>,
}

impl<T: Store + Sync> TamaWiki<T> {
    /// Creates a new instace of TamaWiki
    pub fn new<P: Into<PathBuf>>(store: T, static_path: P) -> Self {
        let document_sessions = DocumentSessionManager::new(store.clone());
        Self {
            static_path: static_path.into(),
            document_sessions,
            store,
        }
    }
    
    fn serve_static(&mut self, req: Request<Body>) -> 
        Box<Future<Item=Response<Body>, Error=HttpError> + Send>
    {
        // First, resolve the request. Returns a `ResolveFuture` for a `ResolveResult`.
        let result = resolve(&self.static_path.as_path(), &req).map_err(|err| {
            eprintln!("Error serving static file: {}", err);
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
        let q = query_params(req);
        let edit = match q.get("action") {
            Some(value) => value == "edit",
            _ => false,
        };
        Box::new(
            self.store.content(&path.as_path()).then(move |result| {
                match result {
                    Ok((seq, doc)) => {
                        let ctx = json!({
                            "title": "Document",
                            "content": doc.content,
                            "participants": doc.participants,
                            "seq": seq
                        });
                        let tmpl = if edit {
                            "editor.html"
                        } else {
                            "document.html"
                        };
                        let text = TERA.render(tmpl, &ctx).unwrap();
                        Ok(Response::builder()
                           .body(Body::from(text))
                           .unwrap())
                    }
                    Err(StoreError::NotFound) => {
                        let text = if edit {
                            TERA.render("editor.html", &json!({
                                "title": "Document (empty)",
                                "content": "",
                                "participants": [],
                                "seq": 0
                            })).unwrap()
                        } else {
                            TERA.render("new_document.html", &json!({
                                "title": "Document"
                            })).unwrap()
                        };
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

    fn handle_websocket(&self, req: Request<Body>) ->
        Box<Future<Item=Response<Body>, Error=HttpError> + Send>
    {
        let path = PathBuf::from(&req.uri().path()[1..]);
        let q = query_params(&req);
        
        let since: SequenceId = match q.get("seq") {
            // TODO: respond with http error message if seq parameter is invalid
            Some(x) => x.parse().unwrap_or(0),
            None => 0
        };
        
        let document_sessions = self.document_sessions.clone();
        
        websocket_upgrade(req, move |websocket| {
            document_sessions.join(&path.as_path())
                .map_err(|e| {
                    eprintln!("Error joining document session: {:?}", e);
                })
                .and_then(move |(id, session)| {
                    let (tx, rx) = WebSocketConnection::from(websocket).split();
                    let events = session.subscribe(since);

                    let send_server_messages = tx.send(
                        ServerMessage::Connected(ConnectedMessage {id})
                    ).and_then(|tx| {
                        tx.send_all(
                            events.map(|(seq, event)| {
                                match event {
                                    Event::Join(Join {id}) => {
                                        ServerMessage::Join(JoinMessage {
                                            seq,
                                            id,
                                        })
                                    },
                                    Event::Leave(Leave {id}) => {
                                        ServerMessage::Leave(LeaveMessage {
                                            seq,
                                            id,
                                        })
                                    },
                                    Event::Edit(Edit {author, operations}) => {
                                        ServerMessage::Edit(EditMessage {
                                            seq,
                                            author,
                                            operations,
                                        })
                                    },
                                }
                            })
                        ).map(|_| ())
                    });

                    let read_client_messages = rx.for_each(
                        |msg| -> FutureResult<(), ConnectionError> {
                            println!("received: {:?}", msg);
                            future::ok(())
                        }
                    );
                    
                    send_server_messages
                        .select(read_client_messages)
                        .then(move |result: Result<_, (ConnectionError, _)>| {
                            if let Err((err, _)) = result {
                                eprintln!("WebSocket error: {:?}", err);
                            }
                            session.leave(id)
                                .map(|_| ())
                                .map_err(|err| {
                                    eprintln!("Error leaving document session: {:?}", err);
                                })
                        })
                })
        })
    }
}

impl<T: Store + Sync> NewService for TamaWiki<T> {
    type ReqBody = Body;
    type ResBody = Body;
    type Error = TamaWikiError;
    type Service = TamaWiki<T>;
    type InitError = TamaWikiError;
    type Future = FutureResult<Self::Service, Self::InitError>;
    
    fn new_service(&self) -> Self::Future {
        future::ok(self.clone())
    }
}

impl<T: Store + Sync> Service for TamaWiki<T> {
    type ReqBody = Body;
    type ResBody = Body;
    type Error = TamaWikiError;
    type Future = Box<Future<Item=Response<Self::ResBody>,
                             Error=Self::Error> +
                      Send>;
    
    fn call(&mut self, req: Request<Self::ReqBody>) -> Self::Future {
        let res = if req.uri().path().starts_with("/_static/") {
            self.serve_static(req)
        } else if is_websocket_upgrade_request(&req) {
            self.handle_websocket(req)
        } else {
            self.serve_document(&req)
        };
        let res = res.then(|result| -> FutureResult<Response<Body>, TamaWikiError> {
            match result {
                Ok(response) => future::ok(response),
                Err(err) => future::ok(err.into()),
            }
        });
        Box::new(res)
    }
}
