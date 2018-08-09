#[macro_use]extern crate tamawiki;
extern crate hyper;
extern crate http;
extern crate futures;

use hyper::service::Service;
use hyper::Body;
use http::{Request, StatusCode};
use futures::future::Future;
use futures::stream::Stream;
use std::path::PathBuf;

use tamawiki::service::TamaWikiService;
use tamawiki::store::memory::MemoryStore;


#[test]
fn get_missing_page() {
    let store = MemoryStore::default();
    let static_path = PathBuf::from("public");
    let mut service = TamaWikiService::new(store, &static_path.as_path());

    let request = Request::get("/missing.html")
        .body(Body::from(""))
        .unwrap();
    
    let response = service.call(request).wait().unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[test]
fn get_page_content_from_store() {
    let store = memorystore! {
        "test.html" => "Testing 123"
    };
    let static_path = PathBuf::from("public");
    let mut service = TamaWikiService::new(
        store.clone(),
        &static_path.as_path()
    );

    let request = Request::get("/test.html")
        .body(Body::from(""))
        .unwrap();
    
    let response = service.call(request).wait().unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().fold(
        Vec::new(),
        |mut body, chunk| -> Result<Vec<u8>, hyper::Error> {
            body.append(&mut chunk.to_vec());
            Ok(body)
        }
    ).wait().unwrap();
    
    // check the document body is somewhere in the response
    assert!(
        String::from_utf8(body.to_vec()).unwrap()
            .contains("Testing 123")
    );
}

#[test]
fn get_static_file() {
    let store = MemoryStore::default();
    let static_path = PathBuf::from("public");
    let mut service = TamaWikiService::new(store, &static_path.as_path());

    let request = Request::get("/_static/css/style.css")
        .body(Body::from(""))
        .unwrap();
    
    hyper::rt::run(
        service.call(request).map(|response| {
            assert_eq!(response.status(), StatusCode::OK);
        }).map_err(|err| panic!("{}", err))
    );
}
