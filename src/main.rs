#[macro_use]
extern crate tamawiki;
#[macro_use]
extern crate clap;
extern crate futures;
extern crate hyper;

use futures::future::Future;
use hyper::Server;
use std::net::{IpAddr, SocketAddr};
use tamawiki::store::memory::MemoryStore;
use tamawiki::TamaWiki;

fn main() {
    let matches = clap_app!(TamaWiki =>
        (version: "0.1.0")
        (author: "Caolan McMahon")
        (about: "A wiki written in Rust")
        (@arg address: -a --address +takes_value "IP address to bind to")
        (@arg port: -p --port +takes_value "Port to bind to")
    ).get_matches();

    let address: IpAddr = matches
        .value_of("address")
        .unwrap_or("127.0.0.1")
        .parse()
        .expect("Invalid IP address");

    let port: u16 = matches
        .value_of("port")
        .unwrap_or("8080")
        .parse()
        .expect("Invalid port");

    let store = memorystore! {
        "index.html" => "Welcome to TamaWiki.\n"
    };

    let bind_to: SocketAddr = (address, port).into();

    let server = Server::bind(&bind_to)
        .serve(TamaWiki::new(store, "public/dist"))
        .map_err(|err| eprintln!("Server error: {}", err));

    println!("Server running at http://{}", bind_to);
    hyper::rt::run(server);
}
