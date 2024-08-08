use std::{path::PathBuf, str::FromStr};
use url::Url;

use super::ResourceError;


#[derive(Debug, Clone)]
pub enum Source {
    Remote{ url: Url },
    Local{ path: PathBuf }
}

impl FromStr for Source {
    type Err = ResourceError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let source = match reqwest::Url::parse(s) {
            Ok(url) => Self::Remote { url },
            Err(_) => Self::Local { path: PathBuf::from(s) },
        };

        Ok(source)
    }
}