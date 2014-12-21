#![feature(phase)]

extern crate baps3_protocol;
extern crate baps3_cli;
#[phase(plugin)] extern crate baps3_cli;
extern crate docopt;
extern crate serialize;
#[phase(plugin)] extern crate docopt_macros;

use baps3_cli::{one_shot, verbose_logger};
use baps3_cli::client::Client;
use baps3_cli::message::Message;

docopt!(Args deriving Show, "
Stops the currently playing file in a BAPS3 server.

Usage:
  baps3-stop [options]

Options:
  -h, --help             Show this message.
  -t, --target <target>  The target BAPS3 server (host:port).
                         [Default: localhost:1350]
  -v, --verbose          Prints a trail of miscellaneous information
                         about the action.
");

fn main() {
    let args: Args = Args::docopt().decode().unwrap_or_else(|e| e.exit());
    let mut log = verbose_logger(args.flag_verbose);

    Client::new(&*args.flag_target)
      .and_then(|c| one_shot(&mut log,
                             c,
                             &["PlayStop"],
                             Message::from_word("stop")))
      .unwrap_or_else(|e| werr!("error: {}", e));
}