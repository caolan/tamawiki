//! A wiki implemented in Rust
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
use actix_web::{Path, State, App, HttpRequest, HttpResponse, http, server, fs};
use actix_web::error::Error;
use actix::prelude::*;

pub mod document;
pub mod store;
pub mod templates;

use store::memory::MemoryStore;
use store::*;


/// Per-thread application state
pub struct TamaWikiState<T: Store> {
    /// The actix address of the Store used to store document Updates
    pub store: Addr<T>,
}

/// Creates a new TamaWiki actix_web App
pub fn app<T: Store>(state: TamaWikiState<T>) -> App<TamaWikiState<T>> {
    App::with_state(state)
        .handler(
            "/_static",
            fs::StaticFiles::new("static").unwrap()
        )
        .resource("/{tail:.*}", |r| {
            r.get().with_async(get_document)
        })
}

#[derive(Deserialize)]
struct PathInfo {
    tail: PathBuf
}

fn get_document<T: Store>(data: (State<TamaWikiState<T>>, Path<PathInfo>)) ->
    Box<Future<Item=HttpResponse, Error=Error>>
{
    let (state, path) = data;
    let msg = Content { path: path.into_inner().tail };
    let res = state.store.send(msg)
        .from_err()
        .map(|result| {
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

/// Creates a new TamaWiki HTTP server and binds to the given address
pub fn server(addr: &str) -> server::HttpServer<impl server::HttpHandler>  {
    // Start MemoryStore in another thread
    let store = Arbiter::start(|ctx: &mut Context<_>| {
        // TODO: remove this when the issue with dropped actix messages
        // when running apache bench is resolved?
        // see issue: https://github.com/actix/actix/issues/120
        // set unbounded mailbox capacity
        ctx.set_mailbox_capacity(0);
        MemoryStore::default()
    });
    let srv = server::new(move || app::<MemoryStore>(TamaWikiState {
        store: store.clone(),
    }));
    srv.bind(addr).unwrap()
}

