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

fn configure_stream(s: TcpStream) -> IoResult<TcpStream> {
    s.set_nodelay(true)?;

    Ok(s)
}

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
        .and_then(configure_stream)
        .map(move |s| Codec::new_client(mappings.as_ref()).framed(s));

    Ok(stream)
}

/// Open a connection to a ROTMG server at `address` using the encryption keys
/// provided by `mappings`. A framed connection is returned, providing duplex
/// communication by way of `RawPacket` instances.
pub fn server_connection(
    address: &SocketAddr,
    mappings: impl AsRef<Mappings>,
) -> impl Future<Item = Framed<TcpStream, Codec>, Error = IoError> {
    TcpStream::connect(address)
        .and_then(configure_stream)
        .map(move |s| Codec::new_server(mappings.as_ref()).framed(s))
}
