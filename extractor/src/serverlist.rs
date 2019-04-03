//! Automatically scrape a `ServerList` from the ROTMG site.

use failure_derive::Fail;
use futures::{Future, Stream};
use lazy_static::lazy_static;
use realmpipe_core::serverlist::ServerList;
use reqwest::r#async::Client;
use reqwest::Error as ReqError;
use serde::Deserialize;
use std::net::IpAddr;

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

/// Get the official server list by retrieving and parsing the XML
///
/// # Examples
///
/// ```
/// use realmpipe_extractor::serverlist::get_official_servers;
/// use tokio::runtime::current_thread::Runtime;
///
/// // start a tokio runtime
/// let mut rt = Runtime::new().unwrap();
///
/// // use the runtime to get the official server list
/// let servers = rt
///     .block_on(get_official_servers())
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
