use std::{
    path::{Path, PathBuf},
    str::FromStr,
};

use anyhow::{Result, anyhow};
use lsp_types::Uri;
use path_calculate::Calculate;
use url::Url;

use crate::canonicalize;

pub trait UriHelper {
    fn from_path<P: AsRef<Path>>(val: P) -> Result<Uri>;
    fn from_url(url: Url) -> Result<Uri>;
    fn to_file_path(&self) -> Result<PathBuf>;
    fn to_file_path_string(&self) -> Result<String>;
    fn relative_string(&self, other: &Uri) -> Result<String>;
}

impl UriHelper for Uri {
    fn relative_string(&self, other: &Uri) -> Result<String> {
        let other_path = other.to_file_path()?;
        let uri_path = self.to_file_path()?;
        let uri_path = uri_path
            .parent()
            .ok_or(anyhow!("file does not have a parent"))?;
        let diff = other_path
            .related_to(uri_path)
            .map_err(|_| anyhow!("Unable to get path diff"))?;

        let relative_string = diff
            .to_str()
            .ok_or(anyhow!("invalid diff path"))?
            .to_string();

        Ok(relative_string)
    }
    fn from_path<P: AsRef<Path>>(val: P) -> Result<Uri> {
        let absolute_path = canonicalize(val)?;

        Self::from_url(
            Url::from_file_path(&absolute_path)
                .map_err(|_| anyhow!("Unable to find path {:?}", absolute_path))?,
        )
    }

    fn from_url(url: Url) -> Result<Uri> {
        Uri::from_str(url.as_str()).map_err(|e| e.into())
    }

    fn to_file_path(&self) -> Result<PathBuf> {
        Url::from_str(self.as_str())?
            .to_file_path()
            .map_err(|_| anyhow!("Unable to convert {}", self.as_str()))
    }

    fn to_file_path_string(&self) -> Result<String> {
        Ok(self
            .to_file_path()?
            .to_str()
            .ok_or(anyhow!("Not a str"))?
            .to_string())
    }
}
