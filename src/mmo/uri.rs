use once_cell::sync::Lazy;
use regex::Regex;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::utility::{file_utility, url_utility};

static OF_INTERNAL_RESOURCE_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"(.*)?#(.*)").unwrap());

/// separator used to concatenate more than one strings 
const SEPARATOR: &str = "-";

const SPACE_REPLACER: char = '-';


#[derive(Error, Debug)]
pub enum UriError {
    #[error("invalid URL")]
    InvalidUrl,

    #[error("invalid asset")]
    InvalidAsset,

    #[error("invalid internal reference")]
    InvalidInternalReference,
}

/// NMD URI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NUri {
    Url(String),
    Asset(String),

    /// An internal resource is an heading, image or text with an ID of a dossier document. 
    /// An internal resource is composed by "document name" (where there is the resource) and the resource ID.
    InternalReference(String)
}

impl NUri {
    pub fn of_url(raw: &str) -> Result<Self, UriError> {

        if !url_utility::is_valid_remote_resource(raw) {
            return Err(UriError::InvalidUrl)
        }

        Ok(Self::Url(String::from(raw)))
    }

    pub fn of_asset(raw: &str) -> Result<Self, UriError> {
        if !file_utility::is_file_path(raw) {
            return Err(UriError::InvalidAsset)
        }

        Ok(Self::Asset(String::from(raw)))
    }

    /// From raw internal string.
    /// 
    /// Raw string must be in the format: <document-name>#id
    /// 
    /// If `#` is omitted, it is placed automatically.
    /// 
    /// <document-name> can be omitted. 
    pub fn of_internal(raw: &str, document_name_if_missed: Option<&impl ToString>) -> Result<Self, UriError> {

        let mut raw = String::from(raw);

        if !raw.starts_with("#") {
            raw = format!("#{}", raw);
        }

        let raw = raw.to_lowercase();

        let caps = OF_INTERNAL_RESOURCE_REGEX.captures(&raw);

        if caps.is_none() {
            return Err(UriError::InvalidInternalReference)
        }

        let caps = caps.unwrap();

        let document_name = caps.get(1);
        let value = caps.get(2);

        if value.is_none() {
            return Err(UriError::InvalidInternalReference)
        }
        let value = value.unwrap().as_str();

        if let Some(document_name) = document_name {

            let document_name = document_name.as_str().trim();

            if !document_name.is_empty() {

                return Ok(Self::InternalReference(format!("{}{}{}", document_name, SEPARATOR, value)))
            }
        }

        Ok(Self::InternalReference(format!("{}{}{}", document_name_if_missed.unwrap().to_string(), SEPARATOR, value)))
        
    }

    /// Create new based on string. Argument can be in the following forms:
    /// 
    /// - document_name#id
    /// - #id
    /// - url
    /// - url#id
    /// - asset 
    pub fn of(raw: &str, document_name_if_missed: Option<&impl ToString>) -> Result<Self, UriError> {

        if url_utility::is_valid_remote_resource(raw) {
            return Self::of_url(raw)
        }

        if raw.starts_with("#") {
            return Self::of_internal(raw, document_name_if_missed)

        } else {        // asset
            return Self::of_asset(raw)
        }
    }

    pub fn build(&self) -> String {

        match &self {
            NUri::Url(value) | NUri::Asset(value) => value.clone(),
            NUri::InternalReference(value) => format!("#{}", Self::parse_str(value))
        }
    }

    pub fn build_without_internal_sharp(&self) -> String {

        match &self {
            NUri::InternalReference(value) => Self::parse_str(value),
            _ => self.build()
        }

    }

    fn parse_str(s: &str) -> String {

        s.chars().map(|c| {

            if c.is_alphanumeric() {
                return c;
            }

            if c == ' ' {
                return SPACE_REPLACER;
            }

            '-'
        }).collect()
    }
}