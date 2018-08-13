//! Utilities for processing HTTP requests

use hyper::Body;
use http::Request;
use serde_urlencoded;
use std::collections::HashMap;


/// Extracts a HashMap of query parameters from the request URL, if
/// there are no query parameters an empty HashMap is returned.
pub fn query_params(req: &Request<Body>) -> HashMap<String, String> {
    match req.uri().query() {
        Some(qs) => serde_urlencoded::from_str(qs).unwrap(),
        None => Default::default()
    }
}
