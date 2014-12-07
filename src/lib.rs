#![feature(if_let)]
#![feature(phase)]
#![feature(macro_rules)]
#![feature(unboxed_closures)]

extern crate baps3_protocol;
extern crate docopt;
#[phase(plugin)] extern crate docopt_macros;

use std::io::IoResult;
use client::{Client, Request, Response};

pub mod client;
pub mod util;

/// Creates a vector of string-slices from a vector of strings.
///
/// The slice vector lives as long as the original vector.
pub fn slicify<'a>(vec: &'a Vec<String>) -> Vec<&'a str> {
    vec.iter().map(|x| x.as_slice()).collect::<Vec<&str>>()
}

pub fn slicify_msg<'a>(code: &'a String,
                       vec: &'a Vec<String>) -> Vec<&'a str> {
    let mut v = slicify(vec);
    v.insert(0, &**code);
    v
}

pub type Logger<'a> = |&str|:'a;
macro_rules! log(
    ($l:ident, $($arg:tt)*) => (
        let _ = (*$l)(&*format!($($arg)*));
    )
)

pub fn check_baps3(log: &mut Logger,
                   Client{request_tx, response_rx}: Client)
  -> IoResult<Client> {
    'l: loop {
        match response_rx.recv_opt() {
            Ok(Response::Message(code, msg)) => {
                match &*slicify_msg(&code, &msg) {
                    ["OHAI", ident] => {
                        log!(log, "Server ident: {}", ident);
                        break 'l;
                    }
                    _ => return Err(std::io::IoError {
                        kind: std::io::IoErrorKind::OtherIoError,
                        desc: "not a BAPS3 server",
                        detail: None
                    })
                }
            },
            _ => return Err(std::io::IoError {
                kind: std::io::IoErrorKind::OtherIoError,
                desc: "unexpected response",
                detail: None
            })
        }
    }

    Ok(Client{
        request_tx: request_tx,
        response_rx: response_rx
    })
}

pub fn check_features(log: &mut Logger,
                      needed: &[&str],
                      Client{request_tx, response_rx}: Client)
  -> IoResult<Client> {
    'l: loop {
        match response_rx.recv_opt() {
            Ok(Response::Message(code, msg)) => {
                match &*slicify_msg(&code, &msg) {
                    ["FEATURES", fs..] => {
                        log!(log, "Server features: {}", fs);

                        for n in needed.iter() {
                            if !fs.contains(n) {
                                return Err(std::io::IoError {
                                    kind: std::io::IoErrorKind::OtherIoError,
                                    desc: "insufficient features",
                                    detail: Some(format!("have: {} want: {}",
                                                         fs,
                                                         needed))
                                })
                            }
                        }

                        break 'l;
                    }
                    _ => return Err(std::io::IoError {
                        kind: std::io::IoErrorKind::OtherIoError,
                        desc: "expected FEATURES here",
                        detail: None
                    })
                }
            },
            _ => return Err(std::io::IoError {
                kind: std::io::IoErrorKind::OtherIoError,
                desc: "unexpected response",
                detail: None
            })
        }
    }

    Ok(Client{
        request_tx: request_tx,
        response_rx: response_rx
    })
}

pub fn send_command(log: &mut Logger,
                    Client{request_tx, response_rx}: Client,
                    word: &str, args: &[&str])
  -> IoResult<Client> {
    log!(log, "Sending command: {} {}", word, args);

    let oword = word.into_string();
    let oargs = args.iter().map(|arg| arg.into_string()).collect();
    request_tx.send(Request::SendMessage(oword, oargs));

    'l: loop {
        match response_rx.recv_opt() {
            Ok(Response::Message(code, msg)) => {
                match &*slicify_msg(&code, &msg) {
                    ["OKAY", cword, cargs..]
                      if cword == word && cargs == args => {
                        log!(log, "success!");
                        break 'l;
                    },
                    ["WHAT", advice, cword, cargs..]
                      if cword == word && cargs == args => {
                        werr!("command invalid: {}", advice);
                        break 'l;
                    },
                    ["FAIL", advice, cword, cargs..]
                      if cword == word && cargs == args => {
                        werr!("command failed: {}", advice);
                        break 'l;
                    },
                    _ => ()
                }
            },
            _ => return Err(std::io::IoError {
                kind: std::io::IoErrorKind::OtherIoError,
                desc: "unexpected response",
                detail: None
            })
        }
    }

    Ok(Client{
        request_tx: request_tx,
        response_rx: response_rx
    })
}

pub fn quit_client(log: &mut Logger, Client{request_tx, ..}: Client)
  -> IoResult<()> {
    log!(log, "Closing client connection");

    request_tx.send(Request::Quit);
    Ok(())
}

/// A one-shot BAPS3 request.
///
/// This takes a server connection and performs the following:
///   - Checks that the server is a BAPS3 server, by seeing if an OHAI line is
///     being received;
///   - Checks the server's FEATURES flags against `features`, and fails if
///     any are missing;
///   - Sends the command `word` `args`;
///   - Reads until the server sends an OKAY, FAIL, or WHAT response for that
///     command.
pub fn one_shot(log: &mut Logger,
            client: Client,
            features: &[&str],
            word: &str,
            args: &[&str])
  -> IoResult<()> {
    check_baps3(log, client)
      .and_then(|c| check_features(log, features, c))
      .and_then(|c| send_command(log, c, word, args))
      .and_then(|c| quit_client(log, c))
}

/// Creates a Logger from the -v/--verbose flag of a command.
///
/// If verbose is on (-v/--verbose == true), we dump log messages to stderr,
/// else we ignore them.
pub fn verbose_logger<'a>(verbose: bool) -> Logger<'a> {
    if verbose { |s: &str| { let _ = std::io::stderr().write_line(s); } }
    else       { |_| {} }
}