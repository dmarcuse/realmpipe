//! Types and functions to handle ROTMG servers.
//!
//! A `ServerList`, used to map server names and abbreviations to IP addresses,
//! can be retrieved from the official game site, or can be constructed
//! manually. The list can then be used to get IP or socket addresses for each
//! server.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::hash::Hash;
use std::net::{IpAddr, SocketAddr};

/// Automatically generate an abbreviated form of the given server name.
/// This uses substring replacements (e.g. Asia -> as, South -> s, etc),
/// and should still work (to an extent) with unofficial server names.
///
/// # Examples
///
/// ```
/// # use realmpipe_core::serverlist::abbreviate_server_name;
/// assert_eq!(&abbreviate_server_name("USEast"), "use");
/// assert_eq!(&abbreviate_server_name("AsiaSouthEast"), "asse");
/// assert_eq!(&abbreviate_server_name("USSouth"), "uss");
/// assert_eq!(&abbreviate_server_name("USSouthWest"), "ussw");
/// assert_eq!(&abbreviate_server_name("USEast2"), "use2");
/// assert_eq!(&abbreviate_server_name("USNorthWest"), "usnw");
/// assert_eq!(&abbreviate_server_name("AsiaEast"), "ase");
/// assert_eq!(&abbreviate_server_name("EUSouthWest"), "eusw");
/// assert_eq!(&abbreviate_server_name("USSouth2"), "uss2");
/// assert_eq!(&abbreviate_server_name("EUNorth2"), "eun2");
/// assert_eq!(&abbreviate_server_name("EUSouth"), "eus");
/// assert_eq!(&abbreviate_server_name("USSouth3"), "uss3");
/// assert_eq!(&abbreviate_server_name("EUWest2"), "euw2");
/// assert_eq!(&abbreviate_server_name("USMidWest"), "usmw");
/// assert_eq!(&abbreviate_server_name("EUWest"), "euw");
/// assert_eq!(&abbreviate_server_name("USEast3"), "use3");
/// assert_eq!(&abbreviate_server_name("USWest"), "usw");
/// assert_eq!(&abbreviate_server_name("USWest3"), "usw3");
/// assert_eq!(&abbreviate_server_name("USMidWest2"), "usmw2");
/// assert_eq!(&abbreviate_server_name("EUEast"), "eue");
/// assert_eq!(&abbreviate_server_name("Australia"), "aus");
/// ```
pub fn abbreviate_server_name(name: &str) -> String {
    name.to_lowercase()
        .replace("east", "e")
        .replace("west", "w")
        .replace("south", "s")
        .replace("north", "n")
        .replace("asia", "as")
        .replace("mid", "m")
        .replace("australia", "aus")
}

/// A list of remote ROTMG servers
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ServerList {
    servers: HashMap<String, IpAddr>,
}

impl ServerList {
    /// Create a new `ServerList` with the given servers. Note that this method
    /// will create a new map and populate it with lowercase names and
    /// abbreviations (see the abbreviate function for more info).
    pub fn new(servers: &HashMap<impl AsRef<str> + Hash + Eq, IpAddr>) -> Self {
        let mut serverlist = HashMap::new();

        for (name, ip) in servers.iter() {
            serverlist.insert(name.as_ref().to_lowercase(), ip.clone());

            let abbreviation = abbreviate_server_name(&name.as_ref().to_lowercase());
            serverlist.entry(abbreviation).or_insert(*ip);
        }

        Self {
            servers: serverlist,
        }
    }

    /// Get the internal map of server names/abbreviations to server IPs
    pub fn get_map(&self) -> &HashMap<String, IpAddr> {
        &self.servers
    }

    /// Get the IP address of a server
    pub fn get_ip(&self, name: &str) -> Option<IpAddr> {
        self.servers.get(name).cloned()
    }

    /// Get the socket address to connect to a server
    pub fn get_socket(&self, name: &str) -> Option<SocketAddr> {
        self.get_ip(name).map(|i| SocketAddr::new(i, 2050))
    }
}
