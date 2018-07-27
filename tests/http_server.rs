extern crate tamawiki;
extern crate futures;
extern crate actix;
extern crate actix_web;

use actix_web::{client, server, HttpMessage};
use std::path::PathBuf;
use futures::future::Future;
use actix::prelude::*;

use tamawiki::store::{Push};
use tamawiki::store::memory::MemoryStore;
use tamawiki::document::{Update, Operation, Insert};
use tamawiki::{app, State};


#[test]
fn get_missing_page() {
    let mut sys = System::new("test");
    let store: Addr<Syn, _> = MemoryStore::default().start();
    
    let srv = server::new(move || app::<MemoryStore>(State {
        store: store.clone(),
    }));
    
    // bind to port 0 to get random port assigned from OS
    let srv = srv.bind("127.0.0.1:0").unwrap();
        
    let base_url = {
        let (addr, scheme) = srv.addrs_with_scheme()[0];
        format!("{}://{}", scheme, addr)
    };
    
    let req = client::get(format!("{}/missing", base_url))
        .header("User-Agent", "Actix-web")
        .finish().unwrap()
        .send()
        .map(|response| {
            assert_eq!(response.status(), 404);
        });

    srv.start();
    sys.run_until_complete(
        req.map_err(|err| panic!("{}", err))
    ).unwrap();
}


#[test]
fn get_page_content_from_store() {
    let mut sys = System::new("test");
    let store: Addr<Syn, _> = MemoryStore::default().start();
    
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

    let srv = server::new(move || app::<MemoryStore>(State {
        store: store.clone(),
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
    sys.run_until_complete(
        push1
            .and_then(|_| push2)
            .and_then(|_| req.map_err(|err| panic!("{}", err)))
            .map_err(|err| panic!("{}", err))
    ).unwrap();
}
