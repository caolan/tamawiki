#![deny(warnings)]
extern crate tamawiki;
extern crate futures;
extern crate warp;

use tamawiki::store::{Store, StoreClient};
use tamawiki::store::memory::MemoryStore;
use tamawiki::document::{Update, Operation, Insert};
use futures::future::Future;
use std::sync::{Arc, Mutex};
use std::path::PathBuf;


fn main() {
    let store = Arc::new(Mutex::new(MemoryStore::default()));
    let app = tamawiki::app(store.clone());

    let mut store = store.lock().unwrap().client();
    
    let push = store.push(
        PathBuf::from("index.html"),
        Update {
            from: 1,
            operations: vec![Operation::Insert(Insert {
                pos: 0,
                content: String::from("Hello, world!"),
            })]
        }
    );
    
    push.map_err(|err| panic!("{}", err))
        .wait()
        .unwrap();
    
    warp::serve(app).run(([127, 0, 0, 1], 8080));
}

