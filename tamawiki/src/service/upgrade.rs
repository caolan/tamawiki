use hyper::{Body, Request, Response};
use http::header::{UPGRADE, SEC_WEBSOCKET_KEY, SEC_WEBSOCKET_VERSION};
use tungstenite::protocol;
use sha1::{Digest, Sha1};
use futures::{future, Future};
use hyper;
use http;
use base64;

use websocket::WebSocket;
use service::error::HttpError;


pub fn is_websocket_upgrade_request(req: &Request<Body>) -> bool {
    match req.headers().get(UPGRADE) {
        Some(value) => match value.to_str() {
            Ok(protocol) => "websocket".eq_ignore_ascii_case(protocol),
            // invalid ascii chars in header value
            Err(_) => false,
        },
        _ => false
    }
}

pub fn websocket_upgrade<F, U>(req: Request<Body>, fun: F) ->
    Box<Future<Item=Response<Body>, Error=HttpError> + Send>
where
    F: Fn(WebSocket) -> U + Sync + Send + 'static,
    U: Future<Item = (), Error = ()> + Send + 'static,
{
    let upgrade = is_websocket_upgrade_request(&req);
    let version = match req.headers().get(SEC_WEBSOCKET_VERSION) {
        Some(value) => match value.to_str() {
            Ok(protocol) => "13" == protocol,
            // invalid ascii chars in header value
            Err(_) => false,
        },
        _ => false
    };
    let key = req.headers().get(SEC_WEBSOCKET_KEY).map(|k| {
        let mut sha1 = Sha1::default();
        sha1.input(k.as_bytes());
        sha1.input(b"258EAFA5-E914-47DA-95CA-C5AB0DC85B11");
        base64::encode(&sha1.result())
    });

    if !upgrade || !version || key.is_none() {
        return Box::new(future::err(HttpError::BadRequest));
    }
    
    let on_upgrade = req
        .into_body()
        .on_upgrade()
        .map_err(|err| eprintln!("upgrade error: {}", err))
        .and_then(move |upgraded| {
            let io = protocol::WebSocket::from_raw_socket(
                upgraded,
                protocol::Role::Server,
                None
            );
            fun(WebSocket::from(io))
        });

    hyper::rt::spawn(on_upgrade);

    Box::new(
        future::ok(
            http::Response::builder()
                .status(101)
                .header("connection", "upgrade")
                .header("upgrade", "websocket")
                .header("sec-websocket-accept", key.unwrap().as_str())
                .body(Default::default())
                .unwrap()
        )
    )
}
