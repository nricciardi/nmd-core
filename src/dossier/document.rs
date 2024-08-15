pub mod chapter;


use chapter::paragraph::ParagraphTrait;
pub use chapter::Chapter;
use getset::{Getters, MutGetters, Setters};
use serde::Serialize;
use thiserror::Error;
use crate::compiler::compilation_error::CompilationError;
use crate::resource::ResourceError;
use self::chapter::paragraph::ParagraphError;
pub use self::chapter::Paragraph;


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
    #[serde(skip)]      // TODO
    preamble: Vec<Box<dyn ParagraphTrait>>,

    #[getset(get = "pub", get_mut = "pub", set = "pub")]
    chapters: Vec<Chapter>
}


impl Document {

    pub fn new(name: String, preamble: Vec<Box<dyn ParagraphTrait>>, chapters: Vec<Chapter>) -> Self {
        
        Self {
            name,
            preamble,
            chapters
        }
    }
}
