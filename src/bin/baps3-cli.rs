#![feature(phase)]

extern crate baps3_protocol;
extern crate baps3_cli;
extern crate libc;
extern crate docopt;
#[phase(plugin)] extern crate docopt_macros;

use baps3_protocol::Unpacker;
use baps3_cli::client::{Client, Request, Response};
use baps3_cli::message::Message;

fn main() {
    let (int_request_tx, int_request_rx) = channel();

    spawn(move || { stdin_loop(int_request_tx)});

    println!("Disconnected");

    for msg in int_request_rx.iter() {
        match msg {
            Request::Quit => break,
            Request::SendMessage(msg) => match msg.as_str_vec().as_slice() {
                ["connect", dest] => match Client::new(dest) {
                    Ok(client) => {
                        client_main_loop(
                            client,
                            &int_request_rx,
                        );
                        println!("Disconnected");
                    },
                    Err(e) => println!("{}", e)
                },
                _ => println!("can't do that, disconnected!")
            }
        }
    }

    println!("Quitting");
}

fn stdin_loop(
    request_tx: Sender<Request>
) {
    let mut u = Unpacker::new();

    for line in std::io::stdin().lock().lines() {
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
            request_tx.send(Request::SendMessage(Message::new(cmd, args)));
        }
    }
}

fn client_main_loop(Client {
    request_tx,
    response_rx
}: Client, int_request_rx: &Receiver<Request>) {
    let (int_response_tx, int_response_rx) = channel();

    spawn(move || { response_iter(int_response_rx) });

    let sel = std::comm::Select::new();

    let mut reqh = sel.handle(int_request_rx);
    unsafe { reqh.add(); }

    let mut resh = sel.handle(&response_rx);
    unsafe { resh.add(); }

    loop {
        let id = sel.wait();
        if id == reqh.id() {
            match reqh.recv_opt() {
                Ok(Request::SendMessage(msg)) =>
                  match msg.as_str_vec().as_slice() {
                    ["disconnect"] => {
                        request_tx.send(Request::Quit);
                        return;
                    },
                    [word, args..] => {
                        println!("> {} {}", word, args);
                        request_tx.send(Request::SendMessage(Message::new(word, args)));
                    },
                    [] => ()
                },
                Ok(req) => request_tx.send(req),
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
            Response::Message(m) => println!("< {} {}", m.word(), m.args()),
            Response::ClientError(e) => {
                println!("! {}", e);
                return;
            }
            Response::Gone => return
        }
    }
}