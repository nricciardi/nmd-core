pub mod chapter;


pub use chapter::Chapter;
use getset::{Getters, MutGetters, Setters};
use serde::Serialize;
use thiserror::Error;
use crate::compiler::content_bundle::ContentBundle;
use crate::compiler::compilation_error::CompilationError;
use crate::resource::ResourceError;
use self::chapter::paragraph::ParagraphError;


#[derive(Error, Debug)]
pub enum DocumentError {
    #[error(transparent)]
    Load(#[from] ResourceError),

    #[error(transparent)]
    Compilation(#[from] CompilationError),

    #[error(transparent)]
    ParagraphError(#[from] ParagraphError),
}

#[derive(Debug, Getters, MutGetters, Setters, Serialize)]
pub struct Document {

    #[getset(get = "pub", set = "pub")]
    name: String,

    #[getset(get = "pub", get_mut = "pub", set = "pub")]
    content: ContentBundle
}


impl Document {

    pub fn new(name: String, content: ContentBundle) -> Self {
        
        Self {
            name,
            content,
        }
    }
}
