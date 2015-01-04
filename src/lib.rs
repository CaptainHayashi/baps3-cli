//! Support library for BAPS3 command-line interfaces.
#![feature(phase)]
#![feature(macro_rules)]
#![feature(unboxed_closures)]

extern crate baps3_protocol;
extern crate docopt;
#[phase(plugin)] extern crate docopt_macros;

use std::borrow::ToOwned;
use std::error::{ Error, FromError };
use std::fmt::{ Show, Formatter };
use std::io::{ IoError, IoErrorKind, IoResult };
use std::io::net::ip::ToSocketAddr;
use baps3_protocol::client::{ Client, Request, Response };
use baps3_protocol::proto::Message;
use util::unslicify;

pub mod util;
pub mod time;

pub type Logger<'a> = |&str|:'a;
#[macro_export]
macro_rules! log(
    ($l:ident, $($arg:tt)*) => (
        let _ = (*$l)(&*format!($($arg)*));
    )
);

/// Error type for high-level BAPS3 client errors.
pub enum Baps3Error {
    /// A command failed.
    CmdFailed { advice: String },

    /// A command was invalid.
    CmdInvalid { advice: String },

    /// The server hung up while we were waiting for it to tell us something.
    HungUp,

    /// A path somewhere was invalid.
    InvalidPath { path: String },

    /// General IO error.
    Io { err: IoError },

    /// The server did not have the appropriate feature set.
    MissingFeatures { wanted: Vec<String>, have: Vec<String> },

    /// The server is not actually speaking the BAPS3 protocol.
    NotBaps3Server,

    /// We received a response from the server we weren't expecting.
    UnexpectedResponse { code:        String,
                         args:        Vec<String>,
                         expectation: String },
}

