//! A simple example which starts a listener on 127.0.0.1:2050 accepting
//! incoming game connections, then logs all chat messages to console

use log::{error, info};
use realmpipe::extractor::Extractor;
use realmpipe::mappings::Mappings;
use realmpipe::packets::server;
use realmpipe::pipe::{AutoPacket, PacketContext, Pipe};
use realmpipe::pipe::{Plugin, PluginState};
use realmpipe::proxy::{client_listener, Connection};
use realmpipe::serverlist::ServerList;
use std::fs::write;
use std::net::{Ipv4Addr, SocketAddr};
use std::sync::Arc;
use tempfile::tempdir;
use tokio::prelude::*;
use tokio::runtime::Runtime;

struct LoggingPlugin;

impl Plugin for LoggingPlugin {
    fn init_plugin(&mut self, client: &Connection, server: &Connection) -> Box<PluginState> {
        info!(
            "Initializing state for connection between {} and {}",
            client.get_ref().peer_addr().unwrap(),
            server.get_ref().peer_addr().unwrap()
        );
        Box::new(LoggingPlugin)
    }
}

impl PluginState for LoggingPlugin {
    fn on_packet(&mut self, packet: &mut AutoPacket, ctx: &mut PacketContext) {
        if let Some(m) = packet.downcast::<server::Text>() {
            info!("{} said: {}", m.name, m.text);
        } else if let Some(r) = packet.downcast::<server::Reconnect>() {
            info!("Got reconnect packet: {:?}", r);
        }
    }
}

fn extract_mappings() -> Mappings {
    // create a temp dir
    let dir = tempdir().expect("error creating temp dir");

    // extract swf
    let swf = dir.path().join("client.swf");
    write(
        &swf,
        &include_bytes!("../tests/AssembleeGameClient1554116567.swf")[..],
    )
    .expect("error extracting client SWF");

    // extract game mappings
    let extractor = Extractor::unpack().expect("error extracting binaries");

    extractor
        .extract_mappings(&swf, false)
        .expect("error extracting mappings")
}

fn main() {
    simple_logger::init_with_level(log::Level::Info).expect("error initializing logger");

    // get the current mappings
    info!("Extracting game mappings");
    let mappings = Arc::new(extract_mappings());

    // the local address to listen on
    let local_addr = SocketAddr::from((Ipv4Addr::new(127, 0, 0, 1), 2050));

    // initialize a tokio runtime
    let mut rt = Runtime::new().expect("error initializing tokio runtime");

    // get the server list
    let servers = rt
        .block_on(ServerList::get_official_servers())
        .expect("error getting server list");

    // now that we have the necessary components, construct the pipe
    let pipe: Arc<Pipe> = Arc::new(
        Pipe::builder()
            .servers(servers, "USEast")
            .mappings(Arc::clone(&mappings))
            .plugin(Box::new(LoggingPlugin))
            .build()
            .expect("error constructing pipe"),
    );

    // everything is set up, we just need to actually use the pipe now
    rt.block_on_all(
        client_listener(&local_addr, Arc::clone(&mappings))
            .expect("error starting listener")
            .for_each(move |client| {
                tokio::spawn(Arc::clone(&pipe).accept_client(client).map_err(|e| {
                    error!("relay error: {:?}", e);
                }));

                Ok(())
            })
            .map_err(|e| {
                error!("listener error: {:?}", e);
            }),
    )
    .expect("error with listener");
}
