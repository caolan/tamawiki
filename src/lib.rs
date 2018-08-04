#![deny(warnings)]

#[cfg(test)]
#[macro_use] extern crate proptest;

#[macro_use] extern crate serde_derive;
extern crate serde;
extern crate futures;
extern crate warp;


pub mod connection;
pub mod document;
pub mod store;


use warp::{Filter, Reply};
use warp::filters::BoxedFilter;
use futures::future::Future;
use std::sync::{Arc, Mutex};
use std::path::PathBuf;

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
    let display_document = document_content.map(|data: (SequenceId, Document)| {
        format!("{}\n\n\nseq: {}", data.1.content, data.0)
    });

    // app filter
    display_document.boxed()
}
