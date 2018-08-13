//! A wiki implemented in Rust

#![warn(missing_docs)]

#[cfg(test)]
#[macro_use] extern crate proptest;

#[macro_use] extern crate tera;
#[macro_use] extern crate lazy_static;
#[macro_use] extern crate serde_derive;
#[macro_use] extern crate serde_json;
extern crate serde_urlencoded;
extern crate serde;
extern crate hyper;
extern crate futures;
extern crate http;
extern crate hyper_staticfile;
extern crate tungstenite;
extern crate base64;
extern crate sha1;

pub mod service;
pub mod document;
pub mod store;
mod templates;
mod websocket;
mod session;

pub use service::TamaWiki;
