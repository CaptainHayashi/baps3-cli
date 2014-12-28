#![feature(phase)]

extern crate baps3_protocol;
extern crate baps3_cli;
#[phase(plugin)] extern crate baps3_cli;

extern crate "rustc-serialize" as rustc_serialize;
extern crate docopt;
#[phase(plugin)] extern crate docopt_macros;

use std::io::{IoError, IoResult, IoErrorKind};
use std::os;
use std::path;

use baps3_cli::{one_shot, verbose_logger};
use baps3_protocol::proto::Message;

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
");

fn client(verbose: bool, target: &str, path: &str) -> IoResult<()> {
    let mut log = verbose_logger(verbose);
    one_shot(&mut log,
             target,
             &["FileLoad"],
             Message::new("load", &[path]))
}

fn io_err(desc: &'static str) -> IoError {
    IoError { kind: IoErrorKind::OtherIoError,
              desc: desc,
              detail: None }
}

/// Converts a potentially-relative path string to an absolute path string.
fn to_absolute_path_str(rel: &str) -> IoResult<String> {
    // This is a convoluted, entangled mess of Results and Options.
    // I sincerely apologise.
    path::Path::new_opt(rel)
      .ok_or_else(|| io_err("invalid path"))
      .and_then(|p| os::make_absolute(&p))
      .and_then(|ap| ap.as_str()
                       .map(|aps| aps.to_string())
                       .ok_or_else(|| io_err("non-utf8 path")))
}

fn main() {
    let args: Args = Args::docopt().decode().unwrap_or_else(|e| e.exit());

    to_absolute_path_str(&*args.arg_file)
      .and_then(|ap| client(args.flag_verbose, &*args.flag_target, &*ap))
      .unwrap_or_else(|e| werr!("error: {}", e));
}
