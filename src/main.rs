#![deny(bare_trait_objects)]

mod extractor;
mod net;

use cursive::Cursive;
use log::Level;
use structopt::StructOpt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, StructOpt)]
struct Opts {
    log_level: Level,
}

fn main() {
    // setup logging via cursive
    cursive::logger::init();

    // initialize cursive
    let mut siv = Cursive::default();

    siv.run();
}
