#[macro_use]
extern crate tamawiki;
extern crate futures;
extern crate http;
extern crate hyper;

use futures::future::Future;
use futures::stream::Stream;
use http::{Request, StatusCode};
use hyper::service::Service;
use hyper::Body;

use tamawiki::store::memory::MemoryStore;
use tamawiki::TamaWiki;

#[test]
fn get_missing_page() {
    let store = MemoryStore::default();
    let mut service = TamaWiki::new(store, "public/dist");

    let request = Request::get("/missing.html").body(Body::from("")).unwrap();

    let response = service.call(request).wait().unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[test]
fn get_page_content_from_store() {
    let store = memorystore! {
        "test.html" => "Testing 123"
    };
    let mut service = TamaWiki::new(store, "public/dist");

    let request = Request::get("/test.html").body(Body::from("")).unwrap();

    let response = service.call(request).wait().unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = response
        .into_body()
        .fold(
            Vec::new(),
            |mut body, chunk| -> Result<Vec<u8>, hyper::Error> {
                body.append(&mut chunk.to_vec());
                Ok(body)
            },
        ).wait()
        .unwrap();

    // check the document body is somewhere in the response
    assert!(
        String::from_utf8(body.to_vec())
            .unwrap()
            .contains("Testing 123")
    );
}

#[test]

fn get_static_file() {
    let store = MemoryStore::default();
    let mut service = TamaWiki::new(store, "public/dist");

    let request = Request::get("/_static/css/main.css")
        .body(Body::from(""))
        .unwrap();

    hyper::rt::run(
        service
            .call(request)
            .map(|response| {
                assert_eq!(response.status(), StatusCode::OK);
            }).map_err(|err| panic!("{}", err)),
    );
}

#[test]
fn request_missing_page_with_edit_action() {
    let store = MemoryStore::default();
    let mut service = TamaWiki::new(store, "public/dist");

    let request = Request::get("/missing.html?action=edit")
        .body(Body::from(""))
        .unwrap();

    let response = service.call(request).wait().unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    let body = response
        .into_body()
        .fold(
            Vec::new(),
            |mut body, chunk| -> Result<Vec<u8>, hyper::Error> {
                body.append(&mut chunk.to_vec());
                Ok(body)
            },
        ).wait()
        .unwrap();

    assert!(
        String::from_utf8(body.to_vec())
            .unwrap()
            .contains("id=\"editor\"")
    );
}

#[test]
fn request_existing_page_with_edit_action() {
    let store = memorystore! {
        "example.html" => "test"
    };
    let mut service = TamaWiki::new(store, "public/dist");

    let request = Request::get("/example.html?action=edit")
        .body(Body::from(""))
        .unwrap();

    let response = service.call(request).wait().unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = response
        .into_body()
        .fold(
            Vec::new(),
            |mut body, chunk| -> Result<Vec<u8>, hyper::Error> {
                body.append(&mut chunk.to_vec());
                Ok(body)
            },
        ).wait()
        .unwrap();

    assert!(
        String::from_utf8(body.to_vec())
            .unwrap()
            .contains("id=\"editor\"")
    );
}
