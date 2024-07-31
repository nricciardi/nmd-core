use std::fmt::Display;
use getset::{Getters, Setters};
use thiserror::Error;
use crate::{codex::modifier::ModifierIdentifier, compiler::{compilation_result::CompilationResult, compilation_result_accessor::CompilationResultAccessor}};


#[derive(Error, Debug)]
pub enum ParagraphError {
    #[error("creation error")]
    Creation,

    #[error("empty content")]
    Empty
}

pub type ParagraphType = ModifierIdentifier;

#[derive(Debug, Getters, Setters, Clone)]
pub struct Paragraph {

    #[getset(get = "pub", set = "pub")]
    content: String,

    #[getset(set = "pub")]
    compilation_result: Option<CompilationResult>,

    #[getset(get = "pub", set = "pub")]
    paragraph_type: ParagraphType,
}

impl Paragraph {

    pub fn new(content: String, paragraph_type: ParagraphType) -> Self {
        Self {
            content,
            paragraph_type,
            compilation_result: None
        }
    }

    pub fn contains_only_newlines(&self) -> bool {
        self.content.chars().all(|c| c == '\n' || c == '\r')
    }
}

impl CompilationResultAccessor for Paragraph {
    fn compilation_result(&self) -> &Option<CompilationResult> {
        &self.compilation_result
    }
}

impl Display for Paragraph {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.content)
    }
}
