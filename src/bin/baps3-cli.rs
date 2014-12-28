#![feature(phase)]

extern crate baps3_protocol;
extern crate libc;
extern crate docopt;
#[phase(plugin)] extern crate docopt_macros;

use std::borrow::ToOwned;

use baps3_protocol::client::{Client, Request, Response};
use baps3_protocol::proto::{Unpacker, Message};

fn commands() {
    println!("Commands: ");
    println!("  !c HOST:PORT - connect (if not connected)");
    println!("  !d           - disconnect (if connected)");
    println!("  !h           - this help message");
    println!("  !t           - report current time");
    println!("  !T           - toggle whether to report time");
    println!("  !q           - quit");
    println!("");
    println!("Anything not prefixed with ! is sent to the server.");
}

fn main() {
    let (int_request_tx, int_request_rx) = channel();

    std::thread::Thread::spawn(move || { stdin_loop(int_request_tx)})
                        .detach();

    println!("Currently disconnected.");
    println!("Type !h <newline> for command help");

    'l: for msg in int_request_rx.iter() {
        match msg {
            Request::Quit => break,
            Request::SendMessage(msg) => match msg.as_str_vec().as_slice() {
                ["!c", dest] => match Client::new(dest) {
                    Ok(client) => {
                        let quit = client_main_loop(
                            client,
                            &int_request_rx,
                        );
                        println!("Disconnected");

                        if quit { break 'l };
                    },
                    Err(e) => println!("{}", e)
                },
                ["!h"] => commands(),
                ["!q"] => return,
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
                println!("{}", e);
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

fn forward(tx: Sender<Request>, word: &str, args: &[&str]) {
    println!("> {} {}", word, args);
    tx.send(Request::SendMessage(Message::new(word, args)));
}

struct CliClient {
    /// The last time-stamp reported by the server.
    last_time: String,

    /// Whether to report time.
    report_time: bool
}
impl CliClient {
    /// Creates a new CliClient.
    fn new() -> CliClient {
        CliClient { last_time:   "0:00".to_owned(),
                    report_time: true }
    }

    /// Toggles whether to report time.
    fn toggle_time(&mut self) {
        self.report_time = !self.report_time;
        println!("i time reporting: {}",
                 if self.report_time { "on" } else { "off" });
    }

    /// Reports the current time.
    fn report_time(&self) {
        println!("T {}", self.last_time);
    }

    /// Handles a TIME notification for this CliClient.
    fn time(&mut self, t: &str) {
        if let Some(ti) = t.parse::<i64>() {
            let d = std::time::Duration::microseconds(ti);
            let s = format!("{}{:02}:{:02}",
                if 0 < d.num_hours() { format!("{}:", d.num_hours()) }
                else                 { String::new()                 },
                d.num_minutes() % 60,
                d.num_seconds() % 60);
            if s != self.last_time {
                self.last_time = s;
                if self.report_time { self.report_time() };
            }
        }
    }
}

/// Returns true if the outer main loop must exit.
fn client_main_loop(Client {
    request_tx,
    response_rx
}: Client, int_request_rx: &Receiver<Request>) -> bool {
    let mut state = CliClient::new();

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
                    ["!d"] => {
                        request_tx.send(Request::Quit);
                        return false;
                    },
                    ["!h"] => commands(),
                    ["!q"] => {
                        request_tx.send(Request::Quit);
                        return true;
                    }
                    ["!T"] => state.toggle_time(),
                    ["!t"] => state.report_time(),
                    [word, args..] => forward(request_tx.clone(), word, args),
                    [] => ()
                },
                Ok(req) => request_tx.send(req),
                Err(_) => return false
            }
        } else {
            match resh.recv_opt() {
                Ok(Response::Gone) => return false,
                Ok(Response::ClientError(e)) => {
                    println!("! {}", e);
                    return false;
                },
                Ok(Response::Message(m)) => match &*m.as_str_vec() {
                    [ "TIME", t ] => state.time(t),
                    [ word, args.. ] => println!("< {} {}", word, args),
                    [] => ()
                },
                Err(_) => {
                    return false;
                }
            }
        }
    }
}