fn baps3_err_desc(err: &Baps3Error) -> &'static str {
    match *err {
        Baps3Error::CmdFailed          { .. } => "command failed",
        Baps3Error::CmdInvalid         { .. } => "command invalid",
        Baps3Error::HungUp                    => "server hung up",
        Baps3Error::InvalidPath        { .. } => "invalid path",
        Baps3Error::Io         { err: ref e } => e.desc,
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
            Baps3Error::CmdFailed   { advice: ref a } => Some(a.to_owned()),
            Baps3Error::CmdInvalid  { advice: ref a } => Some(a.to_owned()),
            Baps3Error::InvalidPath { path:   ref p } => Some(p.to_owned()),
            Baps3Error::Io          { err:    ref e } => e.detail.clone(),
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
impl FromError<Baps3Error> for IoError {
    fn from_error(err: Baps3Error) -> IoError {
        if let Baps3Error::Io { err: e } = err {
            e
        } else {
            IoError {
                kind: IoErrorKind::OtherIoError,
                desc: baps3_err_desc(&err),
                detail: err.detail()
            }
        }
    }
}
impl FromError<IoError> for Baps3Error {
    fn from_error(err: IoError) -> Baps3Error {
        Baps3Error::Io { err: err }
    }
}
impl Show for Baps3Error {
    fn fmt(&self, fmt: &mut Formatter) -> std::fmt::Result {
        fmt.pad(self.description())
           .and_then(|_| if let Some(details) = self.detail() {
            fmt.pad(": ").and_then(|_| fmt.pad(&*details))
        } else {
            Ok(())
        })
    }
}
pub type Baps3Result<A> = Result<A, Baps3Error>;

pub fn check_baps3(log: &mut Logger,
                   Client{request_tx, response_rx}: Client)
  -> Baps3Result<Client> {
    'l: loop {
        match response_rx.recv() {
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

/// Determines if a BAPS3 server is missing features needed by this client.
///
/// When performing a missing features check on a Client, prefer
/// `check_features`.
///
/// # Examples
///
/// This server is OK:
///
/// ```rust
/// use baps3_cli::missing_features;
/// assert!(!missing_features(&["PlayStop", "End"],
///                           &["PlayStop", "End", "FileLoad"]))
/// ```
///
/// However, this one is in trouble:
///
/// ```rust
/// use baps3_cli::missing_features;
/// assert!(missing_features(&["PlayStop", "End", "FileLoad"],
///                          &["PlayStop", "End"]))
/// ```
pub fn missing_features(needed: &[&str], have: &[&str]) -> bool {
    needed.iter().any(|n| !have.contains(n))
}

pub fn check_features(log: &mut Logger,
                      needed: &[&str],
                      Client{request_tx, response_rx}: Client)
  -> Baps3Result<(Client, Vec<String>)> {
    let mut vhave : Vec<String> = vec![];

    'l: loop {
        match response_rx.recv() {
            Ok(Response::Message(msg)) => match msg.as_str_vec().as_slice() {
                ["FEATURES", have..] => {
                    log!(log, "Server features: {}", have);
                    if missing_features(needed, have) {
                        return Err(Baps3Error::MissingFeatures {
                            wanted: unslicify(needed),
                            have: unslicify(have)
                        })
                    }

                    for h in have.iter() {
                        vhave.push((*h).to_owned());
                    }

                    break 'l;
                },
                [c, a..] => return Err(Baps3Error::UnexpectedResponse {
                    code: c.to_owned(),
                    args: unslicify(a),
                    expectation: "FEATURES".to_owned()
                }),
                [] => panic!("got empty slice from message")
            },
            _ => return Err(Baps3Error::HungUp)
        }
    }

    Ok(( Client { request_tx: request_tx,
                  response_rx: response_rx },
         vhave ))
}

pub fn send_command(log:    &mut Logger,
                    client: &mut Client,
                    msg:    &Message)
  -> Baps3Result<()> {
    let word = msg.word();
    let args = msg.args();
    log!(log, "Sending command: {} {}", word, args);

    client.request_tx.send(Request::SendMessage(msg.clone()));

    let result = wait_response(&client.response_rx, word, &*args);

    if let Ok(_) = result {
        log!(log, "success!");
    }

    result
}

fn wait_response(rx: &Receiver<Response>, word: &str, args: &[&str]) -> Baps3Result<()> {
    loop {
        match rx.recv() {
            Ok(Response::Message(msg)) => match msg.as_str_vec().as_slice() {
                ["OK", cword, cargs..]
                  if cword == word && cargs == args =>
                    return Ok(()),
                ["WHAT", advice, cword, cargs..]
                  if cword == word && cargs == args =>
                    return Err(Baps3Error::CmdInvalid { advice: advice.to_owned() }),
                ["FAIL", advice, cword, cargs..]
                  if cword == word && cargs == args =>
                    return Err(Baps3Error::CmdFailed { advice: advice.to_owned() }),
                _ => ()
            },
            _ => return Err(Baps3Error::HungUp)
        }
    }
}

pub fn quit_client(log: &mut Logger, Client { request_tx, .. }: Client)
  -> Baps3Result<()> {
    log!(log, "Closing client connection");

    request_tx.send(Request::Quit);
    Ok(())
}

pub struct Baps3<'a> {
    client:   Client,
    logger:   &'a mut Logger<'a>,
    features: Vec<String>
}

impl<'a> Baps3<'a> {
    /// Constructs a new Baps3.
    pub fn new<T>(logger:   &'a mut Logger<'a>,
                  addr:     T,
                  features: &[&str]) -> IoResult<Baps3<'a>>
    where T: ToSocketAddr {
        let ( client, all_features ) = try!(
            check_baps3(logger, try!(Client::new(addr)))
              .and_then(|c| check_features(logger, features, c))
        );

        Ok( Baps3 { client:   client,
                    logger:   logger,
                    features: all_features } )
    }

    /// Sends a command.
    /// Blocks until the command is acknowledged.
    pub fn send(&mut self, msg: &Message) -> Baps3Result<()> {
        send_command(self.logger, &mut self.client, msg)
    }

    pub fn quit(self) {
        self.client.request_tx.send(Request::Quit);
    }
}

/// A one-shot BAPS3 request.
///
/// This takes a server connection and performs the following:
///   - Checks that the server is a BAPS3 server, by seeing if an OHAI line is
///     being received;
///   - Checks the server's FEATURES flags against `features`, and fails if
///     any are missing;
///   - Sends the message `msg`;
///   - Reads until the server sends an OKAY, FAIL, or WHAT response for that
///     command.
pub fn one_shot<'a, T>(log: &'a mut Logger<'a>,
                       addr: T,
                       features: &[&str],
                       msg: Message) -> Baps3Result<()>
where T: ToSocketAddr {
    let mut b3  = try!(Baps3::new(log, addr, features));
    let res     = b3.send(&msg);
    b3.quit();

    res
}

/// Creates a Logger from the -v/--verbose flag of a command.
///
/// If verbose is on (-v/--verbose == true), we dump log messages to stderr,
/// else we ignore them.
pub fn verbose_logger<'a>(verbose: bool) -> Logger<'a> {
    if verbose { |s: &str| { let _ = std::io::stderr().write_line(s); } }
    else       { |_| {} }
}
