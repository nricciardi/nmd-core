pub mod chapter;

use std::fmt::Display;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::{Arc, RwLock};
use std::time::Instant;

pub use chapter::Chapter;
use getset::{Getters, Setters};
use thiserror::Error;
use log;
use rayon::prelude::*;

use crate::codex::Codex;
use crate::compiler::compilable::Compilable;
use crate::compiler::compilation_configuration::compilation_configuration_overlay::CompilationConfigurationOverLay;
use crate::compiler::compilation_configuration::CompilationConfiguration;
use crate::compiler::compilation_error::CompilationError;
use crate::output_format::OutputFormat;
use crate::resource::{Resource, ResourceError};

use self::chapter::paragraph::ParagraphError;
pub use self::chapter::Paragraph;
use self::chapter::chapter_builder::{ChapterBuilder, ChapterBuilderError};


#[derive(Error, Debug)]
pub enum DocumentError {
    #[error(transparent)]
    Load(#[from] ResourceError),

    #[error(transparent)]
    Compilation(#[from] CompilationError),

    #[error(transparent)]
    ChapterBuilderError(#[from] ChapterBuilderError),

    #[error(transparent)]
    ParagraphError(#[from] ParagraphError),
}

#[derive(Debug, Getters, Setters)]
pub struct Document {

    #[getset(get = "pub", set = "pub")]
    name: String,

    #[getset(get = "pub", set = "pub")]
    preamble: Vec<Paragraph>,

    #[getset(get = "pub", set = "pub")]
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

impl Compilable for Document {

    fn standard_compile(&mut self, format: &OutputFormat, codex: Arc<Codex>, compilation_configuration: Arc<RwLock<CompilationConfiguration>>, compilation_configuration_overlay: Arc<Option<CompilationConfigurationOverLay>>) -> Result<(), CompilationError> {

        let parallelization = compilation_configuration.read().unwrap().parallelization();

        log::info!("compile {} chapters of document: '{}'", self.chapters().len(), self.name);

        compilation_configuration.write().unwrap().metadata_mut().set_document_name(Some(self.name().clone()));

        if parallelization {

            let maybe_one_failed: Option<Result<(), CompilationError>> = self.preamble.par_iter_mut()
                .map(|paragraph| {

                    paragraph.compile(format, Arc::clone(&codex), Arc::clone(&compilation_configuration), Arc::clone(&compilation_configuration_overlay))
                
                }).find_any(|result| result.is_err());

            if let Some(result) = maybe_one_failed {
                return result;
            }

            let maybe_one_failed: Option<Result<(), CompilationError>> = self.chapters.par_iter_mut()
                .map(|chapter| {

                    chapter.compile(format, Arc::clone(&codex), Arc::clone(&compilation_configuration), Arc::clone(&compilation_configuration_overlay))
                
                }).find_any(|result| result.is_err());

            if let Some(result) = maybe_one_failed {
                return result;
            }
        
        } else {

            let maybe_one_failed: Option<Result<(), CompilationError>> = self.preamble.iter_mut()
                .map(|paragraph| {

                    paragraph.compile(format, Arc::clone(&codex), Arc::clone(&compilation_configuration), Arc::clone(&compilation_configuration_overlay))
                
                }).find(|result| result.is_err());

            if let Some(result) = maybe_one_failed {
                return result;
            }
            
            let maybe_one_failed: Option<Result<(), CompilationError>> = self.chapters.iter_mut()
                .map(|chapter| {

                    chapter.compile(format, Arc::clone(&codex), Arc::clone(&compilation_configuration), Arc::clone(&compilation_configuration_overlay))
                
                }).find(|result| result.is_err());

            if let Some(result) = maybe_one_failed {
                return result;
            }
        }

        Ok(())

    }
}

