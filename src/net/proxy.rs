//! The actual proxy implementation

pub mod codec;
pub mod raw;

use std::net::SocketAddr;
use tokio::prelude::*;
