use std::io;
use thiserror::Error;
use crate::mmo::{uri::UriError, MultiMediaObjectError};


#[derive(Error, Debug)]
pub enum LoadError {

    #[error("bucket of errors: '{0:#?}'")]
    BucketOfErrors(Vec<LoadError>),

    #[error(transparent)]
    MmoError(#[from] MultiMediaObjectError),

    #[error(transparent)]
    UriError(#[from] UriError),

    #[error("elaboration error: {0}")]
    ElaborationError(String),
    
    #[error(transparent)]
    IoError(#[from] io::Error),

    #[error("block error: {0}")]
    BlockError(String),

    #[error("invalid tag: {0}")]
    InvalidTag(String)
}

impl Clone for LoadError {
    fn clone(&self) -> Self {
        match self {
            Self::IoError(e) => Self::ElaborationError(e.to_string()),
            other => other.clone()
        }
    }
}