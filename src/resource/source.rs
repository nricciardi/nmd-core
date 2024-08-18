use std::{fs::File, io::Read, path::PathBuf, str::FromStr};
use base64::Engine;
use url::Url;

use super::ResourceError;


#[derive(Debug, Clone)]
pub enum Source {
    Remote{ url: Url },
    Local{ path: PathBuf },
    Base64String{ base64: String },
    Bytes{ bytes: Vec<u8> },
}

impl Source {

    pub fn try_to_base64(&self) -> Result<String, ResourceError> {

        let bytes = self.try_to_bytes()?;

        Ok(base64::engine::general_purpose::STANDARD.encode(bytes))
    }

    pub fn try_into_base64(self) -> Result<Self, ResourceError> {

        Ok(Self::Base64String { base64: self.try_to_base64()? })

    }

    pub fn try_to_bytes(&self) -> Result<Vec<u8>, ResourceError> {
        match self {
            Self::Remote { url } => return Err(ResourceError::WrongElaboration(format!("url '{}' cannot be parsed into bytes", url))),
            Self::Local { path } => {

                let mut image_file = File::open(path.clone())?;
                let mut bytes: Vec<u8> = Vec::new();
                image_file.read_to_end(&mut bytes)?;

                return Ok(bytes)
            },
            Self::Base64String { base64 } => Ok(base64::engine::general_purpose::STANDARD.decode(base64).unwrap()),
            Self::Bytes { bytes } => Ok(bytes.clone()),
        }
    }

    pub fn try_into_bytes(self) -> Result<Self, ResourceError> {

        let bytes = self.try_to_bytes()?;

        Ok(Self::Bytes { bytes })
    }
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

impl ToString for Source {
    fn to_string(&self) -> String {
        match self {
            Source::Remote { url } => url.as_str().to_string(),
            Source::Local { path } => path.to_string_lossy().to_string(),
            Source::Base64String { base64 } => base64.clone(),
            Source::Bytes { bytes: base64 } => base64::engine::general_purpose::STANDARD.encode(base64),
        }
    }
}