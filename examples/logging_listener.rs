//! A simple example which starts a listener on 127.0.0.1:2050 accepting
//! incoming game connections, then logging all packets received.

use std::fs::write;
use std::io::Result as IoResult;
use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::Arc;

use log::{error, info, warn};
use tempfile::tempdir;
use tokio::prelude::*;

use realmpipe::extractor::Extractor;
use realmpipe::mappings::Mappings;
use realmpipe::net::proxy::{client_listener, server_connection};
use realmpipe::serverconfig::{get_servers, ServerList};

fn extract_mappings() -> Mappings {
    // create a temp dir
    let dir = tempdir().expect("error creating temp dir");

    // extract swf
    let swf = dir.path().join("client.swf");
    write(
        &swf,
        &include_bytes!("../tests/AssembleeGameClient1553152472.swf")[..],
    )
    .expect("error extracting client SWF");

    // extract game mappings
    let extractor = Extractor::extract().expect("error extracting binaries");

    extractor
        .extract_mappings(&swf, false)
        .expect("error extracting mappings")
}

fn main() {
    simple_logger::init_with_level(log::Level::Debug).expect("error initializing logger");

    info!("extracting game mappings");
    let mappings = Arc::new(extract_mappings());
    let mappings2 = mappings.clone();

    let localhost = SocketAddr::from_str("127.0.0.1:2050").unwrap();
    info!("starting listener on {:?}", &localhost);

    let selected = SocketAddr::from_str("52.23.232.42:2050").unwrap();

    tokio::run(
        client_listener(&localhost, mappings.clone())
            .unwrap()
            .and_then(move |client| {
                info!(
                    "incoming connection from {}",
                    client.get_ref().peer_addr().unwrap()
                );
                server_connection(&selected, mappings2.clone()).map(|server| (client, server))
            })
            .for_each(|(client, server)| -> IoResult<()> {
                info!(
                    "connected to server at {} and client at {}",
                    server.get_ref().peer_addr().unwrap(),
                    client.get_ref().peer_addr().unwrap()
                );

                let (client_sink, client_stream) = client.split();
                let (server_sink, server_stream) = server.split();

                let client_fwd = client_stream
                    .inspect(|p| info!("client -> server: id {}", p.game_id()))
                    .forward(server_sink)
                    .map_err(|e| error!("client -> server: {:?}", e))
                    .map(|_| ());

                let server_fwd = server_stream
                    .inspect(|p| info!("server -> client: id {}", p.game_id()))
                    .forward(client_sink)
                    .map_err(|e| error!("server -> client: {:?}", e))
                    .map(|_| ());

                tokio::spawn(client_fwd);
                tokio::spawn(server_fwd);

                Ok(())
            })
            .map_err(|e| panic!("unexpected error: {}", e)),
    );
}
