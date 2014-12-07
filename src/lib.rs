#![feature(if_let)]
#![feature(phase)]
#![feature(macro_rules)]
#![feature(unboxed_closures)]

extern crate baps3_protocol;
extern crate docopt;
#[phase(plugin)] extern crate docopt_macros;

pub mod client;
pub mod util;