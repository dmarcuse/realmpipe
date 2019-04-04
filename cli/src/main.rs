#![deny(bare_trait_objects)]

mod config;
mod net;

use log::{debug, LevelFilter};
use log4rs::append::file::FileAppender;
use log4rs::config::{Appender, Config, Root};
use log4rs::Handle;
use std::fs::create_dir_all;
use std::path::{Path, PathBuf};
use structopt::StructOpt;

/// Get the default data directory
fn default_data_dir() -> PathBuf {
    appdirs::user_data_dir(Some(env!("CARGO_PKG_NAME")), None, false)
        .expect("getting application data directory")
}

#[derive(Debug, Clone, PartialEq, Eq, StructOpt)]
struct Opts {}

/// Create the logger using the given options
fn init_logger(data_dir: &Path) -> Handle {
    // create file appender for logging
    let file_appender = FileAppender::builder()
        .append(false)
        .build(data_dir.join("realmpipe_cli.log"))
        .expect("constructing file logger");

    // create logger configuration
    let config = Config::builder()
        .appender(Appender::builder().build("file", Box::new(file_appender)))
        .build(Root::builder().appender("file").build(LevelFilter::Trace))
        .expect("constructing logger configuration");

    log4rs::init_config(config).expect("initializing logger")
}

fn main() {
    // parse command line arguments
    let opts: Opts = Opts::from_args();
    let data_dir = default_data_dir();

    init_logger(&data_dir);
    debug!("Logger initialized");

    debug!("Data directory: {:?}", &data_dir);
    if !data_dir.is_dir() {
        create_dir_all(&data_dir).expect("creating data dir");
    }
}
