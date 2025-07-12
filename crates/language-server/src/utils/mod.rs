use std::str::FromStr;

use lsp_types::Uri;

pub mod cst;
pub mod diff;
pub mod rope;

pub trait UriHelper {
    fn from_string(val: &str) -> Result<Uri, <Uri as FromStr>::Err>;
}

impl UriHelper for Uri {
    fn from_string(val: &str) -> Result<Uri, <Uri as FromStr>::Err> {
        let mut orig_uri = Uri::from_str(val)?;
        if orig_uri.scheme().is_none() {
            orig_uri = Uri::from_str(&format!("file:///{}", val))?;
        }
        Ok(orig_uri)
    }
}
