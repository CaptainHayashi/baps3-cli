#![feature(if_let)]

extern crate baps3_protocol;
extern crate libc;

use baps3_protocol::Unpacker;
use client::{Client, Request, Response};

mod client;

fn main() {
    let (int_request_tx, int_request_rx) = channel();

    spawn(proc() { stdin_loop(int_request_tx)});

    println!("Disconnected");

    for msg in int_request_rx.iter() {
        match msg {
            Request::Quit => break,
            Request::SendMessage(cmd, args) => {
                if cmd.as_slice() == "connect" {
                    if args.len() == 1 {
                        match Client::new(args[0].as_slice()) {
                            Ok(client) => {
                                client_main_loop(
                                    client,
                                    &int_request_rx,
                                );
                                println!("Disconnected");
                            },
                            Err(e) => println!("{}", e)
                        }
                    }
                } else {
                    println!("can't do that, disconnected!");
                }
            }
        }
    }

    println!("Quitting");
}

fn stdin_loop(
    request_tx: Sender<Request>
) {
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
}

fn send_message(
    request_tx: &Sender<Request>,
    unpacker: &mut Unpacker,
    message: &str
) {
    for pline in unpacker.feed(message.as_slice()).iter() {
        if let [ref cmd, args..] = pline.as_slice() {
            request_tx.send(Request::SendMessage(cmd.clone(), args.to_vec()));
        }
    }
}

fn client_main_loop(Client {
    request_tx,
    response_rx
}: Client, int_request_rx: &Receiver<Request>) {
    let (int_response_tx, int_response_rx) = channel();

    spawn(proc() { response_iter(int_response_rx) });

    let sel = std::comm::Select::new();

    let mut reqh = sel.handle(int_request_rx);
    unsafe { reqh.add(); }

    let mut resh = sel.handle(&response_rx);
    unsafe { resh.add(); }

    loop {
        let id = sel.wait();
        if id == reqh.id() {
            match reqh.recv_opt() {
                Ok(Request::SendMessage(ref cmd, ref args))
                    if cmd.as_slice() == "disconnect" &&
                       args.len() == 0 => {
                    request_tx.send(Request::Quit);
                    return;
                },
                Ok(req) => {
                    if let Request::SendMessage(ref cmd, ref args) = req {
                        println!("> {} {}", cmd, args);
                    }
                    request_tx.send(req);
                },
                Err(_) => return
            }
        } else {
            match resh.recv_opt() {
                Ok(Response::Gone) => {
                    int_response_tx.send(Response::Gone);
                    return;
                },
                Ok(r) => int_response_tx.send(r),
                Err(_) => {
                    int_response_tx.send(Response::Gone);
                    return;
                }
            }
        }
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