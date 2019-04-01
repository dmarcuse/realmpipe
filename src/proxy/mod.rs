//! The actual implementation of the proxy server.

pub mod codec;
mod policy;
pub mod raw;

use self::codec::Codec;
use self::policy::handle_policy_request;
use crate::mappings::Mappings;
use std::convert::identity;
use std::io::{Error as IoError, Result as IoResult};
use std::net::SocketAddr;
use tokio::codec::{Decoder, Framed};
use tokio::net::{TcpListener, TcpStream};
use tokio::prelude::*;

/// A framed TCP connection that operates on `RawPacket` instances
pub type Connection = Framed<TcpStream, Codec>;

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
) -> IoResult<impl Stream<Item = Connection, Error = IoError>> {
    let stream = TcpListener::bind(address)?
        .incoming()
        .and_then(configure_stream)
        .and_then(handle_policy_request)
        .filter_map(identity)
        .map(move |s| Codec::new_client(mappings.as_ref()).framed(s));

    Ok(stream)
}

/// Open a connection to a ROTMG server at `address` using the encryption keys
/// provided by `mappings`. A framed connection is returned, providing duplex
/// communication by way of `RawPacket` instances.
pub fn server_connection(
    address: &SocketAddr,
    mappings: impl AsRef<Mappings>,
) -> impl Future<Item = Connection, Error = IoError> {
    TcpStream::connect(address)
        .and_then(configure_stream)
        .map(move |s| Codec::new_server(mappings.as_ref()).framed(s))
}
