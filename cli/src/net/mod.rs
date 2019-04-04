use lazy_static::lazy_static;
use reqwest::r#async::Client;

mod autoupdate;

lazy_static! {
    /// The HTTP client to use for all requests
    static ref CLIENT: Client = Client::new();
}
