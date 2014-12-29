#![feature(phase)]

extern crate baps3_protocol;
extern crate baps3_cli;
#[phase(plugin)] extern crate baps3_cli;

extern crate "rustc-serialize" as rustc_serialize;
extern crate docopt;
#[phase(plugin)] extern crate docopt_macros;

use baps3_cli::{one_shot, verbose_logger};
use baps3_protocol::proto::Message;

docopt!(Args deriving Show, "
Plays the currently loaded file in a BAPS3 server.

Usage:
  baps3-play -h
  baps3-play [-v] [-t <target>]

Options:
  -h, --help             Show this message.
  -v, --verbose          Prints a trail of miscellaneous information
                         about the action.
  -t, --target <target>  The target BAPS3 server (host:port).
                         [Default: localhost:1350]
");

fn main() {
    let args: Args = Args::docopt().decode().unwrap_or_else(|e| e.exit());
    let mut log = verbose_logger(args.flag_verbose);

    one_shot(&mut log,
             &*args.flag_target,
             &["PlayStop"],
             Message::from_word("play"))
      .unwrap_or_else(|e| werr!("error: {}", e));
}
