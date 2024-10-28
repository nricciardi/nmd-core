pub mod paragraph;
pub mod heading;
pub mod chapter_tag;
pub mod chapter_header;


use chapter_header::ChapterHeader;
use getset::{Getters, MutGetters, Setters};
use paragraph::Paragraph;
use rayon::iter::{IntoParallelRefMutIterator, ParallelIterator};
use serde::Serialize;
use crate::{codex::Codex, compilation::{compilable::Compilable, compilation_configuration::{compilation_configuration_overlay::CompilationConfigurationOverLay, CompilationConfiguration}, compilation_error::CompilationError, compilation_outcome::CompilationOutcome}, output_format::OutputFormat};


#[derive(Debug, Getters, MutGetters, Setters, Serialize)]
pub struct Chapter {

    #[getset(get = "pub", get_mut = "pub", set = "pub")]
    header: ChapterHeader,
    
    #[getset(get = "pub", get_mut = "pub", set = "pub")]
    #[serde(skip)]      // TODO
    paragraphs: Vec<Box<dyn Paragraph>>,
}


impl Chapter {

    pub fn new(header: ChapterHeader, paragraphs: Vec<Box<dyn Paragraph>>) -> Self {
        Self {
            header,
            paragraphs
        }
    }    
}


impl Compilable for Chapter {
    fn standard_compile(&mut self, format: &OutputFormat, codex: &Codex, compilation_configuration: &CompilationConfiguration, compilation_configuration_overlay: CompilationConfigurationOverLay) -> Result<CompilationOutcome, CompilationError> {
        
        log::debug!("compile chapter: {:?}", self.header);

        let compiled_heading = self.header.heading_mut().compile(format, codex, compilation_configuration, compilation_configuration_overlay.clone())?;
        
        let mut paragraph_outcomes: Vec<CompilationOutcome> = Vec::new();

        if compilation_configuration.parallelization() {

            let paragraph_results: Vec<Result<CompilationOutcome, CompilationError>> = self.paragraphs.par_iter_mut()
                .map(|paragraph| {

                    paragraph.compile(format, codex, compilation_configuration, compilation_configuration_overlay.clone())
                
                }).collect();

            let mut paragraph_errors: Vec<CompilationError> = Vec::new();

            paragraph_results.into_iter().for_each(|result| {

                match result {
                    Ok(outcome) => paragraph_outcomes.push(outcome),
                    Err(err) => paragraph_errors.push(err),
                }
            });

            if !paragraph_errors.is_empty() {
                return Err(CompilationError::BucketOfErrors(paragraph_errors))
            }

        } else {

            for paragraph in self.paragraphs.iter_mut() {

                paragraph_outcomes.push(paragraph.compile(format, codex, compilation_configuration, compilation_configuration_overlay.clone())?);
            }
        }

        Ok(CompilationOutcome::from(codex.assembler().assemble_chapter(self.header().tags(), &compiled_heading, &paragraph_outcomes, compilation_configuration_overlay.assembler_configuration())?))  
    }
}