use std::io::Read;
use std::iter::Iterator;
use std::{fs, path::PathBuf};

use anyhow::{Result, anyhow};
use sha2::{Digest, Sha256};
use url::Url;

use crate::completion::global::object_ref::FileResolver;

const MIME_TYPES: [&str; 4] = [
    "application/json",
    "application/yaml",
    "text/yaml",
    "text/plain",
];

pub struct UrlNodeResolver {
    url: Url,
    max_download_size: u64,
}

impl UrlNodeResolver {
    pub fn new(url: Url, max_download_size: u64) -> Self {
        Self {
            url,
            max_download_size,
        }
    }
}

impl FileResolver for UrlNodeResolver {
    fn resolve_file(&self) -> Result<PathBuf> {
        if !self.url.scheme().starts_with("http") {
            return Err(anyhow!("Unsupported scheme"));
        }
        let mut hasher = Sha256::new();
        hasher.update(self.url.as_str().as_bytes());
        let hash: String = hasher
            .finalize()
            .iter()
            .map(|b| format!("{:02x}", b))
            .collect();

        let cache_file = dirs::cache_dir()
            .ok_or(anyhow!("No cache dir"))?
            .join("grustonnet")
            .join(hash);
        if !cache_file.exists() {
            let mut response_body = ureq::get(self.url.as_str())
                .header("Accept", MIME_TYPES.join(", "))
                .call()?;
            let response_reader = response_body.body_mut().as_reader();

            let mut response = String::new();
            response_reader
                .take(self.max_download_size)
                .read_to_string(&mut response)?;

            fs::create_dir_all(cache_file.parent().ok_or(anyhow!("no parent"))?)?;
            fs::write(&cache_file, &response)?;
        }
        Ok(cache_file)
    }
}
