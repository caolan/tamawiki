extern crate tamawiki;
extern crate futures;
extern crate actix;
extern crate actix_web;
extern crate serde_json;
extern crate tokio;

use tokio::prelude::*;
use tokio::timer::Delay;
use std::time::{Duration, Instant};

use actix_web::{ws, server};
use std::path::PathBuf;
use futures::future::Future;
use futures::stream::Stream;
use actix::prelude::*;

use tamawiki::{app, TamaWikiState};
use tamawiki::store::{Push};
use tamawiki::store::memory::MemoryStore;
use tamawiki::session::EditSessionManager;
use tamawiki::document::{Update, Operation, Insert};

use tamawiki::connection::{
    ServerMessage, Connected, Join, Leave, Change,
    ClientMessage, Edit
};

#[test]
fn websocket_connected_join_and_leave_notifications() {
    System::run(|| {
        let store = MemoryStore::default().start();
        let session_manager = {
            let store = store.clone();
            EditSessionManager::new(store).start()
        };
        
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

        let url = format!("{}/example.html?seq=0", base_url);
        
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

fn parse_server_message(raw: ws::Message) -> ServerMessage {
    if let ws::Message::Text(text) = raw {
        serde_json::from_str::<ServerMessage>(&text).unwrap()
    } else {
        panic!("Expected websocket Text message")
    }
}

fn read_message(args: (ws::ClientReader, ws::ClientWriter)) ->
    Box<Future<Item=(ServerMessage, (ws::ClientReader, ws::ClientWriter)), Error=()>>
{
    let (reader, writer) = args;
    Box::new(
        reader.into_future()
            // must complete within 1 second
            .deadline(
                Instant::now() + Duration::from_millis(1000)
            )
            .map(|(raw, reader)| {
                (parse_server_message(raw.unwrap()), (reader, writer))
            })
            .map_err(|err| {
                panic!("{:?}", err)
            })
    )
}

#[test]
fn websocket_gets_missing_past_updates_on_connect() {
    System::run(|| {
        let store = MemoryStore::default().start();
        let session_manager = {
            let store = store.clone();
            EditSessionManager::new(store).start()
        };
        
        let push1 = store.send(Push {
            path: PathBuf::from("test.html"),
            update: Update {
                from: 1,
                operations: vec![Operation::Insert(Insert {
                    pos: 0,
                    content: String::from("Hello, "),
                })]
            }
        });

        let push2 = store.send(Push {
            path: PathBuf::from("test.html"),
            update: Update {
                from: 1,
                operations: vec![Operation::Insert(Insert {
                    pos: 7,
                    content: String::from("world"),
                })]
            }
        });

        let push3 = store.send(Push {
            path: PathBuf::from("test.html"),
            update: Update {
                from: 1,
                operations: vec![Operation::Insert(Insert {
                    pos: 12,
                    content: String::from("!"),
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

        let url = format!("{}/test.html?seq=1", base_url);
        let client = ws::Client::new(&url).connect();
        
        srv.start();

        Arbiter::spawn(
            push1
                .and_then(|_| push2)
                .and_then(|_| push3).map_err(|err| panic!("{}", err))
                .and_then(|_| client).map_err(|err| panic!("{}", err))
                .and_then(read_message)
                .map(|(msg, streams)| {
                    assert_eq!(
                        msg,
                        ServerMessage::Connected(Connected {
                            id: 1,
                            participants: vec![],
                        })
                    );
                    streams
                })
                .and_then(read_message)
                .map(|(msg, streams)| {
                    assert_eq!(
                        msg,
                        ServerMessage::Change(Change {
                            seq: 2,
                            update: Update {
                                from: 1,
                                operations: vec![Operation::Insert(Insert {
                                    pos: 7,
                                    content: String::from("world"),
                                })]
                            }
                        })
                    );
                    streams
                })
                .and_then(read_message)
                .map(|(msg, _streams)| {
                    assert_eq!(
                        msg,
                        ServerMessage::Change(Change {
                            seq: 3,
                            update: Update {
                                from: 1,
                                operations: vec![Operation::Insert(Insert {
                                    pos: 12,
                                    content: String::from("!"),
                                })]
                            }
                        })
                    );
                    System::current().stop()
                })
                .map_err(|_| assert!(false))
        );
    });
}

// #[test]
// fn websocket_edits_and_change_notifications() {
//     System::run(|| {
//         let store = MemoryStore::default().start();
//         let session_manager = {
//             let store = store.clone();
//             EditSessionManager::new(store).start()
//         };
        
//         let srv = server::new(move || app::<MemoryStore>(TamaWikiState {
//             store: store.clone(),
//             session_manager: session_manager.clone(),
//         }));
        
//         // bind to port 0 to get random port assigned from OS
//         let srv = srv.bind("127.0.0.1:0").unwrap();
        
//         let base_url = {
//             let (addr, scheme) = srv.addrs_with_scheme()[0];
//             format!("{}://{}", scheme, addr)
//         };

//         let url = format!("{}/test.html?seq=1", base_url);
        
//         let client1 = ws::Client::new(&url).connect();
//         let client2 = ws::Client::new(&url).connect();
        
//         srv.start();

//         Arbiter::spawn(
//             client1.and_then(|(reader1, writer1)| {
//                 client2.map(|(reader2, writer2)| {
//                     (reader1, writer1, reader2, writer2)
//                 })
//             }).map_err(|err| {
//                 panic!("{}", err);
//             }).and_then(|(reader1, mut writer1, reader2, _writer2)| {
//                 read_message(reader1)
//                     .and_then(|(msg, _reader1)| {
//                         assert_eq!(
//                             msg,
//                             ServerMessage::Connected(Connected {
//                                 id: 1,
//                                 participants: vec![],
//                             })
//                         );
//                         read_message(reader2)
//                     })
//                     .and_then(move |(msg, reader2)| {
//                         assert_eq!(
//                             msg,
//                             ServerMessage::Connected(Connected {
//                                 id: 2,
//                                 participants: vec![1],
//                             })
//                         );
//                         writer1.text(
//                             serde_json::to_string(&ClientMessage::Edit(Edit {
//                                 seq: 0,
//                                 client_seq: 1,
//                                 operations: vec![
//                                     Operation::Insert(Insert {
//                                         pos: 0,
//                                         content: String::from("Hello")
//                                     })
//                                 ]
//                             })).unwrap()
//                         );
//                         read_message(reader2)
//                     })
//                     .map(|(msg, _reader2)| {
//                         assert_eq!(
//                             msg,
//                             ServerMessage::Change(Change {
//                                 seq: 1,
//                                 update: Update {
//                                     from: 1,
//                                     operations: vec![
//                                         Operation::Insert(Insert {
//                                             pos: 0,
//                                             content: String::from("Hello")
//                                         })
//                                     ]
//                                 }
//                             })
//                         );
//                         System::current().stop();
//                     })
//             }).map_err(|_| {
//                 assert!(false)
//             })
//         );
//     });
// }


#[test]
fn websocket_edits_and_change_notifications() {
    System::run(|| {
        let store = MemoryStore::default().start();
        let session_manager = {
            let store = store.clone();
            EditSessionManager::new(store).start()
        };
        
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

        let url = format!("{}/test.html?seq=1", base_url);
        
        let client1 = ws::Client::new(&url).connect()
            .map_err(|err| panic!("{}", err))
            .and_then(read_message)
            .map(|(msg, streams)| {
                assert_eq!(
                    msg,
                    ServerMessage::Connected(Connected {
                        id: 1,
                        participants: vec![],
                    })
                );
                streams
            });
            
        let client2 = ws::Client::new(&url).connect()
            .map_err(|err| panic!("{}", err))
            .and_then(read_message)
            .map(|(msg, streams)| {
                assert_eq!(
                    msg,
                    ServerMessage::Connected(Connected {
                        id: 2,
                        participants: vec![1],
                    })
                );
                streams
            });

        srv.start();

        Arbiter::spawn(
            client1.and_then(|streams1| {
                client2.map(|streams2| {
                    (streams1, streams2)
                })
            })
                .and_then(|((reader1, mut writer1), (reader2, writer2))| {
                    writer1.text(
                        serde_json::to_string(&ClientMessage::Edit(Edit {
                            seq: 0,
                            client_seq: 1,
                            operations: vec![
                                Operation::Insert(Insert {
                                    pos: 0,
                                    content: String::from("Hello")
                                })
                            ]
                        })).unwrap()
                    );
                    Delay::new(Instant::now() + Duration::from_millis(100))
                        .map(|_| ((reader1, writer1), (reader2, writer2)))
                        .map_err(|err| panic!("{}", err))
                })
                .and_then(|((reader1, writer1), (reader2, mut writer2))| {
                    writer2.text(
                        serde_json::to_string(&ClientMessage::Edit(Edit {
                            seq: 1,
                            client_seq: 1,
                            operations: vec![
                                Operation::Insert(Insert {
                                    pos: 5,
                                    content: String::from(", world")
                                })
                            ]
                        })).unwrap()
                    );
                    read_message((reader1, writer1))
                        .join(read_message((reader2, writer2)))
                })
                .and_then(|((msg1, streams1), (msg2, _streams2))| {
                    assert_eq!(
                        msg1,
                        ServerMessage::Join(Join {
                            id: 2
                        })
                    );
                    assert_eq!(
                        msg2,
                        ServerMessage::Change(Change {
                            seq: 1,
                            update: Update {
                                from: 1,
                                operations: vec![
                                    Operation::Insert(Insert {
                                        pos: 0,
                                        content: String::from("Hello")
                                    })
                                ]
                            }
                        })
                    );
                    read_message(streams1)
                })
                .map(|(msg1, _streams1)| {
                    assert_eq!(
                        msg1,
                        ServerMessage::Change(Change {
                            seq: 2,
                            update: Update {
                                from: 2,
                                operations: vec![
                                    Operation::Insert(Insert {
                                        pos: 5,
                                        content: String::from(", world")
                                    })
                                ]
                            }
                        })
                    );
                    System::current().stop();
                })
                .map_err(|_| {
                    assert!(false)
                })
        );
    });
}
