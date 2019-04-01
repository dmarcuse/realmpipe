#![allow(missing_docs)]

use super::{AutoPacket, PipeError, Plugin};
use crate::mappings::Mappings;
use crate::proxy::{server_connection, Connection};
use crate::serverlist::ServerList;
use derive_builder::Builder;
use std::net::SocketAddr;
use std::ops::Deref;
use std::sync::Arc;
use std::sync::Mutex;
use tokio::prelude::*;

/// An indicator of which side a packet was sent from
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum PacketSide {
    /// The packet was sent by the server
    Server,

    /// The packet was sent by the client
    Client,
}

/// Represents a
#[derive(Builder)]
#[builder(pattern = "owned")]
pub struct Pipe {
    plugins: Mutex<Vec<Box<dyn Plugin>>>,
    mappings: Arc<Mappings>,
    #[builder(private, setter(name = "internal_servers"))]
    servers: ServerList,
    #[builder(private, setter(name = "internal_default_server"))]
    default_server: String,
}

impl PipeBuilder {
    /// Add a single plugin
    pub fn plugin(mut self, plugin: Box<dyn Plugin>) -> Self {
        if let Some(plugins) = &self.plugins {
            plugins.lock().unwrap().push(plugin);
        } else {
            self.plugins = Some(Mutex::new(vec![plugin]));
        }
        self
    }

    /// Specify the list of remote servers and the default one. The list must
    /// contain at least one server, and the default server name must be present
    /// in the list.
    pub fn servers(self, list: ServerList, default: String) -> Self {
        if list.get_map().is_empty() {
            panic!("server list may not be empty");
        } else if let None = list.get_ip(&default) {
            panic!(
                "default server must be present in list: default {} list {:?}",
                default, list
            );
        }

        self.internal_servers(list).internal_default_server(default)
    }
}

impl Pipe {
    /// Get the socket address for the default server
    pub fn get_default_server(&self) -> SocketAddr {
        self.servers.get_socket(&self.default_server).unwrap()
    }

    pub fn accept_client(self: Arc<Self>, client: Connection) -> impl Future<Error = PipeError> {
        server_connection(&self.get_default_server(), Arc::clone(&self.mappings))
            .from_err()
            .and_then(move |server| {
                // by now, both halves of the pipe have been connected

                // start by initializing the plugins
                let mut plugins = self
                    .plugins
                    .lock()
                    .expect("error acquiring plugin lock")
                    .iter_mut()
                    .map(|p| p.init_plugin(&client, &server))
                    .collect::<Vec<_>>();

                // split both connections
                let (client_sink, client_stream) = client.split();
                let (server_sink, server_stream) = server.split();

                // map the streams to include an indicator of which side sent the packet
                let client_stream = client_stream.map(|p| (PacketSide::Client, p));
                let server_stream = server_stream.map(|p| (PacketSide::Server, p));

                // combine the two streams
                let stream = client_stream.select(server_stream);

                // map the sinks to filter to the packets from the appropriate side
                let client_sink = client_sink.with_flat_map(
                    |(side, pkt)| -> Box<dyn Stream<Item = _, Error = _>> {
                        match side {
                            PacketSide::Client => Box::new(futures::stream::empty()),
                            PacketSide::Server => Box::new(futures::stream::once(Ok(pkt))),
                        }
                    },
                );

                let server_sink = server_sink.with_flat_map(
                    |(side, pkt)| -> Box<dyn Stream<Item = _, Error = _>> {
                        match side {
                            PacketSide::Client => Box::new(futures::stream::once(Ok(pkt))),
                            PacketSide::Server => Box::new(futures::stream::empty()),
                        }
                    },
                );

                // combine the two sinks
                let sink = client_sink.fanout(server_sink);

                // finally, tie it all together into one future
                stream
                    .map(
                        move |(side, raw_pkt)| -> Box<dyn Stream<Item = _, Error = PipeError>> {
                            // wrap the raw packet as an auto packet for easy downcasting
                            let mut auto_pkt = AutoPacket::new(raw_pkt, self.mappings.deref());

                            // invoke plugin callbacks
                            plugins.iter_mut().for_each(|p| p.on_packet(&mut auto_pkt));

                            // pass through the original packet
                            Box::new(futures::stream::once(Ok((side, auto_pkt.into_raw()))))
                        },
                    )
                    .flatten()
                    .forward(sink)
                    .from_err()
            })
    }
}
