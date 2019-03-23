//! The actual proxy implementation

pub mod codec;
pub mod raw;

use crate::mappings::Mappings;
use crate::net::proxy::codec::Codec;
use std::io::{Error as IoError, Result as IoResult};
use std::net::SocketAddr;
use tokio::codec::{Decoder, Framed};
use tokio::net::{TcpListener, TcpStream};
use tokio::prelude::*;

/// Start a client listener, listening for incoming client connections on
/// `address` and using encryption keys provided by `mappings`. A stream of
/// framed connections is returned, providing duplex communication by way of
/// `RawPacket` instances.
pub fn client_listener(
    address: &SocketAddr,
    mappings: impl AsRef<Mappings>,
) -> IoResult<impl Stream<Item = Framed<TcpStream, Codec>, Error = IoError>> {
    let stream = TcpListener::bind(address)?
        .incoming()
        .and_then(|s| -> IoResult<TcpStream> {
            // put the stream in nodelay mode to reduce latency
            s.set_nodelay(true)?;

            Ok(s)
        })
        .map(move |s| Codec::new_client(mappings.as_ref()).framed(s));

    Ok(stream)
}
