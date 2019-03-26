//! Types and functions to handle ROTMG servers.
//!
//! A `ServerList`, used to map server names and abbreviations to IP addresses,
//! can be retrieved from the official game site, or can be constructed
//! manually. Thie list can then be used to get IP or socket addresses for each
//! server.

use std::collections::HashMap;
use std::hash::Hash;
use std::net::{IpAddr, SocketAddr};

use failure_derive::Fail;
use futures::stream::Stream;
use futures::Future;
use lazy_static::lazy_static;
use reqwest::r#async::Client;
use reqwest::Error as ReqError;
use serde::{Deserialize, Serialize};

/// Automatically generate an abbreviated form of the given server name.
/// This uses substring replacements (e.g. Asia -> as, South -> s, etc),
/// and should still work (to an extent) with unofficial server names.
///
/// # Examples
///
/// ```
/// # use realmpipe::serverlist::abbreviate;
/// assert_eq!(&abbreviate("USEast"), "use");
/// assert_eq!(&abbreviate("AsiaSouthEast"), "asse");
/// assert_eq!(&abbreviate("USSouth"), "uss");
/// assert_eq!(&abbreviate("USSouthWest"), "ussw");
/// assert_eq!(&abbreviate("USEast2"), "use2");
/// assert_eq!(&abbreviate("USNorthWest"), "usnw");
/// assert_eq!(&abbreviate("AsiaEast"), "ase");
/// assert_eq!(&abbreviate("EUSouthWest"), "eusw");
/// assert_eq!(&abbreviate("USSouth2"), "uss2");
/// assert_eq!(&abbreviate("EUNorth2"), "eun2");
/// assert_eq!(&abbreviate("EUSouth"), "eus");
/// assert_eq!(&abbreviate("USSouth3"), "uss3");
/// assert_eq!(&abbreviate("EUWest2"), "euw2");
/// assert_eq!(&abbreviate("USMidWest"), "usmw");
/// assert_eq!(&abbreviate("EUWest"), "euw");
/// assert_eq!(&abbreviate("USEast3"), "use3");
/// assert_eq!(&abbreviate("USWest"), "usw");
/// assert_eq!(&abbreviate("USWest3"), "usw3");
/// assert_eq!(&abbreviate("USMidWest2"), "usmw2");
/// assert_eq!(&abbreviate("EUEast"), "eue");
/// assert_eq!(&abbreviate("Australia"), "aus");
/// ```
pub fn abbreviate(name: &str) -> String {
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

/// An error getting the official server list
#[derive(Debug, Fail)]
pub enum GetServersError {
    /// An error with the network request
    #[fail(display = "Network error: {}", _0)]
    NetError(ReqError),

    /// An error converting the response from XML
    #[fail(display = "XML error: {}", _0)]
    XmlError(String),
}

lazy_static! {
    static ref CLIENT: Client = Client::new();
}

impl ServerList {
    /// Create a new `ServerList` with the given servers. Note that this method
    /// will create a new map and populate it with lowercase names and
    /// abbreviations (see the abbreviate function for more info).
    pub fn new(servers: &HashMap<impl AsRef<str> + Hash + Eq, IpAddr>) -> Self {
        let mut serverlist = HashMap::new();

        for (name, ip) in servers.iter() {
            serverlist.insert(name.as_ref().to_lowercase(), ip.clone());

            let abbreviation = abbreviate(&name.as_ref().to_lowercase());
            if !serverlist.contains_key(&abbreviation) {
                serverlist.insert(abbreviation, ip.clone());
            }
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

    /// Get the official server list by retrieving and parsing the XML
    ///
    /// # Examples
    ///
    /// ```
    /// use realmpipe::serverlist::ServerList;
    /// use tokio::runtime::Runtime;
    ///
    /// // start a tokio runtime
    /// let mut rt = Runtime::new().unwrap();
    ///
    /// // use the runtime to get the official server list
    /// let servers = rt
    ///     .block_on(ServerList::get_official_servers())
    ///     .expect("error getting server list");
    ///
    /// println!("Server list: {:?}", servers);
    ///
    /// ```
    pub fn get_official_servers() -> impl Future<Item = ServerList, Error = GetServersError> {
        #[derive(Deserialize)]
        struct Server {
            #[serde(rename = "Name")]
            name: String,

            #[serde(rename = "DNS")]
            ip: IpAddr,
        }

        #[derive(Deserialize)]
        struct Servers {
            #[serde(rename = "Server")]
            server_list: Vec<Server>,
        }

        #[derive(Deserialize)]
        struct Chars {
            #[serde(rename = "Servers")]
            servers: Servers,
        }

        CLIENT
            .get("https://realmofthemadgodhrd.appspot.com/char/list")
            .send()
            .and_then(|response| response.into_body().concat2())
            .map_err(GetServersError::NetError)
            .map(|utf8| String::from_utf8_lossy(&utf8).into_owned())
            .and_then(|text| {
                serde_xml_rs::from_str::<Chars>(&text)
                    .map_err(|e| GetServersError::XmlError(e.to_string()))
            })
            .map(|s| {
                let official = s
                    .servers
                    .server_list
                    .into_iter()
                    .map(|s| (s.name, s.ip))
                    .collect();

                ServerList::new(&official)
            })
    }
}
