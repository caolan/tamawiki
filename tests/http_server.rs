extern crate tamawiki;
extern crate actix_web;

use actix_web::{http, test};


fn create_test_server() -> test::TestServer {
    test::TestServer::with_factory(|| {
        tamawiki::app(tamawiki::State {})
    })
}

#[test]
fn get_root_url() {
    let mut srv = create_test_server();
    let request = srv.client(http::Method::GET, "/").finish().unwrap();
    let response = srv.execute(request.send()).unwrap();

    assert!(response.status().is_success());
}
