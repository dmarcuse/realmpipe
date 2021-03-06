//! Realmpipe - a lightweight proxy for Realm of the Mad God. This crate
//! exposes the functionality as a library, allowing other tools to be built
//! using its logic.

#![deny(bare_trait_objects)]
#![deny(missing_docs)]

pub mod adapters;
mod ext;
pub mod gamedata;
pub mod mappings;
pub mod packets;
pub mod pipe;
pub mod proxy;
pub mod rc4;
pub mod serverlist;
