pub mod chapter;


pub use chapter::Chapter;
use getset::{Getters, MutGetters, Setters};
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

#[derive(Debug, Getters, MutGetters, Setters)]
pub struct Document {

    #[getset(get = "pub", set = "pub")]
    name: String,

    #[getset(get = "pub", get_mut = "pub", set = "pub")]
    preamble: Vec<Paragraph>,

    #[getset(get = "pub", get_mut = "pub", set = "pub")]
    chapters: Vec<Chapter>
}


impl Document {

    pub fn new(name: String, preamble: Vec<Paragraph>, chapters: Vec<Chapter>) -> Self {
        
        Self {
            name,
            preamble,
            chapters
        }
    }
}
