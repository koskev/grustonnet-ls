use std::{fs, str::FromStr};

use anyhow::{Result, anyhow};
use lsp_types::Uri;

pub mod cst;
pub mod diff;
pub mod hashqueue;
pub mod rope;

pub trait UriHelper {
    fn from_path(val: &str) -> Result<Uri>;
}

impl UriHelper for Uri {
    fn from_path(val: &str) -> Result<Uri> {
        let mut orig_uri = Uri::from_str(val)?;
        if orig_uri.scheme().is_none() {
            let absolute = fs::canonicalize(val)?;
            orig_uri = Uri::from_str(&format!(
                "file://{}",
                absolute
                    .to_str()
                    .ok_or(anyhow!("Unable to convert path to string"))?
            ))?;
        }
        Ok(orig_uri)
    }
}
