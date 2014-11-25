#![feature(if_let)]

extern crate baps3_protocol;
extern crate libc;

use baps3_protocol::Unpacker;
use client::{Client, Request, Response};

use libc::{close, STDIN_FILENO};

mod client;

fn main() {
    match Client::new("127.0.0.1:1350") {
        Ok(client) => client_main_loop(client),
        Err(e) => println!("{}", e)
    }
}

fn send_message(
    request_tx: &Sender<Request>,
    unpacker: &mut Unpacker,
    message: &str
) {
    for pline in unpacker.feed(message.as_slice()).iter() {
        if let [ref cmd, args..] = pline.as_slice() {
            println!("> {} {}", cmd, args);
            request_tx.send(Request::SendMessage(cmd.clone(), args.to_vec()));
        }
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
                Ok(l) => send_message(&request_tx, &mut u, l.as_slice()),
                Err(e) => {
                    println!("{}", e)
                    return;
                }
            }
        }

        request_tx.send(Request::Quit);
    });

    response_iter(response_rx);

    println!("Closing...");
    unsafe {
        close(STDIN_FILENO);
    }
}

fn response_iter(response_rx: Receiver<Response>) {
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