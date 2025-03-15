pub(crate) mod source;
pub(crate) mod uri;
pub(crate) mod image;
pub mod nmd_unique_identifier;
pub mod compilation_outcome;
pub(crate) mod table;
pub mod bucket;
pub mod span;
pub(crate) mod text_reference;
pub(crate) mod resource;


pub type HashMap<K, V> = ahash::HashMap<K, V>;


use std::io;
use thiserror::Error;


#[derive(Error, Debug)]
pub enum MultiMediaObjectError {

    #[error("MMO '{0}' not found")]
    MmoNotFound(String),

    #[error("MMO is invalid")]
    InvalidMmo,

    #[error("resource is invalid because: {0}")]
    InvalidResourceVerbose(String),

    #[error("MMO cannot be created: {0}")]
    CreationError(String),

    #[error("MMO '{0}' cannot be read")]
    ReadError(String),

    #[error(transparent)]
    IoError(#[from] io::Error),

    #[error("elaboration error: {0}")]
    ElaborationError(String),
}

impl Clone for MultiMediaObjectError {
    fn clone(&self) -> Self {
        match self {
            Self::IoError(e) => Self::ElaborationError(e.to_string()),
            other => other.clone()
        }
    }
}