#![deny(warnings)]

#[cfg(test)]
#[macro_use] extern crate proptest;

#[macro_use] extern crate serde_derive;
extern crate serde;
extern crate futures;
extern crate warp;
extern crate http;


pub mod connection;
pub mod document;
pub mod store;


use warp::{Filter, Reply};
use warp::filters::BoxedFilter;
use futures::future::Future;
use std::sync::{Arc, Mutex};
use std::path::PathBuf;
use http::{StatusCode, Response};

use store::{Store, StoreClient, StoreError, SequenceId};
use document::Document;


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
    let document_content = warp::any()
        .and(store_client)
        .and(document_path)
        .and_then(|store: T::Client, tail: PathBuf| {
            store.content(&tail.as_path())
                .map_err(|err| {
                    match err {
                        StoreError::NotFound => warp::reject::not_found(),
                        _ => warp::reject::server_error(),
                    }
                })
        });

    // display document on success
    let display_document = warp::get(
        document_content
            .map(|data: (SequenceId, Document)| {
                Response::builder()
                    .status(StatusCode::OK)
                    .header("Content-Type", "text/html")
                    .body(format!("{}\n\n\nseq: {}", data.1.content, data.0))
            })
    );

    let static_files = warp::path("_static")
        .and(warp::fs::dir("static"));
    
    // app filter
    warp::get(static_files)
        .or(display_document
            .or_else(|err: warp::reject::Rejection| {
                match err.status() {
                    StatusCode::NOT_FOUND => {
                        Ok((Response::builder()
                            .status(StatusCode::NOT_FOUND)
                            .header("Content-Type", "text/html")
                            .body("Not found\n".to_owned()),))
                    },
                    _ => Err(err)
                }
            }))
        .boxed()
}
