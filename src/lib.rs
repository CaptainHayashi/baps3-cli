#![feature(phase)]
#![feature(macro_rules)]
#![feature(unboxed_closures)]

extern crate baps3_protocol;
extern crate docopt;
#[phase(plugin)] extern crate docopt_macros;

use std::error::{Error, FromError};
use client::{Client, Request, Response};
use message::Message;
use util::unslicify;

pub mod client;
pub mod message;
pub mod util;

pub type Logger<'a> = |&str|:'a;
#[macro_export]
macro_rules! log(
    ($l:ident, $($arg:tt)*) => (
        let _ = (*$l)(&*format!($($arg)*));
    )
)

/// Error type for high-level BAPS3 client errors.
pub enum Baps3Error {
    /// The server hung up while we were waiting for it to tell us something.
    HungUp,

    /// The server did not have the appropriate feature set.
    MissingFeatures { wanted: Vec<String>, have: Vec<String> },

    /// The server is not actually speaking the BAPS3 protocol.
    NotBaps3Server,

    /// We received a response from the server we weren't expecting.
    UnexpectedResponse { code: String, args: Vec<String>, expectation: String }
}

fn baps3_err_desc(err: &Baps3Error) -> &'static str {
    match *err {
        Baps3Error::HungUp                    => "server hung up",
        Baps3Error::MissingFeatures    { .. } => "server missing features",
        Baps3Error::NotBaps3Server            => "not a BAPS3 server",
        Baps3Error::UnexpectedResponse { .. } => "unexpected response"
    }
}

impl Error for Baps3Error {
    fn description(&self) -> &str {
        baps3_err_desc(self)
    }

    fn detail(&self) -> Option<String> {
        match *self {
            Baps3Error::MissingFeatures { wanted: ref w, have: ref h }
                => Some(format!("wanted: {}, have: {}", w, h)),
            Baps3Error::UnexpectedResponse { code: ref c,
                                             args: ref a,
                                             expectation: ref e }
                => Some(format!("code: {}, args: {}; expected {}", c, a, e)),
            _ => None
        }
    }
}
impl FromError<Baps3Error> for std::io::IoError {
    fn from_error(err: Baps3Error) -> std::io::IoError {
        std::io::IoError {
            kind: std::io::IoErrorKind::OtherIoError,
            desc: baps3_err_desc(&err),
            detail: err.detail()
        }
    }
}
pub type Baps3Result<A> = Result<A, Baps3Error>;

pub fn check_baps3(log: &mut Logger,
                   Client{request_tx, response_rx}: Client)
  -> Baps3Result<Client> {
    'l: loop {
        match response_rx.recv_opt() {
            Ok(Response::Message(msg)) => match msg.as_str_vec().as_slice() {
                ["OHAI", ident] => {
                    log!(log, "Server ident: {}", ident);
                    break 'l;
                }
                _ => return Err(Baps3Error::NotBaps3Server)
            },
            _ => return Err(Baps3Error::HungUp)
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
  -> Baps3Result<Client> {
    'l: loop {
        match response_rx.recv_opt() {
            Ok(Response::Message(msg)) => match msg.as_str_vec().as_slice() {
                ["FEATURES", have..] => {
                    log!(log, "Server features: {}", have);

                    for n in needed.iter() {
                        if !have.contains(n) {
                            return Err(Baps3Error::MissingFeatures {
                                wanted: unslicify(needed),
                                have: unslicify(have)
                            })
                        }
                    }

                    break 'l;
                },
                [c, a..] => return Err(Baps3Error::UnexpectedResponse {
                    code: c.into_string(),
                    args: unslicify(a),
                    expectation: "FEATURES".into_string()
                }),
                [] => panic!("got empty slice from message")
            },
            _ => return Err(Baps3Error::HungUp)
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
  -> Baps3Result<Client> {
    log!(log, "Sending command: {} {}", word, args);

    request_tx.send(Request::SendMessage(Message::new(word, args)));

    'l: loop {
        match response_rx.recv_opt() {
            Ok(Response::Message(msg)) => match msg.as_str_vec().as_slice() {
                ["OK", cword, cargs..]
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
            },
            _ => return Err(Baps3Error::HungUp)
        }
    }

    Ok(Client{
        request_tx: request_tx,
        response_rx: response_rx
    })
}

pub fn quit_client(log: &mut Logger, Client{request_tx, ..}: Client)
  -> Baps3Result<()> {
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
pub fn one_shot<E: FromError<Baps3Error>>(log: &mut Logger,
                                          client: Client,
                                          features: &[&str],
                                          word: &str,
                                          args: &[&str])
  -> Result<(), E> {
    check_baps3(log, client)
      .and_then(|c| check_features(log, features, c))
      .and_then(|c| send_command(log, c, word, args))
      .and_then(|c| quit_client(log, c))
      .or_else(|e| Err(FromError::from_error(e)))
}

/// Creates a Logger from the -v/--verbose flag of a command.
///
/// If verbose is on (-v/--verbose == true), we dump log messages to stderr,
/// else we ignore them.
pub fn verbose_logger<'a>(verbose: bool) -> Logger<'a> {
    if verbose { |s: &str| { let _ = std::io::stderr().write_line(s); } }
    else       { |_| {} }
}