extern crate warp;
extern crate http;
extern crate futures;
extern crate tamawiki;

use http::StatusCode;
use std::sync::{Arc, Mutex};
use std::path::PathBuf;
use futures::future::Future;

use tamawiki::store::{Store, StoreClient};
use tamawiki::store::memory::{MemoryStore};
use tamawiki::document::{Update, Operation, Insert};


#[test]
fn get_missing_page() {
    let store = Arc::new(Mutex::new(MemoryStore::default()));
    let app = tamawiki::app(store.clone());
    
    let response = warp::test::request()
        .method("GET")
        .path("/missing.html")
        .reply(&app);

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[test]
fn get_page_content_from_store() {
    let store = Arc::new(Mutex::new(MemoryStore::default()));
    let app = tamawiki::app(store.clone());

    let mut store = store.lock().unwrap().client();
    
    let push1 = store.push(
        PathBuf::from("test.html"),
        Update {
            from: 1,
            operations: vec![Operation::Insert(Insert {
                pos: 0,
                content: String::from("Testing"),
            })]
        }
    );

    let push2 = store.push(
        PathBuf::from("test.html"),
        Update {
            from: 1,
            operations: vec![Operation::Insert(Insert {
                pos: 7,
                content: String::from(" 123"),
            })]
        }
    );

    // write document updates to store
    push1.and_then(|_| push2)
        .map_err(|err| panic!("{}", err))
        .wait()
        .unwrap();

    let response = warp::test::request()
        .method("GET")
        .path("/test.html")
        .reply(&app);

    assert_eq!(response.status(), StatusCode::OK);

    // check the document body is somewhere in the response
    assert!(
        String::from_utf8(response.body().to_vec()).unwrap()
            .contains("Testing 123")
    );
}

#[test]
fn request_static_file() {
    let store = Arc::new(Mutex::new(MemoryStore::default()));
    let app = tamawiki::app(store);
        
    let response = warp::test::request()
        .method("GET")
        .path("/_static/css/style.css")
        .reply(&app);

    assert_eq!(response.status(), StatusCode::OK);
}
