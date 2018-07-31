extern crate tamawiki;
extern crate futures;
extern crate actix;
extern crate actix_web;
extern crate serde_json;

use actix_web::{ws, client, server, HttpMessage};
use std::path::PathBuf;
use futures::future::Future;
use futures::stream::Stream;
use actix::prelude::*;

use tamawiki::store::{Push};
use tamawiki::store::memory::MemoryStore;
use tamawiki::session::EditSessionManager;
use tamawiki::document::{Update, Operation, Insert};
use tamawiki::connection::{ServerMessage, Connected, Join, Leave};
use tamawiki::{app, TamaWikiState};


#[test]
fn get_missing_page() {
    System::run(|| {
        let store = MemoryStore::default().start();
        let session_manager = EditSessionManager::default().start();
        
        let srv = server::new(move || app::<MemoryStore>(TamaWikiState {
            store: store.clone(),
            session_manager: session_manager.clone(),
        }));
        
        // bind to port 0 to get random port assigned from OS
        let srv = srv.bind("127.0.0.1:0").unwrap();
        
        let base_url = {
            let (addr, scheme) = srv.addrs_with_scheme()[0];
            format!("{}://{}", scheme, addr)
        };
        
        srv.start();
        
        let req = client::get(format!("{}/missing", base_url))
            .header("User-Agent", "Actix-web")
            .finish().unwrap()
            .send()
            .map(|response| {
                assert_eq!(response.status(), 404);
            })
            .map_err(|err| {
                panic!("{}", err)
            });

        Arbiter::spawn(
            req.map(|_| System::current().stop())
        );
    });
}


#[test]
fn get_page_content_from_store() {
    System::run(|| {
        let store = MemoryStore::default().start();
        let session_manager = EditSessionManager::default().start();
        
        let push1 = store.send(Push {
            path: PathBuf::from("test.html"),
            update: Update {
                from: 1,
                operations: vec![Operation::Insert(Insert {
                    pos: 0,
                    content: String::from("Testing"),
                })]
            }
        });

        let push2 = store.send(Push {
            path: PathBuf::from("test.html"),
            update: Update {
                from: 1,
                operations: vec![Operation::Insert(Insert {
                    pos: 7,
                    content: String::from(" 123"),
                })]
            }
        });

        let srv = server::new(move || app::<MemoryStore>(TamaWikiState {
            store: store.clone(),
            session_manager: session_manager.clone(),
        }));
        
        // bind to port 0 to get random port assigned from OS
        let srv = srv.bind("127.0.0.1:0").unwrap();
        
        let base_url = {
            let (addr, scheme) = srv.addrs_with_scheme()[0];
            format!("{}://{}", scheme, addr)
        };

        let req = client::get(format!("{}/test.html", base_url))
            .header("User-Agent", "Actix-web")
            .finish().unwrap()
            .send()
            .and_then(|response| {
                response.body().map_err(|err| panic!("{}", err))
            })
            .map(|body| {
                assert!(
                    String::from_utf8(body.to_vec()).unwrap()
                        .contains("Testing 123")
                );
            });

        srv.start();
        Arbiter::spawn(
            push1
                .and_then(|_| push2)
                .and_then(|_| req.map_err(|err| panic!("{}", err)))
                .map_err(|err| panic!("{}", err))
                .map(|_| System::current().stop())
        )
    });
}

#[test]
fn request_static_file() {
    System::run(|| {
        let store = MemoryStore::default().start();
        let session_manager = EditSessionManager::default().start();
        
        let srv = server::new(move || app::<MemoryStore>(TamaWikiState {
            store: store.clone(),
            session_manager: session_manager.clone(),
        }));
        
        // bind to port 0 to get random port assigned from OS
        let srv = srv.bind("127.0.0.1:0").unwrap();
        
        let base_url = {
            let (addr, scheme) = srv.addrs_with_scheme()[0];
            format!("{}://{}", scheme, addr)
        };

        let req = client::get(format!("{}/_static/css/style.css", base_url))
            .header("User-Agent", "Actix-web")
            .finish().unwrap()
            .send()
            .map(|response| {
                assert_eq!(response.status(), 200);
            });

        srv.start();
        
        Arbiter::spawn(
            req.map_err(|err| {
                panic!("{}", err)
            }).map(|_| {
                System::current().stop()
            })
        )
    });
}

