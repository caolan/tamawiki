#[macro_use]
extern crate tamawiki;
#[macro_use]
extern crate serde_json;
extern crate futures;
extern crate hyper;
extern crate serde;
extern crate tokio;
extern crate tokio_tls;
extern crate tokio_tungstenite;
extern crate url;

use futures::future::Future;
use futures::sink::Sink;
use futures::stream::Stream;
use hyper::server::Server;
use serde_json::Value;
use tokio::net::TcpStream;
use tokio::runtime::current_thread::Runtime;
use tokio_tls::TlsStream;
use tokio_tungstenite::tungstenite::error::Error;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::WebSocketStream;
use url::Url;

use tamawiki::store::memory::MemoryStore;
use tamawiki::TamaWiki;

type WsStream = WebSocketStream<tokio_tungstenite::stream::Stream<TcpStream, TlsStream<TcpStream>>>;

fn read_message(ws: WsStream) -> impl Future<Item = (Option<Message>, WsStream), Error = Error> {
    ws.into_future()
        .map(|(msg, rest)| {
            println!("client received: {:?}", msg);
            (msg, rest)
        }).map_err(|err| panic!(err))
}

fn connect_websocket(url: &str) -> impl Future<Item = WsStream, Error = Error> {
    let url = Url::parse(url).unwrap();
    tokio_tungstenite::connect_async(url)
        .map(|(ws, _)| ws)
        .map_err(|_| panic!("Could not establish websocket connection"))
}

#[test]
fn connect_via_websocket() {
    let mut rt = Runtime::new().expect("new test runtime");

    // bind to port 0 to get random port assigned by OS
    let addr = ([127, 0, 0, 1], 0).into();
    let store = memorystore! {
        "index.html" => "Welcome to TamaWiki.\n"
    };
    let server = Server::bind(&addr).serve(TamaWiki::new(store, "public/dist"));

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
    let server = Server::bind(&addr).serve(TamaWiki::new(store, "public/dist"));

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

    rt.spawn(server.map_err(|err| panic!("Server error: {}", err)));

    rt.block_on(
        client1
            // thread through the `ws` streams so they don't get dropped
            // (and therefore close the websocket) before all three
            // clients are connected.
            .and_then(|ws1| client2.map(move |ws2| (ws1, ws2)))
            .join(client3)
            .map(|_| ()),
    ).unwrap();
}

#[test]
fn websocket_join_and_leave_notifications() {
    let mut rt = Runtime::new().expect("new test runtime");

    // bind to port 0 to get random port assigned by OS
    let addr = ([127, 0, 0, 1], 0).into();
    let store = MemoryStore::default();
    let server = Server::bind(&addr).serve(TamaWiki::new(store, "public/dist"));

    // find out which port number we got
    let port = server.local_addr().port();

    let client1 = connect_websocket(&format!("ws://127.0.0.1:{}/index.html?seq=0", port));
    let client2 = connect_websocket(&format!("ws://127.0.0.1:{}/index.html?seq=1", port));
    let client3 = connect_websocket(&format!("ws://127.0.0.1:{}/index.html?seq=2", port));

    rt.spawn(server.map_err(|err| panic!("Server error: {}", err)));

    rt.block_on(
        // connect client 1
        client1
            .and_then(|ws1| {
                // then connect client 2
                client2.map(|ws2| (ws1, ws2))
            }).and_then(|(ws1, ws2)| {
                // then connect client 3
                client3.map(|ws3| (ws1, ws2, ws3))
            }).and_then(|(ws1, ws2, ws3)| {
                let msgs1 = ws1.map(Message::into_text).map(Result::unwrap);
                let msgs2 = ws2.map(Message::into_text).map(Result::unwrap);
                let msgs3 = ws3.map(Message::into_text).map(Result::unwrap);

                // the disconnect order is dictated by the order the underlying stream is dropped
                // client2 reads only 3 messages then disconnects first
                // client3 waits for leave messages from client 2 then disconnects second
                // client1 waits for leave messages from clients 2 and 3, then disconnects last

                // read desired messages from all streams
                msgs1
                    .take(5)
                    .collect()
                    .join3(msgs2.take(2).collect(), msgs3.take(2).collect())
            }).map(|(msgs1, msgs2, msgs3)| {
                assert_eq!(
                    msgs1,
                    vec![
                        "{\"Connected\":{\"id\":1}}",
                        "{\"Event\":{\"client_seq\":0,\"seq\":2,\"event\":{\"Join\":{\"id\":2}}}}",
                        "{\"Event\":{\"client_seq\":0,\"seq\":3,\"event\":{\"Join\":{\"id\":3}}}}",
                        "{\"Event\":{\"client_seq\":0,\"seq\":4,\"event\":{\"Leave\":{\"id\":2}}}}",
                        "{\"Event\":{\"client_seq\":0,\"seq\":5,\"event\":{\"Leave\":{\"id\":3}}}}",
                    ]
                );
                assert_eq!(
                    msgs2,
                    vec![
                        "{\"Connected\":{\"id\":2}}",
                        "{\"Event\":{\"client_seq\":0,\"seq\":3,\"event\":{\"Join\":{\"id\":3}}}}",
                    ]
                );
                assert_eq!(
                    msgs3,
                    vec![
                        "{\"Connected\":{\"id\":3}}",
                        "{\"Event\":{\"client_seq\":0,\"seq\":4,\"event\":{\"Leave\":{\"id\":2}}}}",
                    ]
                );
            }).map(|_| ()),
    ).unwrap();
}

