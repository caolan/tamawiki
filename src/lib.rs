//! A wiki implemented in Rust

#![warn(missing_docs)]

#[cfg(test)]
#[macro_use]
extern crate proptest;

#[macro_use]
extern crate tera;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serde_json;
extern crate base64;
extern crate futures;
extern crate http;
extern crate hyper;
extern crate hyper_staticfile;
extern crate serde;
extern crate serde_urlencoded;
extern crate sha1;
extern crate tokio;
extern crate tungstenite;

pub mod document;
pub mod service;
pub mod session;
pub mod store;
mod templates;
mod websocket;

pub use service::TamaWiki;
