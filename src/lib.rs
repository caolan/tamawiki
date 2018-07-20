//! A wiki implemented in Rust

extern crate actix_web;
extern crate actix;

use actix_web::{App, Responder, HttpRequest, HttpResponse, server, http};
use actix::System;


pub mod document;


/// Per-thread application state
pub struct State {}

/// Creates a new TamaWiki actix_web App
pub fn app(state: State) -> App<State> {
    App::with_state(state)
        .handler("/", request_handler)
}

fn request_handler(_req: HttpRequest<State>) -> impl Responder {
    HttpResponse::Ok()
        .header(http::header::CONTENT_TYPE, "text/html")
        .body("Hello, world!")
}

/// Start the TamaWiki HTTP server running on the given address
pub fn start(addr: &str) {
    let sys = System::new("tamawiki");
    server::new(|| app(State {}))
        .bind(addr)
        .unwrap()
        .start();

    println!("TamaWiki running at {}", addr);
    sys.run();
}
