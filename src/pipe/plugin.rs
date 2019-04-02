use crate::pipe::AutoPacket;
use crate::proxy::Connection;

/// A plugin to handle events
pub trait Plugin: Send {
    /// Handle a new connection, initializing a new plugin state for it
    fn init_plugin(&mut self, client: &Connection, server: &Connection) -> Box<dyn PluginState>;
}

/// An instance of a plugin for a single connection
#[allow(unused_variables)]
pub trait PluginState: Send {
    /// Handle an intercepted packet
    fn on_packet(&mut self, packet: &mut AutoPacket) {}
}
