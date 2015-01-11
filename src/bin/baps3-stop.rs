#![feature(plugin)]

extern crate baps3_protocol;
#[macro_use] extern crate baps3_cli;

extern crate "rustc-serialize" as rustc_serialize;
extern crate docopt;
#[plugin] #[no_link] extern crate docopt_macros;

use baps3_cli::{ Baps3, Baps3Result, verbose_logger };
use baps3_protocol::proto::Message;

docopt!(Args, "
Stops the currently playing file in a BAPS3 server.

Usage:
  baps3-stop -h
  baps3-stop [-rv] [-t <target>]

Options:
  -h, --help             Show this message.
  -r, --rewind           Seek to the beginning of the file after stopping.
  -v, --verbose          Prints a trail of miscellaneous information
                         about the action.
  -t, --target <target>  The target BAPS3 server (host:port).
                         [Default: localhost:1350]
");

fn stop(Args { flag_rewind,
               flag_target,
               flag_verbose, .. }: Args) -> Baps3Result<()> {
    let mut log   = |&:s:&str| verbose_logger(flag_verbose, s);

    let mut baps3 = try!(Baps3::new(log, &*flag_target,
        &*(if flag_rewind { vec!["PlayStop", "Seek"] }
           else           { vec!["PlayStop"]         })));

    try!(baps3.send(&Message::from_word("stop")));

    if flag_rewind {
        try!(baps3.send(&Message::new("seek", &["0"])));
    }

    Ok(())
}

fn main() {
    let args: Args = Args::docopt().decode().unwrap_or_else(|e| e.exit());
    stop(args).unwrap_or_else(|e| werr!("error: {}", e));
}
