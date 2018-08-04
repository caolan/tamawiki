#![deny(warnings)]

#[macro_use] extern crate serde_derive;
extern crate serde;

#[cfg(test)]
#[macro_use] extern crate proptest;

pub mod connection;
pub mod document;
