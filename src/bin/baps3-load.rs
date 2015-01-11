#![feature(plugin)]

extern crate baps3_protocol;
#[macro_use] extern crate baps3_cli;

extern crate "rustc-serialize" as rustc_serialize;
extern crate docopt;
#[plugin] #[no_link] extern crate docopt_macros;

use std::borrow::ToOwned;
use std::os;
use std::path;

use baps3_cli::{ Baps3, Baps3Error, Baps3Result, verbose_logger };
use baps3_protocol::proto::Message;

docopt!(Args, "
Loads a file into a BAPS3 server.

Usage:
  baps3-load -h
  baps3-load [-pv] [-t <target>] <file>

Options:
  -h, --help             Show this message.
  -p, --play             If set, play the file upon loading.
  -v, --verbose          Prints a trail of miscellaneous information
                         about the action.
  -t, --target <target>  The target BAPS3 server (host:port).
                         [Default: localhost:1350]
");

fn load(Args { arg_file,
               flag_play,
               flag_target,
               flag_verbose, .. }: Args) -> Baps3Result<()> {
    let ap        = try!(to_absolute_path_str(&*arg_file));
    let log       = |&:s:&str| verbose_logger(flag_verbose, s);
    let mut baps3 = try!(Baps3::new(log, &*flag_target,
        &*(if flag_play { vec!["FileLoad", "PlayStop"] }
           else         { vec!["FileLoad"]             })));

    try!(baps3.send(&Message::new("load", &[&*ap])));

    if flag_play {
        try!(baps3.send(&Message::from_word("play")));
    }

    Ok(())
}

/// Converts a potentially-relative path string to an absolute path string.
fn to_absolute_path_str(rel: &str) -> Baps3Result<String> {
    // This is a convoluted, entangled mess of Results and Options.
    // I sincerely apologise.

    let badpath = |&:| Baps3Error::InvalidPath { path: rel.to_owned() };

    path::Path::new_opt(rel)
      .ok_or(badpath())
      .and_then(|&:p| os::make_absolute(&p).map_err(|_| badpath()))
      .and_then(|&:ap| ap.as_str().map(|&:s| s.to_string()).ok_or(badpath()))
}

fn main() {
    let args: Args = Args::docopt().decode().unwrap_or_else(|e| e.exit());
    load(args).unwrap_or_else(|&:e| werr!("error: {}", e));
}
