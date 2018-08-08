extern crate tamawiki;
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
use tamawiki::store::Store;
use tamawiki::document::{Update, Operation, Insert};


#[test]
fn get_missing_page() {
    let store = MemoryStore::default();
    let mut service = TamaWikiService {store};

    let request = Request::get("/missing.html")
        .body(Body::from(""))
        .unwrap();
    
    let response = service.call(request).wait().unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[test]
fn get_page_content_from_store() {
    let mut store = MemoryStore::default();
    let mut service = TamaWikiService {store: store.clone()};

    let push1 = store.push(
        PathBuf::from("test.html"),
        Update {
            author: 1,
            operations: vec![Operation::Insert(Insert {
                pos: 0,
                content: String::from("Testing"),
            })]
        }
    );

    let push2 = store.push(
        PathBuf::from("test.html"),
        Update {
            author: 1,
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