#[test]
fn request_missing_page_with_edit_param() {
    // should return a page including an editor element
    System::run(|| {
        let store = MemoryStore::default().start();
        let session_manager = EditSessionManager::default().start();
        
        let srv = server::new(move || app::<MemoryStore>(TamaWikiState {
            store: store.clone(),
            session_manager: session_manager.clone(),
        }));
        
        // bind to port 0 to get random port assigned from OS
        let srv = srv.bind("127.0.0.1:0").unwrap();
        
        let base_url = {
            let (addr, scheme) = srv.addrs_with_scheme()[0];
            format!("{}://{}", scheme, addr)
        };

        let req = client::get(format!("{}/example.html?edit=true", base_url))
            .header("User-Agent", "Actix-web")
            .finish().unwrap()
            .send()
            .and_then(|response| {
                response.body().map_err(|err| panic!("{}", err))
            })
            .map(|body| {
                // TODO: this isn't a great test, it should probably
                // be more robust
                assert!(
                    String::from_utf8(body.to_vec()).unwrap()
                        .contains("id=\"editor\"")
                );
            });

        srv.start();
        
        Arbiter::spawn(
            req.map_err(|err| {
                panic!("{}", err)
            }).map(|_| {
                System::current().stop()
            })
        )
    });
}

#[test]
fn websocket_connected_join_and_leave_notifications() {
    System::run(|| {
        let store = MemoryStore::default().start();
        let session_manager = EditSessionManager::default().start();
        
        let srv = server::new(move || app::<MemoryStore>(TamaWikiState {
            store: store.clone(),
            session_manager: session_manager.clone(),
        }));
        
        // bind to port 0 to get random port assigned from OS
        let srv = srv.bind("127.0.0.1:0").unwrap();
        
        let base_url = {
            let (addr, scheme) = srv.addrs_with_scheme()[0];
            format!("{}://{}", scheme, addr)
        };

        let url = format!("{}/example.html", base_url);
        
        let client1 = ws::Client::new(&url).connect().into_stream();
        let client2 = ws::Client::new(&url).connect().into_stream();
        let client3 = ws::Client::new(&url).connect().into_stream();

        let streams = client1.chain(client2).chain(client3).collect();
        
        srv.start();
        
        Arbiter::spawn(
            streams
                .map_err(|err| panic!("{:?}", err))
                .and_then(|mut streams| {
                    let (_reader2, mut writer2) = streams.pop().unwrap();
                    let (reader1, _writer1) = streams.pop().unwrap();
                    let (reader0, _writer0) = streams.pop().unwrap();
                    // all clients should have joined at this point, 
                    // disconnect client 3 by closing write stream
                    writer2.close(Some(ws::CloseReason {
                        code: ws::CloseCode::Normal,
                        description: None,
                    }));
                    // collect expected data from first two read streams
                    reader0.take(4).collect().join(
                        reader1.take(3).collect()
                    )
                })
                .map(|(data0, data1)| {
                    let parse = |data| {
                        if let ws::Message::Text(text) = data {
                            serde_json::from_str::<ServerMessage>(&text).unwrap()
                        } else {
                            panic!("Expected websocket Text message")
                        }
                    };
                    
                    let msgs0 = data0.into_iter()
                        .map(parse).collect::<Vec<ServerMessage>>();
                    
                    let msgs1 = data1.into_iter()
                        .map(parse).collect::<Vec<ServerMessage>>();

                    assert_eq!(
                        msgs0,
                        vec![
                            ServerMessage::Connected(Connected {
                                id: 1,
                                participants: vec![],
                            }),
                            ServerMessage::Join(Join {
                                id: 2,
                            }),
                            ServerMessage::Join(Join {
                                id: 3,
                            }),
                            ServerMessage::Leave(Leave {
                                id: 3,
                            }),
                        ]
                    );
                    
                    assert_eq!(
                        msgs1,
                        vec![
                            ServerMessage::Connected(Connected {
                                id: 2,
                                participants: vec![1],
                            }),
                            ServerMessage::Join(Join {
                                id: 3,
                            }),
                            ServerMessage::Leave(Leave {
                                id: 3,
                            }),
                        ]
                    );
                })
                .map_err(|err| panic!("{:?}", err))
                .map(|_| {
                    System::current().stop()
                })
        )
    });
}
