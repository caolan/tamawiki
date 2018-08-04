#![deny(warnings)]

#[cfg(test)]
#[macro_use] extern crate proptest;

#[macro_use] extern crate serde_derive;
extern crate serde;
extern crate futures;

pub mod connection;
pub mod document;
pub mod store;
