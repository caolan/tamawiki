#[macro_use] extern crate tamawiki;
extern crate futures;
extern crate hyper;

use tamawiki::TamaWiki;
use tamawiki::store::memory::MemoryStore;
use tamawiki::session::EditSessionManager;
use futures::future::Future;
use hyper::Server;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};


fn main() {
    let addr = ([127, 0, 0, 1], 8080).into();

    let store = memorystore! {
        "index.html" => "Welcome to TamaWiki.\n"
    };
    
    let static_path = PathBuf::from("public");
    
    let server = Server::bind(&addr)
        .serve(TamaWiki {
            edit_sessions: Arc::new(Mutex::new(EditSessionManager::default())),
            static_path,
            store
        })
        .map_err(|err| eprintln!("Server error: {}", err));
    
    hyper::rt::run(server);
}
