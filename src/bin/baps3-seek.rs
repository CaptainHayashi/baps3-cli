#![feature(plugin)]

extern crate baps3_protocol;
#[macro_use] extern crate baps3_cli;

extern crate "rustc-serialize" as rustc_serialize;
extern crate docopt;
#[plugin] #[no_link] extern crate docopt_macros;

use baps3_cli::{one_shot, verbose_logger};
use baps3_cli::time::TimeUnit;
use baps3_protocol::proto::Message;

docopt!(Args, "
Seeks to a given position in the currently loaded BAPS3 file.

By default, the position is in microseconds; use one of -H, -M, -S,
or -m to override this.

Usage:
  baps3-seek -h
  baps3-seek [-vHMSm] [-t <target>] <pos>

Options:
  -h, --help             Show this message.
  -v, --verbose          Prints a trail of miscellaneous information
                         about the action.
  -H, --hours            Interpret <pos> as hours.
                         Overrides -M, -S, and -m.
  -M, --minutes          Interpret <pos> as minutes.
                         Overrides -S and -m.
  -S, --seconds          Interpret <pos> as seconds.
                         Overrides -m.
  -m, --milliseconds     Interpret <pos> as milliseconds.
  -t, --target <target>  The target BAPS3 server (host:port).
                         [Default: localhost:1350]
", arg_pos: u64);

/// Uses the unit flags to convert `pos` to microseconds.
fn pos_to_micros<L: Fn(&str)>(log: &L,
                              pos: u64, h: bool, m: bool, s: bool, ms: bool)
  -> u64 {
    let unit   = TimeUnit::from_flags(h, m, s, ms);
    let suffix = unit.suffix();
    let micros = unit.as_micros(pos);

    log!(log, "seek to {}{} ({}us)", pos, suffix, micros);
    micros
}

fn main() {
    let args: Args = Args::docopt().decode().unwrap_or_else(|e| e.exit());
    let log = |&:s:&str| verbose_logger(args.flag_verbose, s);

    let pos = pos_to_micros(&log,
                            args.arg_pos,
                            args.flag_hours,
                            args.flag_minutes,
                            args.flag_seconds,
                            args.flag_milliseconds);
    let spos = pos.to_string();

    one_shot(log,
             &*args.flag_target,
             &["Seek"],
             Message::new("seek").arg(&*spos))
      .unwrap_or_else(|e| werr!("error: {}", e));
}
