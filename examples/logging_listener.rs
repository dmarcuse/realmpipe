//! A simple example which starts a listener on 127.0.0.1:2050 accepting
//! incoming game connections, then logging all packets received.

use std::fs::write;
use std::io::Result as IoResult;
use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::Arc;

use log::{debug, error, info, warn};
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

    info!("Extracting game mappings");
    let mappings = Arc::new(extract_mappings());

    // the local address to listen on
    let local_addr = SocketAddr::from_str("127.0.0.1:2050").unwrap();

    // the remote address to connect to
    // at the time of writing, this is USEast
    let remote_addr = SocketAddr::from_str("52.23.232.42:2050").unwrap();

    info!("starting listener on {:?}", &local_addr);
    tokio::run(
        client_listener(&local_addr, mappings.clone())
            .unwrap()
            .map_err(|e| panic!("unexpected error: {}", e))
            .and_then(move |client| {
                info!(
                    "Accepted connection from {}",
                    client.get_ref().peer_addr().unwrap()
                );

                server_connection(&remote_addr, mappings.clone())
                    .map(|server| (client, server))
                    .inspect(|(client, server)| {
                        info!(
                            "Proxied connection between {} and {} established",
                            client.get_ref().peer_addr().unwrap(),
                            server.get_ref().peer_addr().unwrap()
                        )
                    })
            })
            .map_err(|e| {
                error!("connection error: {}", e);
            })
            .for_each(|(client, server)| {
                let (client_sink, client_stream) = client.split();
                let (server_sink, server_stream) = server.split();

                let client_fwd = client_stream
                    .inspect(|raw| {
                        info!(
                            "Raw client pkt id {} len {}",
                            raw.game_id(),
                            raw.contents().len()
                        );
                    })
                    .forward(server_sink)
                    .map_err(|e| {
                        error!("client pkt error: {}", e);
                    })
                    .map(|_| ());

                let server_fwd = server_stream
                    .inspect(|raw| {
                        info!(
                            "Raw server pkt id {} len {}",
                            raw.game_id(),
                            raw.contents().len()
                        );
                    })
                    .forward(client_sink)
                    .map_err(|e| {
                        error!("server pkt error: {}", e);
                    })
                    .map(|_| ());

                tokio::spawn(client_fwd);
                tokio::spawn(server_fwd);
                Ok(())
            }),
    );
}
