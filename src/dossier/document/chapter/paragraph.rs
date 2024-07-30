use std::{fmt::Display, sync::{Arc, RwLock}};

use getset::{Getters, Setters};
use thiserror::Error;

use crate::{codex::{modifier::ModifierIdentifier, Codex}, compiler::{compilable::{compilation_result_accessor::CompilationResultAccessor, Compilable}, compilation_configuration::{compilation_configuration_overlay::CompilationConfigurationOverLay, CompilationConfiguration}, compilation_error::CompilationError, compilation_result::CompilationResult, Compiler}, output_format::OutputFormat};


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

impl Compilable for Paragraph {
    fn standard_compile(&mut self, format: &OutputFormat, codex: Arc<Codex>, compilation_configuration: Arc<RwLock<CompilationConfiguration>>, compilation_configuration_overlay: Arc<Option<CompilationConfigurationOverLay>>) -> Result<(), CompilationError> {

        let codex = codex.clone();

        let compilation_result = Compiler::compile_paragraph(&*codex, self, Arc::clone(&compilation_configuration), compilation_configuration_overlay)?;

        log::debug!("end to parse paragraph:\n{:#?}", compilation_result);

        self.compilation_result = Some(compilation_result);

        Ok(())
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
