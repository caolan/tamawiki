//! A wiki implemented in Rust

// for property testing in session module
#[cfg(test)]
#[macro_use] extern crate proptest;

#[macro_use]
extern crate tera;

#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate serde_derive;

#[macro_use]
extern crate serde_json;

extern crate serde;
extern crate actix_web;
extern crate actix;
extern crate futures;

use std::path::PathBuf;
use futures::future::Future;
use actix_web::{Query, Path, State, App, Responder, HttpResponse, HttpRequest, http, server, fs, ws, pred};
use actix_web::error::Error;
use actix::prelude::*;

pub mod document;
pub mod store;
pub mod templates;
pub mod connection;
pub mod session;

use connection::Connection;
use store::memory::MemoryStore;
use session::EditSessionManager;
use store::*;


/// Per-thread application state
pub struct TamaWikiState<T: Store> {
    /// The actix address of the Store used to store document Updates
    pub store: Addr<T>,
    pub session_manager: Addr<EditSessionManager<T>>,
}

/// Creates a new TamaWiki actix_web App
pub fn app<T: Store>(state: TamaWikiState<T>) -> App<TamaWikiState<T>> {
    App::with_state(state)
        .handler(
            "/_static",
            fs::StaticFiles::new("static").unwrap()
        )
        .resource("/{tail:.*}", |r| {
            r.route().filter(pred::Header("upgrade", "websocket")).with(start_websocket);
            r.get().with_async(get_document)
        })
}

#[derive(Debug, Deserialize)]
struct DocPath {
    tail: PathBuf
}

#[derive(Debug, Deserialize)]
struct DocQuery {
    #[serde(default)]
    edit: bool
}

fn start_websocket<T: Store>(data: (HttpRequest<TamaWikiState<T>>, Path<DocPath>)) -> impl Responder {
    let (req, path) = data;
    let path = path.into_inner().tail;
    ws::start(&req, Connection::new(path, 0))
}

fn get_document<T: Store>(data: (State<TamaWikiState<T>>, Path<DocPath>, Query<DocQuery>)) ->
    Box<Future<Item=HttpResponse, Error=Error>>
{
    let (state, path, query) = data;
    let path = path.into_inner().tail;
    
    match query.into_inner() {
        DocQuery { edit: true } => edit_document(state, path),
        _ => display_document(state, path),
    }
}


fn display_document<T: Store>(state: State<TamaWikiState<T>>, path: PathBuf) ->
    Box<Future<Item=HttpResponse, Error=Error>>
{
    let res = state.store.send(Content { path })
        .from_err()
        .map(move |result| {
            match result {
                Ok((seq, doc)) => {
                    let mut res = HttpResponse::Ok();
                    res.header(http::header::CONTENT_TYPE, "text/html");
                    
                    templates::render_response(
                        res,
                        "document.html",
                        &json!({
                            "title": "Document",
                            "seq": seq,
                            "content": doc.content
                        })
                    )
                },
                Err(StoreError::NotFound) => {
                    let mut res = HttpResponse::NotFound();
                    res.header(http::header::CONTENT_TYPE, "text/html");

                    templates::render_response(
                        res,
                        "404.html",
                        &json!({
                            "title": "Not found"
                        })
                    )
                },
                Err(_) => {
                    HttpResponse::InternalServerError()
                        .header(http::header::CONTENT_TYPE, "text/html")
                        .body("Error")
                },
            }
        });
    Box::new(res)
}

fn edit_document<T: Store>(state: State<TamaWikiState<T>>, path: PathBuf) ->
    Box<Future<Item=HttpResponse, Error=Error>>
{
    let res = state.store.send(Content { path })
        .from_err()
        .map(move |result| {
            match result {
                Ok((seq, doc)) => {
                    let mut res = HttpResponse::Ok();
                    res.header(http::header::CONTENT_TYPE, "text/html");
                    
                    templates::render_response(
                        res,
                        "editor.html",
                        &json!({
                            "title": "Document",
                            "seq": seq,
                            "content": doc.content
                        })
                    )
                },
                Err(StoreError::NotFound) => {
                    let mut res = HttpResponse::NotFound();
                    res.header(http::header::CONTENT_TYPE, "text/html");

                    templates::render_response(
                        res,
                        "editor.html",
                        &json!({
                            "title": "Document",
                            "seq": 0,
                            "content": ""
                        })
                    )
                },
                Err(_) => {
                    HttpResponse::InternalServerError()
                        .header(http::header::CONTENT_TYPE, "text/html")
                        .body("Error")
                },
            }
        });
    Box::new(res)
}

/// Creates a new TamaWiki HTTP server and binds to the given address
pub fn server(addr: &str) -> server::HttpServer<impl server::HttpHandler>  {
    // Start MemoryStore in another thread
    let store = Arbiter::start(|_ctx| {
        MemoryStore::default()
    });
    let session_manager = Arbiter::start(|_ctx| {
        EditSessionManager::default()
    });
    let srv = server::new(move || app::<MemoryStore>(TamaWikiState {
        store: store.clone(),
        session_manager: session_manager.clone(),
    }));
    srv.bind(addr).unwrap()
}

