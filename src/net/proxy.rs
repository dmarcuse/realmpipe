//! The actual proxy implementation

mod codec;

use std::net::SocketAddr;
use tokio::prelude::*;
