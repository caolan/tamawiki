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
use tamawiki::connection::{ConnectionMessage, Connected};
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
                operations: vec![Operation::Insert(Insert {
                    pos: 0,
                    content: String::from("Testing"),
                })]
            }
        });

        let push2 = store.send(Push {
            path: PathBuf::from("test.html"),
            update: Update {
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
                println!("{}", String::from_utf8(body.to_vec()).unwrap());
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
fn websocket_connections_get_different_ids() {
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

        let client1 = ws::Client::new(format!("{}/example.html", base_url))
            .connect()
            .and_then(|(reader, _writer)| {
                reader.into_future()
                    .map_err(|err| panic!("{:?}", err))
                    .map(|data| {
                        if let (Some(ws::Message::Text(text)), _reader) = data {
                            let msg = serde_json::from_str::<ConnectionMessage>(&text).unwrap();
                            assert_eq!(
                                msg,
                                ConnectionMessage::Connected(Connected {
                                    id: 1
                                })
                            );
                        } else {
                            assert!(false);
                        }
                    })
            });
        
        let client2 = ws::Client::new(format!("{}/example.html", base_url))
            .connect()
            .and_then(|(reader, _writer)| {
                reader.into_future()
                    .map_err(|err| panic!("{:?}", err))
                    .map(|data| {
                        if let (Some(ws::Message::Text(text)), _reader) = data {
                            let msg = serde_json::from_str::<ConnectionMessage>(&text).unwrap();
                            assert_eq!(
                                msg,
                                ConnectionMessage::Connected(Connected {
                                    id: 2
                                })
                            );
                        } else {
                            assert!(false);
                        }
                    })
            });
        
        srv.start();
        
        Arbiter::spawn(
            client1
                .and_then(|_| client2)
                .map_err(|err| panic!("{:?}", err))
                .map(|_| {
                    System::current().stop()
                })
        )
    });
}

#[test]
fn connect_using_websocket() {
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

        let client = ws::Client::new(format!("{}/example.html", base_url))
            .connect()
            .and_then(|(reader, _writer)| {
                reader.into_future()
                    .map_err(|err| panic!("{:?}", err))
                    .map(|data| {
                        if let (Some(ws::Message::Text(text)), _reader) = data {
                            let msg = serde_json::from_str::<ConnectionMessage>(&text).unwrap();
                            assert_eq!(
                                msg,
                                ConnectionMessage::Connected(Connected {
                                    id: 1
                                })
                            );
                        } else {
                            assert!(false);
                        }
                    })
            });
        
        srv.start();
        
        Arbiter::spawn(
            client
                .map_err(|err| panic!("{:?}", err))
                .map(|_| {
                    System::current().stop()
                })
        )
    });
}
