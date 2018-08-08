//! A wiki implemented in Rust

#![warn(missing_docs)]

#[cfg(test)]
#[macro_use] extern crate proptest;

#[macro_use] extern crate serde_derive;
extern crate serde;
extern crate hyper;
extern crate futures;

pub mod service;
pub mod document;
pub mod store;
