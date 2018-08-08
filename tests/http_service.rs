#[macro_use]extern crate tamawiki;
extern crate hyper;
extern crate http;
extern crate futures;

use hyper::service::Service;
use hyper::Body;
use http::{Request, StatusCode};
use futures::future::Future;
use futures::stream::Stream;

use tamawiki::service::TamaWikiService;
use tamawiki::store::memory::MemoryStore;


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
    let store = memorystore! {
        "test.html" => "Testing 123"
    };
    let mut service = TamaWikiService {
        store: store.clone()
    };

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
