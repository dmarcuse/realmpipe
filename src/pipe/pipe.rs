#![allow(missing_docs)]

use super::{AutoPacket, PacketContext, PipeError, Plugin};
use crate::mappings::Mappings;
use crate::proxy::raw::RawPacket;
use crate::proxy::{server_connection, Connection};
use crate::serverlist::ServerList;
use derive_builder::Builder;
use log::warn;
use std::default::Default;
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
    #[builder(default = "Mutex::new(Vec::new())")]
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
    pub fn servers(self, list: ServerList, default: &str) -> Self {
        let default = default.to_lowercase();
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
    /// Create a new pipe builder
    pub fn builder() -> PipeBuilder {
        PipeBuilder::default()
    }

    /// Get the socket address for the default server
    pub fn get_default_server(&self) -> SocketAddr {
        self.servers.get_socket(&self.default_server).unwrap()
    }

    /// Accept a given client connection using this pipe, opening the server
    /// connection, then processing packets with plugins until closure
    pub fn accept_client(
        self: Arc<Self>,
        client: Connection,
    ) -> impl Future<Item = (), Error = PipeError> + Send {
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
                    |(side, pkt)| -> Box<dyn Stream<Item = _, Error = _> + Send> {
                        match side {
                            PacketSide::Client => Box::new(futures::stream::empty()),
                            PacketSide::Server => Box::new(futures::stream::once(Ok(pkt))),
                        }
                    },
                );

                let server_sink = server_sink.with_flat_map(
                    |(side, pkt)| -> Box<dyn Stream<Item = _, Error = _> + Send> {
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
                        move |(side, raw)| -> Box<dyn Stream<Item = _, Error = PipeError> + Send> {
                            // wrap the raw packet as an auto packet for easy downcasting
                            let mut auto = AutoPacket::new(raw, self.mappings.deref());

                            // create a packet context
                            let mut ctx = PacketContext::default();

                            // invoke plugin callbacks
                            plugins
                                .iter_mut()
                                .for_each(|p| p.on_packet(&mut auto, &mut ctx));

                            // queue up packets to send
                            let mut queue = Vec::with_capacity(1 + ctx.extra.len());

                            // if any plugin requested to cancel this packet, we don't send it
                            if !ctx.cancelled {
                                queue.push((side, auto.into_raw()));
                            }

                            // next, we add any packets that plugins requested to be sent
                            for pkt in ctx.extra {
                                let side = if pkt.get_internal_id().is_server() {
                                    PacketSide::Server
                                } else {
                                    PacketSide::Client
                                };

                                let raw = RawPacket::from_packet(pkt, self.mappings.deref());

                                match raw {
                                    Ok(raw) => queue.push((side, raw)),
                                    Err(e) => warn!("Error encoding packet: {:?}", e),
                                }
                            }

                            // finally, return the queue
                            Box::new(futures::stream::iter_ok(queue))
                        },
                    )
                    .flatten()
                    .forward(sink)
                    .from_err()
                    .map(|_| ())
            })
    }
}
