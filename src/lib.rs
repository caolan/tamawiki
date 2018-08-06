#![deny(warnings)]

#[cfg(test)]
#[macro_use] extern crate proptest;

#[macro_use] extern crate tera;
#[macro_use] extern crate lazy_static;
#[macro_use] extern crate serde_derive;
#[macro_use] extern crate serde_json;
extern crate serde;
extern crate futures;
extern crate warp;
extern crate http;


pub mod connection;
pub mod document;
pub mod store;


use warp::{Filter, Reply};
use warp::filters::BoxedFilter;
use warp::reject::Rejection;
use futures::future::Future;
use std::sync::{Arc, Mutex};
use std::path::PathBuf;
use http::{StatusCode, Response};
use serde::Serialize;
use tera::Tera;

use store::{Store, StoreClient, StoreError, SequenceId};
use document::Document;


lazy_static! {
    static ref TERA: Tera = {
        compile_templates!("templates/**/*")
    };
}


pub fn app<T: 'static + Store>(store: Arc<Mutex<T>>) -> BoxedFilter<(impl Reply,)> {
    // request a new store client for this request
    let store_client = warp::any().map(move || {
        store.lock().unwrap().client()
    });

    // extract a document path using the remaining URL path components
    let document_path = warp::any()
        .and(warp::path::tail())
        .map(|tail: warp::path::Tail| PathBuf::from(tail.as_str()));

    // get document content from store or return error
    let display_document = warp::any()
        .and(store_client)
        .and(document_path)
        .and_then(|store: T::Client, tail: PathBuf| {
            store.content(&tail.as_path()).map_err(|err| {
                match err {
                    StoreError::NotFound => warp::reject::not_found(),
                    _ => warp::reject::server_error()
                }
            })
        })
        .and_then(|(seq, doc): (SequenceId, Document)| {
            template("document.html", &json!({
                "title": "Document",
                "seq": seq,
                "content": doc.content
            }))
        })
        .map(|body| {
            Response::builder()
                .status(StatusCode::OK)
                .header("Content-Type", "text/html")
                .body(body)
                .unwrap()
        });

    let static_files = warp::path("_static")
        .and(warp::fs::dir("static"));
    
    // app filter
    warp::get(static_files)
        .or(display_document
            .or_else(render_rejection))
        .boxed()
}


fn template<T: Serialize>(name: &'static str, context: &T) ->
    Result<String, warp::Rejection>
{
    TERA.render(name, &context).map_err(|_| {
        warp::reject::server_error()
    })
}

fn render_rejection(err: Rejection) ->
    Result<(Response<String>,), Rejection>
{
    match err.status() {
        StatusCode::NOT_FOUND => {
            Ok((
                Response::builder()
                    .status(StatusCode::NOT_FOUND)
                    .header("Content-Type", "text/html")
                    .body(
                        template("404.html", &json!({
                            "title": "Not found"
                        }))?
                    )
                    .unwrap(),
            ))
        },
        _ => Err(err),
    }
}
