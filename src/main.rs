#![deny(warnings)]
extern crate warp;

use warp::Filter;

fn main() {
    // Match any request and return hello world!
    let routes = warp::any()
        .map(|| "Hello, World!\n");

    warp::serve(routes)
        .run(([127, 0, 0, 1], 8080));
}

