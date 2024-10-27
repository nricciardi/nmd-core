pub mod paragraph;
pub mod heading;
pub mod chapter_tag;
pub mod chapter_header;


use chapter_header::ChapterHeader;
use getset::{Getters, MutGetters, Setters};
use paragraph::Paragraph;
use rayon::iter::{IntoParallelRefMutIterator, ParallelIterator};
use serde::Serialize;
use crate::{codex::Codex, compilation::{compilation_configuration::{compilation_configuration_overlay::CompilationConfigurationOverLay, CompilationConfiguration}, compilation_error::CompilationError, compilable::Compilable}, output_format::OutputFormat};


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
    fn standard_compile(&mut self, format: &OutputFormat, codex: &Codex, compilation_configuration: &CompilationConfiguration, compilation_configuration_overlay: CompilationConfigurationOverLay) -> Result<(), CompilationError> {
        
        log::debug!("compile chapter: {:?}", self.header);

        self.header.compile(format, codex, compilation_configuration, compilation_configuration_overlay.clone())?;
        
        // TODO: use chapters style

        if compilation_configuration.parallelization() {

            let maybe_failed = self.paragraphs_mut().par_iter_mut()
                .map(|paragraph| {
                    
                    paragraph.compile(format, codex, compilation_configuration, compilation_configuration_overlay.clone())

                })
                .find_any(|result| result.is_err());
    
            if let Some(result) = maybe_failed {
                return result
            }

        } else {

            let compilation_configuration_overlay = compilation_configuration_overlay.clone();
            
            let maybe_failed = self.paragraphs_mut().iter_mut()
                .map({
                    let compilation_configuration_overlay = compilation_configuration_overlay.clone();

                    move |paragraph| {
                        paragraph.compile(format, codex, compilation_configuration, compilation_configuration_overlay.clone())
                    }
                })
                .find(|result| result.is_err());
    
            if let Some(result) = maybe_failed {
                return result
            }
        }

        Ok(())  
    }
}