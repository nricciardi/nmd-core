pub mod paragraph;
pub mod heading;
pub mod chapter_builder;
pub mod chapter_tag;

use std::fmt::Display;
use std::sync::{Arc, RwLock};
use std::thread;

use chapter_tag::ChapterTag;
use getset::{Getters, Setters};
use rayon::iter::{IntoParallelRefMutIterator, ParallelIterator};


use crate::codex::Codex;
use crate::compiler::compilable::Compilable;
use crate::compiler::compilation_configuration::compilation_configuration_overlay::CompilationConfigurationOverLay;
use crate::compiler::compilation_configuration::CompilationConfiguration;
use crate::compiler::compilation_error::CompilationError;
use crate::output_format::OutputFormat;

use self::heading::Heading;
pub use self::paragraph::Paragraph;


#[derive(Debug, Getters, Setters)]
pub struct Chapter {

    #[getset(get = "pub", set = "pub")]
    heading: Heading,

    #[getset(get = "pub", set = "pub")]
    tags: Vec<ChapterTag>,
    
    #[getset(get = "pub", set = "pub")]
    paragraphs: Vec<Paragraph>,
}


impl Chapter {

    pub fn new(heading: Heading, tags: Vec<ChapterTag>, paragraphs: Vec<Paragraph>) -> Self {
        Self {
            heading,
            tags,
            paragraphs
        }
    }
}


impl Compilable for Chapter {
    fn standard_compile(&mut self, format: &OutputFormat, codex: Arc<Codex>, compilation_configuration: Arc<RwLock<CompilationConfiguration>>, compilation_configuration_overlay: Arc<Option<CompilationConfigurationOverLay>>) -> Result<(), CompilationError> {

        self.heading.compile(format, Arc::clone(&codex), Arc::clone(&compilation_configuration), Arc::clone(&compilation_configuration_overlay))?;

        log::debug!("parsing chapter:\n{:#?}", self);

        if compilation_configuration.read().unwrap().parallelization() {

            let maybe_failed = self.paragraphs.par_iter_mut()
                .map(|paragraph| {
                    paragraph.compile(format, Arc::clone(&codex), Arc::clone(&compilation_configuration), Arc::clone(&compilation_configuration_overlay))
                })
                .find_any(|result| result.is_err());
    
            if let Some(result) = maybe_failed {
                return result
            }

        } else {
            
            let maybe_failed = self.paragraphs.iter_mut()
                .map(|paragraph| {
                    paragraph.compile(format, Arc::clone(&codex), Arc::clone(&compilation_configuration), Arc::clone(&compilation_configuration_overlay))
                })
                .find(|result| result.is_err());
    
            if let Some(result) = maybe_failed {
                return result
            }
        }

        Ok(())
    }
}