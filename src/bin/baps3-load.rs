#![feature(if_let)]
#![feature(phase)]
#![feature(unboxed_closures)]
#![feature(macro_rules)]

extern crate baps3_protocol;
#[phase(plugin)] extern crate baps3_cli;
extern crate baps3_cli;
extern crate docopt;
extern crate serialize;
#[phase(plugin)] extern crate docopt_macros;

use std::io::IoResult;

use baps3_cli::client::{Client, Request, Response};

docopt!(Args deriving Show, "
Loads a file into a BAPS3 server.

Usage:
  baps3-load [options] <file>

Options:
  -h, --help             Show this message.
  -t, --target <target>  The target BAPS3 server (host:port).
                         [Default: localhost:1350]
  -v, --verbose          Prints a trail of miscellaneous information
                         about the action.
")

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

type Logger<'a> = |&str|:'a;
macro_rules! log(
    ($l:ident, $($arg:tt)*) => (
        let _ = (*$l)(&*format!($($arg)*));
    )
)

fn check_baps3(log: &mut Logger, Client{request_tx, response_rx}: Client)
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

fn check_features(log: &mut Logger,
                  Client{request_tx, response_rx}: Client)
  -> IoResult<Client> {
    'l: loop {
        match response_rx.recv_opt() {
            Ok(Response::Message(code, msg)) => {
                match &*slicify_msg(&code, &msg) {
                    ["FEATURES", fs..] => {
                        log!(log, "Server features: {}", fs);
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

fn send_command(log: &mut Logger,
                Client{request_tx, response_rx}: Client,
                word: &str, args: &[&str]) -> IoResult<Client> {
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

fn quit_client(log: &mut Logger, Client{request_tx, ..}: Client)
  -> IoResult<()> {
    log!(log, "Closing client connection");

    request_tx.send(Request::Quit);
    Ok(())
}

fn main() {
    let args: Args = Args::docopt().decode().unwrap_or_else(|e| e.exit());

    let mut log: Logger = if args.flag_verbose {
        |s: &str| { let _ = std::io::stderr().write_line(s); }
    } else {
        |_| {}
    };

    Client
     ::new(&*args.flag_target)
      .and_then(|c| check_baps3(&mut log, c))
      .and_then(|c| check_features(&mut log, c))
      .and_then(|c| send_command(&mut log, c, "load", &[&*args.arg_file]))
      .and_then(|c| quit_client(&mut log, c))
      .unwrap_or_else(|e| werr!("error: {}", e));
}