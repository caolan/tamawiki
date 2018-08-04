#![deny(warnings)]
extern crate tamawiki;
extern crate warp;

use tamawiki::store::memory::MemoryStore;
use std::sync::{Arc, Mutex};


fn main() {
    let store = Arc::new(Mutex::new(MemoryStore::default()));
    let app = tamawiki::app(store.clone());
    
    warp::serve(app).run(([127, 0, 0, 1], 8080));
}

