#[macro_use] extern crate tamawiki;
extern crate tungstenite;
extern crate futures;
extern crate hyper;
extern crate tokio;
extern crate url;

use tamawiki::TamaWiki;
use tamawiki::store::memory::MemoryStore;
use futures::future::Future;
use hyper::server::Server;
use std::path::PathBuf;

use tokio::runtime::Runtime;
use tokio::prelude::future::lazy;
use url::Url;


#[test]
fn connect_via_websocket() {
    let mut rt = Runtime::new().expect("test");

    // bind to port 0 to get random port assigned by OS
    let addr = ([127, 0, 0, 1], 0).into();
    
    let static_path = PathBuf::from("public");
    let store = memorystore! {
        "index.html" => "Welcome to TamaWiki.\n"
    };
    
    let server = Server::bind(&addr)
        .serve(TamaWiki {
            static_path,
            store
        });

    // find out which port number we got
    let port = server.local_addr().port();
    
    let client = lazy(move || {
        let url = format!("ws://127.0.0.1:{}/index.html?seq=0", port);
        tungstenite::connect(Url::parse(&url).unwrap()).map_err(|_| {
            "Could not establish websocket connection"
        })
    });

    rt.spawn(server.map_err(|err| panic!("Server error: {}", err)));
    rt.block_on(client).unwrap();
}
