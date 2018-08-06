#[cfg(test)]
#[macro_use] extern crate proptest;

#[macro_use] extern crate tera;
#[macro_use] extern crate lazy_static;
#[macro_use] extern crate serde_derive;
#[macro_use] extern crate serde_json;
extern crate serde_urlencoded;
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
use std::collections::HashMap;
use http::{StatusCode, Response};
use serde::Serialize;
use tera::Tera;

use store::{Store, StoreClient, StoreError};


lazy_static! {
    static ref TERA: Tera = {
        compile_templates!("templates/**/*")
    };
}


pub fn app<T: 'static + Store>(store: Arc<Mutex<T>>) -> BoxedFilter<(impl Reply,)> {
    warp::get(static_files())
        // TODO: chaining render_rejections() off document() until
        // https://github.com/seanmonstar/warp/issues/8#issuecomment-410100585
        // is implemented
        .or(document(store).or_else(render_rejection))
        .boxed()
}

fn static_files() -> BoxedFilter<(impl Reply,)> {
    warp::path("_static").and(warp::fs::dir("static")).boxed()
}

fn document<T: 'static + Store>(store: Arc<Mutex<T>>) ->
    BoxedFilter<(Response<String>,)>
{
    // request a new store client for this request
    let store_client = warp::any().map(move || {
        store.lock().unwrap().client()
    });

    // extract a document path using the remaining URL path components
    let document_path = warp::any()
        .and(warp::path::tail())
        .and_then(|tail: warp::path::Tail| {
            let tail = tail.as_str();
            // all paths starting with an underscore are reserved
            if tail.starts_with("_") {
                Err(warp::reject::not_found())
            }
            else {
                Ok(PathBuf::from(tail))
            }
        });

    // serde_urlencoded (0.5.2) doesn't support deserializing unit
    // enums, so instead of using a custom query struct to match
    // available actions we have to fall back to a hashmap for now.
    // TODO: change this one warp support a different deserializer or
    // https://github.com/nox/serde_urlencoded/pull/30 is merged.
    let document_action = warp::query()
        .map(|q: HashMap<String, String>| {
            match q.get("action") {
                Some(name) if name == "edit" => DocumentAction::Edit,
                _ => DocumentAction::View,
            }
        })
        .or_else(|_| {
            Ok((DocumentAction::View,))
        });

    // display document content from store, render empty document
    // page, or return warp rejection
    warp::any()
        .and(document_path)
        .and(document_action)
        .and(store_client.clone())
        .and_then(|path: PathBuf, action: DocumentAction, store: T::Client| {
            store.content(&path.as_path()).then(move |result| {
                match result {
                    Ok((seq, doc)) => {
                        let tmpl = match action {
                            DocumentAction::Edit => "editor.html",
                            DocumentAction::View => "document.html",
                        };
                        let res = Response::builder()
                            .status(StatusCode::OK)
                            .header("Content-Type", "text/html")
                            .body(
                                template(tmpl, &json!({
                                    "title": "Document",
                                    "seq": seq,
                                    "content": doc.content
                                }))?
                            );
                        Ok(res.unwrap())
                    },
                    Err(StoreError::NotFound) => {
                        let body = match action {
                            DocumentAction::Edit => {
                                template("editor.html", &json!({
                                    "title": "Document",
                                    "seq": 0,
                                    "content": ""
                                }))?
                            },
                            DocumentAction::View => {
                                template("empty_document.html", &json!({
                                    "title": "Document",
                                }))?
                            }
                        };
                        let res = Response::builder()
                            .status(StatusCode::NOT_FOUND)
                            .header("Content-Type", "text/html")
                            .body(body);
                        Ok(res.unwrap())
                    },
                    Err(_) =>
                        Err(warp::reject::server_error()),
                }
            })
        })
        .boxed()
}

#[derive(Deserialize, Debug)]
enum DocumentAction {
    View,
    Edit,
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
