#[macro_use] extern crate tamawiki;
extern crate futures;
extern crate hyper;

use tamawiki::TamaWiki;
use tamawiki::store::memory::MemoryStore;
use futures::future::Future;
use hyper::Server;
use std::path::PathBuf;


fn main() {
    let addr = ([127, 0, 0, 1], 8080).into();

    let store = memorystore! {
        "index.html" => "Welcome to TamaWiki.\n"
    };
    
    let static_path = PathBuf::from("public");
    
    let server = Server::bind(&addr)
        .serve(TamaWiki {
            static_path,
            store
        })
        .map_err(|err| eprintln!("Server error: {}", err));
    
    hyper::rt::run(server);
}
