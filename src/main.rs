extern crate tamawiki;
extern crate futures;
extern crate hyper;

use tamawiki::service::TamaWiki;
use futures::future::Future;
use hyper::Server;


fn main() {
    let addr = ([127, 0, 0, 1], 8080).into();

    let server = Server::bind(&addr)
        .serve(TamaWiki::default())
        .map_err(|err| eprintln!("Server error: {}", err));
    
    hyper::rt::run(server);
}
