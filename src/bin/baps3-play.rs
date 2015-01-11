#![feature(plugin)]

extern crate baps3_protocol;
#[macro_use] extern crate baps3_cli;

extern crate "rustc-serialize" as rustc_serialize;
extern crate docopt;
#[plugin] #[no_link] extern crate docopt_macros;

use baps3_cli::{one_shot, verbose_logger};
use baps3_protocol::proto::Message;

docopt!(Args, "
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
    let log = |&:s:&str| verbose_logger(args.flag_verbose, s);

    one_shot(log,
             &*args.flag_target,
             &["PlayStop"],
             Message::from_word("play"))
      .unwrap_or_else(|e| werr!("error: {}", e));
}