#[test]
fn websocket_edits() {
    let mut rt = Runtime::new().expect("new test runtime");

    // bind to port 0 to get random port assigned by OS
    let addr = ([127, 0, 0, 1], 0).into();
    let store = MemoryStore::default();
    let server = Server::bind(&addr).serve(TamaWiki::new(store, "public/dist"));

    // find out which port number we got
    let port = server.local_addr().port();

    let client1 = connect_websocket(&format!("ws://127.0.0.1:{}/index.html?seq=0", port));
    let client2 = connect_websocket(&format!("ws://127.0.0.1:{}/index.html?seq=1", port));

    rt.spawn(server.map_err(|err| panic!("Server error: {}", err)));

    let client1_receives = |expected: Value| {
        |(ws1, ws2)| {
            read_message(ws1).map(move |(msg, ws1)| {
                let msg: Value = serde_json::from_str(&msg.unwrap().into_text().unwrap()).unwrap();
                assert_eq!(msg, expected);
                (ws1, ws2)
            })
        }
    };

    let client2_receives = |expected: Value| {
        let expected = expected.to_owned();
        |(ws1, ws2)| {
            read_message(ws2).map(move |(msg, ws2)| {
                let msg: Value = serde_json::from_str(&msg.unwrap().into_text().unwrap()).unwrap();
                assert_eq!(msg, expected);
                (ws1, ws2)
            })
        }
    };

    let client1_sends = |msg: Value| {
        move |(ws1, ws2): (WsStream, WsStream)| {
            ws1.send(Message::text(msg.to_string()))
                .map(move |ws1| (ws1, ws2))
        }
    };

    let client2_sends = |msg: Value| {
        move |(ws1, ws2): (WsStream, WsStream)| {
            ws2.send(Message::text(msg.to_string()))
                .map(move |ws2| (ws1, ws2))
        }
    };

    rt.block_on(
        // connect client 1
        client1
            .and_then(|ws1| {
                // then connect client 2
                client2.map(|ws2| (ws1, ws2))
            }).and_then(client1_receives(json!({"Connected": {"id": 1}})))
            .and_then(client1_receives(json!({
                "Event": {
                    "client_seq": 0,
                    "seq": 2,
                    "event": {
                        "Join": {
                            "id": 2
                        }
                    }
                }
            }))).and_then(client2_receives(json!({"Connected": {"id": 2}})))
            .and_then(client1_sends(json!({
                "ClientEdit": {
                    "client_seq": 1,
                    "parent_seq": 2,
                    "operations": [{"Insert":{"pos":0,"content":"Hello"}}]
                }
            }))).and_then(client2_receives(json!({
                "Event": {
                    "seq": 3,
                    "client_seq": 0,
                    "event": {
                        "Edit": {
                            "author": 1,
                            "operations": [
                                {"Insert": {"pos":0, "content": "Hello"}}
                            ]
                        }
                    }
                }
            }))).and_then(client1_sends(json!({
                "ClientEdit": {
                    "client_seq": 2,
                    "parent_seq": 3,
                    "operations": [{"Insert":{"pos":0,"content":"="}}]
                }
            }))).and_then(client2_receives(json!({
                "Event": {
                    "seq": 4,
                    "client_seq": 0,
                    "event": {
                        "Edit": {
                            "author": 1,
                            "operations": [
                                {"Insert": {"pos":0, "content": "="}}
                            ]
                        }
                    }
                }
            }))).and_then(client2_sends(json!({
                "ClientEdit": {
                    "client_seq": 1,
                    "parent_seq": 3,
                    "operations": [{"Insert":{"pos":5,"content":", world"}}]
                }
            }))).and_then(client1_receives(json!({
                "Event": {
                    "seq": 5,
                    "client_seq": 2,
                    "event": {
                        "Edit": {
                            "author": 2,
                            "operations": [
                                {"Insert": {"pos":6, "content": ", world"}}
                            ]
                        }
                    }
                }
            }))).map(|_| ()),
    ).unwrap();
}
