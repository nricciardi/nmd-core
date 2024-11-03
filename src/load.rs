use std::io;
use std::path::PathBuf;
use getset::{CopyGetters, Getters, Setters};
use thiserror::Error;
use crate::resource::resource_reference::ResourceReferenceError;
use crate::resource::ResourceError;




#[derive(Error, Debug)]
pub enum LoadError {
    #[error(transparent)]
    ResourceError(#[from] ResourceError),

    #[error(transparent)]
    ResourceReferenceError(#[from] ResourceReferenceError),

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


#[derive(Debug, Getters, CopyGetters, Setters)]
pub struct LoadConfiguration {
    
    #[getset(get = "pub", set = "pub")]
    input_location: PathBuf,

    #[getset(get_copy = "pub", set = "pub")]
    strict_focus_block_check: bool,

    #[getset(get_copy = "pub", set = "pub")]
    strict_paragraphs_loading_rules_check: bool,
}

impl Default for LoadConfiguration {
    fn default() -> Self {
        Self {
            input_location: PathBuf::from("."),
            strict_focus_block_check: false,
            strict_paragraphs_loading_rules_check: true,
        }
    }
}


#[derive(Debug, Getters, Setters, Default, Clone)]
pub struct LoadConfigurationOverLay {

    #[getset(get = "pub", set = "pub")]
    dossier_name: Option<String>,

    #[getset(get = "pub", set = "pub")]
    document_name: Option<String>,
}
