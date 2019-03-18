#![deny(bare_trait_objects)]

mod extractor;
mod net;
mod ui;

use structopt::StructOpt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, StructOpt)]
struct Opts {}

fn main() {
    let opts: Opts = Opts::from_args();

    // setup logging via cursive
    cursive::logger::init();

    ui::run();
}
