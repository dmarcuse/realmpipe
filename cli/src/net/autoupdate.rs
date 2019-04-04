use super::CLIENT;
use bytes::Buf;
use futures::{Future, Stream};
use reqwest::r#async::Chunk;
use reqwest::Error as ReqError;

/// Get the latest version of the game client
pub fn get_latest_version() -> impl Future<Item = String, Error = ReqError> {
    CLIENT
        .get("https://realmofthemadgodhrd.appspot.com/version.txt")
        .send()
        .and_then(|response| response.into_body().concat2())
        .map(|body| String::from_utf8_lossy(body.bytes()).into_owned())
}

/// Get the given game client version
fn get_client(version: &str) -> impl Stream<Item = Chunk, Error = ReqError> {
    CLIENT
        .get(&format!(
            "https://realmofthemadgodhrd.appspot.com/AssembleeGameClient{}.swf",
            version
        ))
        .send()
        .map(|response| response.into_body())
        .flatten_stream()
}

/// Get the latest version of the game client
pub fn get_latest_client(version: &str) -> impl Stream<Item = Chunk, Error = ReqError> {
    get_latest_version()
        .map(|version| get_client(&version))
        .flatten_stream()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::runtime::Runtime;

    #[test]
    pub fn test_get_client_version() {
        let mut rt = Runtime::new().unwrap();
        let version = rt.block_on(get_latest_version()).unwrap();
        println!("Latest client version: {}", version);
    }
}