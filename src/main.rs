#![feature(if_let)]

extern crate baps3_protocol;
use baps3_protocol::Unpacker;
use client::{Client, Request, Response};

mod client;

fn main() {
    match Client::new("127.0.0.1:1350") {
        Ok(client) => client_main_loop(client),
        Err(e) => println!("{}", e)
    }
}

fn client_main_loop(Client {
    request_tx,
    response_rx
}: Client) {
    spawn(proc() {
        let mut u = Unpacker::new();

        for line in std::io::stdin().lines() {
            match line {
                Ok(l) => for pline in u.feed(l.as_slice()).iter() {
                    if let [ref cmd, args..] = pline.as_slice() {
                        println!("> {} {}", cmd, args);
                        request_tx.send(Request::SendMessage(cmd.clone(),
                                                             args.to_vec()));
                    }
                },
                Err(e) => println!("{}", e)
            }
        }

        request_tx.send(Request::Quit);
    });

    for m in response_rx.iter() {
        match m {
            Response::Message(cmd, args) => println!("< {} {}", cmd, args),
            Response::ClientError(e) => {
                println!("! {}", e);
                return;
            }
            Response::Gone => return
        }
    }
}