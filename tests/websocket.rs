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
use tokio_tungstenite::tungstenite::Message;
use tokio::net::TcpStream;
use tokio_tls::TlsStream;
use url::Url;

use tamawiki::TamaWiki;
use tamawiki::store::memory::MemoryStore;

type WsStream = WebSocketStream<
    tokio_tungstenite::stream::Stream<TcpStream, TlsStream<TcpStream>>>;


fn read_message(ws: WsStream) ->
impl Future<Item=(Option<Message>, WsStream), Error=Error>
{
    ws.into_future().map(|(msg, rest)| {
        (msg, rest)
    }).map_err(|err| panic!(err))
}

fn connect_websocket(url: &str) -> impl Future<Item=WsStream, Error=Error> {
    let url = Url::parse(url).unwrap();
    tokio_tungstenite::connect_async(url)
        .map(|(ws, _)| ws)
        .map_err(|_| {
            panic!("Could not establish websocket connection")
        })
}

#[test]
fn connect_via_websocket() {
    let mut rt = Runtime::new().expect("new test runtime");

    // bind to port 0 to get random port assigned by OS
    let addr = ([127, 0, 0, 1], 0).into();
    let store = memorystore! {
        "index.html" => "Welcome to TamaWiki.\n"
    };
    let server = Server::bind(&addr)
        .serve(TamaWiki::new(store, "public"));

    // find out which port number we got
    let port = server.local_addr().port();

    let url = format!("ws://127.0.0.1:{}/index.html", port);
    let client = connect_websocket(&url)
        .and_then(read_message)
        .map(|(msg, _ws)| {
            assert_eq!(
                msg.unwrap().into_text().unwrap(),
                "{\"Connected\":{\"id\":1}}"
            );
        });

    rt.spawn(server.map_err(|err| panic!("Server error: {}", err)));
    rt.block_on(client).unwrap();
}

#[test]
fn websocket_connections_get_different_ids() {
    let mut rt = Runtime::new().expect("new test runtime");

    // bind to port 0 to get random port assigned by OS
    let addr = ([127, 0, 0, 1], 0).into();
    let store = memorystore! {
        "index.html" => "Welcome to TamaWiki.\n"
    };
    let server = Server::bind(&addr)
        .serve(TamaWiki::new(store, "public"));

    // find out which port number we got
    let port = server.local_addr().port();

    let url1 = format!("ws://127.0.0.1:{}/index.html", port);
    let url2 = format!("ws://127.0.0.1:{}/other_page.html", port);
    
    let client1 = connect_websocket(&url1)
        .and_then(read_message)
        .map(|(msg, ws)| {
            assert_eq!(
                msg.unwrap().into_text().unwrap(),
                "{\"Connected\":{\"id\":1}}"
            );
            ws
        });

    let client2 = connect_websocket(&url1)
        .and_then(read_message)
        .map(|(msg, ws)| {
            assert_eq!(
                msg.unwrap().into_text().unwrap(),
                "{\"Connected\":{\"id\":2}}"
            );
            ws
        });

    // client 3 connects to a different page and should also get id=1
    let client3 = connect_websocket(&url2)
        .and_then(read_message)
        .map(|(msg, ws)| {
            assert_eq!(
                msg.unwrap().into_text().unwrap(),
                "{\"Connected\":{\"id\":1}}"
            );
            ws
        });

    rt.spawn(server.map_err(|err| {
        panic!("Server error: {}", err)
    }));

    rt.block_on(
        client1
            // thread through the `ws` streams so they don't get dropped
            // (and therefore close the websocket) before all three
            // clients are connected.
            .and_then(|ws1| client2.map(move |ws2| (ws1, ws2)))
            .join(client3)
            .map(|_| ())
    ).unwrap();
}

#[test]
fn websocket_join_and_leave_notifications() {
    let mut rt = Runtime::new().expect("new test runtime");

    // bind to port 0 to get random port assigned by OS
    let addr = ([127, 0, 0, 1], 0).into();
    let store = MemoryStore::default();
    let server = Server::bind(&addr)
        .serve(TamaWiki::new(store, "public"));

    // find out which port number we got
    let port = server.local_addr().port();

    let client1 = connect_websocket(&format!("ws://127.0.0.1:{}/index.html?seq=0", port));
    let client2 = connect_websocket(&format!("ws://127.0.0.1:{}/index.html?seq=1", port));
    let client3 = connect_websocket(&format!("ws://127.0.0.1:{}/index.html?seq=2", port));

    rt.spawn(server.map_err(|err| {
        panic!("Server error: {}", err)
    }));
    
    rt.block_on(
        // connect client 1
        client1
            .and_then(|ws1| {
                // then connect client 2
                client2.map(|ws2| (ws1, ws2))
            })
            .and_then(|(ws1, ws2)| {
                // then connect client 3
                client3.map(|ws3| (ws1, ws2, ws3))
            })
            .and_then(|(ws1, ws2, ws3)| {
                let msgs1 = ws1.map(Message::into_text).map(Result::unwrap);
                let msgs2 = ws2.map(Message::into_text).map(Result::unwrap);
                let msgs3 = ws3.map(Message::into_text).map(Result::unwrap);

                // the disconnect order is dictated by the order the underlying stream is dropped
                // client2 reads only 3 messages then disconnects first
                // client3 waits for leave messages from client 2 then disconnects second
                // client1 waits for leave messages from clients 2 and 3, then disconnects last
                
                // read desired messages from all streams
                msgs1.take(6).collect()
                    .join3(msgs2.take(3).collect(),
                           msgs3.take(3).collect())
            })
            .map(|(msgs1, msgs2, msgs3)| {
                assert_eq!(msgs1, vec![
                    "{\"Connected\":{\"id\":1}}",
                    "{\"Join\":{\"seq\":1,\"id\":1}}",
                    "{\"Join\":{\"seq\":2,\"id\":2}}",
                    "{\"Join\":{\"seq\":3,\"id\":3}}",
                    "{\"Leave\":{\"seq\":4,\"id\":2}}",
                    "{\"Leave\":{\"seq\":5,\"id\":3}}",
                ]);
                assert_eq!(msgs2, vec![
                    "{\"Connected\":{\"id\":2}}",
                    "{\"Join\":{\"seq\":2,\"id\":2}}",
                    "{\"Join\":{\"seq\":3,\"id\":3}}",
                ]);
                assert_eq!(msgs3, vec![
                    "{\"Connected\":{\"id\":3}}",
                    "{\"Join\":{\"seq\":3,\"id\":3}}",
                    "{\"Leave\":{\"seq\":4,\"id\":2}}",
                ]);
            })
            .map(|_| ())
    ).unwrap();
}
