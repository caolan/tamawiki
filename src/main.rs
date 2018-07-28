extern crate tamawiki;
extern crate actix;

use actix::System;

const ADDR: &'static str = "localhost:8080";

fn main() {
    System::run(|| {
        let srv = tamawiki::server(ADDR);
    
        println!("TamaWiki running at:");
        for (addr, scheme) in &srv.addrs_with_scheme() {
            println!("  {}://{}", scheme, addr);
        }
        println!("");

        srv.start();
    });
}
