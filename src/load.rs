use std::io;
use std::path::PathBuf;
use getset::{CopyGetters, Getters, Setters};
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


#[derive(Debug, Getters, CopyGetters, Setters, Clone)]
pub struct LoadConfiguration {
    
    #[getset(get = "pub", set = "pub")]
    input_location: PathBuf,

    #[getset(get_copy = "pub", set = "pub")]
    strict_dossier_configuration_check: bool,

    #[getset(get_copy = "pub", set = "pub")]
    strict_focus_block_check: bool,

    #[getset(get_copy = "pub", set = "pub")]
    strict_paragraphs_loading_rules_check: bool,

    #[getset(get_copy = "pub", set = "pub")]
    parallelization: bool,

    // #[getset(get = "pub", set = "pub")]
    // dossier_name: Option<String>,       // TODO: remove it, place it as method parameter

    // #[getset(get = "pub", set = "pub")]
    // document_name: Option<String>,       // TODO: remove it, place it as method parameter
}

impl Default for LoadConfiguration {
    fn default() -> Self {
        Self {
            input_location: PathBuf::from("."),
            strict_focus_block_check: false,
            strict_dossier_configuration_check: true,
            strict_paragraphs_loading_rules_check: true,
            parallelization: true,
            // document_name: None,
            // dossier_name: None
        }
    }
}















