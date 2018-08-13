#[macro_use] extern crate tamawiki;
extern crate tokio_tungstenite;
extern crate tokio_tls;
extern crate futures;
extern crate hyper;
extern crate tokio;
extern crate url;

use futures::future::Future;
use futures::stream::Stream;
use hyper::server::Server;
use tokio::runtime::current_thread::Runtime;
use tokio_tungstenite::tungstenite::error::Error;
use tokio_tungstenite::WebSocketStream;
use tokio::net::TcpStream;
use tokio_tls::TlsStream;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use url::Url;

use tamawiki::TamaWiki;
use tamawiki::store::memory::MemoryStore;
use tamawiki::session::EditSessionManager;


fn connect_websocket(url: &str) ->
impl Future<Item=(Option<tokio_tungstenite::tungstenite::Message>,
                  WebSocketStream<tokio_tungstenite::stream::Stream<TcpStream, TlsStream<TcpStream>>>),
            Error=Error>
{
    let url = Url::parse(url).unwrap();
    tokio_tungstenite::connect_async(url)
        .map_err(|_| {
            panic!("Could not establish websocket connection")
        })
        .and_then(move |(ws, _)| {
            ws.into_future().map_err(|err| panic!(err))
        })
}

#[test]
fn connect_via_websocket() {
    let mut rt = Runtime::new().expect("new test runtime");

    // bind to port 0 to get random port assigned by OS
    let addr = ([127, 0, 0, 1], 0).into();
    
    let static_path = PathBuf::from("public");
    let store = memorystore! {
        "index.html" => "Welcome to TamaWiki.\n"
    };
    
    let server = Server::bind(&addr)
        .serve(TamaWiki {
            edit_sessions: Arc::new(Mutex::new(EditSessionManager::default())),
            static_path,
            store
        });

    // find out which port number we got
    let port = server.local_addr().port();

    let url = format!("ws://127.0.0.1:{}/index.html", port);
    let client = connect_websocket(&url)
        .map(|(msg, _ws)| {
            assert_eq!(
                msg.unwrap().into_text().unwrap(),
                "{\"Connected\":{\"id\":1}}"
            )
        });

    rt.spawn(server.map_err(|err| panic!("Server error: {}", err)));
    rt.block_on(client).unwrap();
}

#[test]
fn websocket_connections_get_different_ids() {
    let mut rt = Runtime::new().expect("new test runtime");

    // bind to port 0 to get random port assigned by OS
    let addr = ([127, 0, 0, 1], 0).into();
    
    let static_path = PathBuf::from("public");
    let store = memorystore! {
        "index.html" => "Welcome to TamaWiki.\n"
    };
    
    let server = Server::bind(&addr)
        .serve(TamaWiki {
            edit_sessions: Arc::new(Mutex::new(EditSessionManager::default())),
            static_path,
            store
        });

    // find out which port number we got
    let port = server.local_addr().port();

    let url1 = format!("ws://127.0.0.1:{}/index.html", port);
    let url2 = format!("ws://127.0.0.1:{}/other_page.html", port);
    
    let client1 = connect_websocket(&url1)
        .map(|(msg, _ws)| {
            assert_eq!(
                msg.unwrap().into_text().unwrap(),
                "{\"Connected\":{\"id\":1}}"
            )
        });

    let client2 = connect_websocket(&url1)
        .map(|(msg, _ws)| {
            assert_eq!(
                msg.unwrap().into_text().unwrap(),
                "{\"Connected\":{\"id\":2}}"
            )
        });

    // client 3 connects to a different page and should also get id=1
    let client3 = connect_websocket(&url2)
        .map(|(msg, _ws)| {
            assert_eq!(
                msg.unwrap().into_text().unwrap(),
                "{\"Connected\":{\"id\":1}}"
            )
        });

    rt.spawn(server.map_err(|err| {
        panic!("Server error: {}", err)
    }));
    
    rt.block_on(
        client1
            .and_then(|_| client2)
            .join(client3)
            .map(|_| ())
    ).unwrap();
}
