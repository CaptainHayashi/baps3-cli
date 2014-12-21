#![feature(phase)]
#![feature(unboxed_closures)]

extern crate baps3_protocol;
extern crate baps3_cli;
#[phase(plugin)] extern crate baps3_cli;
extern crate docopt;
extern crate serialize;
#[phase(plugin)] extern crate docopt_macros;

use baps3_cli::{Logger, one_shot, verbose_logger};
use baps3_cli::client::Client;
use baps3_cli::message::Message;
use baps3_cli::time::TimeUnit;

docopt!(Args deriving Show, "
Seeks to a given position in the currently loading BAPS3 file.

By default, the position is in microseconds; use one of -H, -M, -S,
or -m to override this.

Usage:
  baps3-seek [options] <pos>

Options:
  -h, --help             Show this message.
  -t, --target <target>  The target BAPS3 server (host:port).
                         [Default: localhost:1350]
  -v, --verbose          Prints a trail of miscellaneous information
                         about the action.
  -H, --hours            Interpret <pos> as hours.
                         Overrides -M, -S, and -m.
  -M, --minutes          Interpret <pos> as minutes.
                         Overrides -S and -m.
  -S, --seconds          Interpret <pos> as seconds.
                         Overrides -m.
  -m, --milliseconds     Interpret <pos> as milliseconds.
", arg_pos: u64);

/// Uses the unit flags to convert `pos` to microseconds.
fn pos_to_micros(log: &mut Logger, pos: u64, h: bool, m: bool, s: bool, ms: bool)
  -> u64 {
    let unit   = TimeUnit::from_flags(h, m, s, ms);
    let suffix = unit.suffix();
    let micros = unit.as_micros(pos);

    log!(log, "seek to {}{} ({}us)", pos, suffix, micros);
    micros
}

fn main() {
    let args: Args = Args::docopt().decode().unwrap_or_else(|e| e.exit());
    let mut log = verbose_logger(args.flag_verbose);

    let pos = pos_to_micros(&mut log,
                            args.arg_pos,
                            args.flag_hours,
                            args.flag_minutes,
                            args.flag_seconds,
                            args.flag_milliseconds);
    let spos = pos.to_string();

    Client::new(&*args.flag_target)
      .and_then(|c| one_shot(&mut log,
                             c,
                             &["Seek"],
                             Message::new("seek", &[&*spos])))
      .unwrap_or_else(|e| werr!("error: {}", e));
}