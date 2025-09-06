pub use reqwest::blocking::{Client, Response};

use snapshot_cache_core::util::{Error, Result};

use crate::sources::sources::ConfigSource;

pub struct HttpConfigSource {
    client: Client,
    url: String,
}

impl HttpConfigSource {
    pub fn new(client: Client, url: String) -> HttpConfigSource {
        HttpConfigSource {
            client,
            url,
        }
    }

    fn get_version(resp: &Response) -> Option<String> {
        let option = resp.headers()
            .get("Last-Modified")
            .map(|h| h.to_str())
            .map(|r| r.map(String::from));
        match option {
            None | Some(Err(_)) => None,
            Some(Ok(s)) => Some(s),
        }
    }
}

impl ConfigSource<String, Response> for HttpConfigSource {
    fn fetch(&self) -> Result<(Option<String>, Response)> {
        let resp = self.client.get(self.url.as_str()).send()?;

        if resp.status().is_success() {
            Ok((HttpConfigSource::get_version(&resp), resp))
        } else {
            Err(Error::new(format!("Fetch failed. Status: {}", resp.status().as_str()).as_str()))
        }
    }

    fn fetch_if_newer(&self, version: &String) -> Result<Option<(Option<String>, Response)>> {
        let resp = self.client.get(self.url.as_str())
            .header("If-Modified-Since", version)
            .send()?;

        if resp.status().is_success() {
            Ok(Some((HttpConfigSource::get_version(&resp), resp)))
        } else if resp.status() == 304 {
            Ok(None)
        } else {
            Err(Error::new(format!("Fetch failed. Status: {}", resp.status().as_str()).as_str()))
        }
    }
}
