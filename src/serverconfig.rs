//! Module containing types and logic for retrieving ROTMG server information

use reqwest::{get, Result as ReqResult};
use serde::Deserialize;
use std::collections::HashMap;
use std::net::IpAddr;

/// Get the official list of servers
pub fn get_servers() -> ReqResult<HashMap<String, IpAddr>> {
    #[derive(Deserialize)]
    pub struct Server {
        #[serde(rename = "Name")]
        name: String,

        #[serde(rename = "DNS")]
        address: IpAddr,
    }

    #[derive(Deserialize)]
    pub struct Chars {
        #[serde(rename = "Servers")]
        servers: Vec<Server>,
    }

    let resp = get("https://realmofthemadgodhrd.appspot.com/char/list")?.text()?;
    let outer = serde_xml_rs::from_str::<Chars>(&resp).unwrap(); // todo: don't use unwrap
    Ok(outer
        .servers
        .into_iter()
        .map(|s| (s.name, s.address))
        .collect())
}

/// The list of servers that may be connected to by game clients
#[derive(Debug, Clone)]
pub struct ServerList {
    servers: HashMap<String, IpAddr>,
    selected_server: String,
}

impl ServerList {
    /// Create a new server list with the given set of servers and default
    pub fn new(servers: HashMap<String, IpAddr>, default: String) -> Self {
        Self {
            servers,
            selected_server: default,
        }
    }

    /// Get the address of the currently selected server
    pub fn get_selected_address(&self) -> &IpAddr {
        &self.servers[&self.selected_server]
    }
}
