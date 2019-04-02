//! High-level API for interacting with packets via a plugin system

mod autopacket;
mod context;
mod error;
mod pipe;
mod plugin;

pub use self::autopacket::AutoPacket;
pub use self::context::PacketContext;
pub use self::error::PipeError;
pub use self::pipe::{Pipe, PipeBuilder};
pub use self::plugin::{Plugin, PluginState};
