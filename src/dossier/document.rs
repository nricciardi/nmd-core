pub mod chapter;


pub use chapter::Chapter;
use getset::{Getters, MutGetters, Setters};
use rayon::iter::{IntoParallelRefMutIterator, ParallelIterator};
use serde::Serialize;
use thiserror::Error;
use crate::codex::Codex;
use crate::compilation::compilation_configuration::compilation_configuration_overlay::CompilationConfigurationOverLay;
use crate::compilation::compilation_configuration::CompilationConfiguration;
use crate::compilation::compilation_error::CompilationError;
use crate::compilation::self_compile::SelfCompile;
use crate::content_bundle::ContentBundle;
use crate::output_format::OutputFormat;
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


impl SelfCompile for Document {
    fn standard_compile(&mut self, format: &OutputFormat, codex: &Codex, compilation_configuration: &CompilationConfiguration, mut compilation_configuration_overlay: CompilationConfigurationOverLay) -> Result<(), CompilationError> {
        let parallelization = compilation_configuration.parallelization();

        log::info!("compile {} chapters of document: '{}'", self.content().chapters().len(), self.name());

        compilation_configuration_overlay.set_document_name(Some(self.name().clone()));

        if parallelization {

            let maybe_one_failed: Option<Result<(), CompilationError>> = self.content_mut().preamble_mut().par_iter_mut()
                .map(|paragraph| {

                    paragraph.compile(format, codex, compilation_configuration, compilation_configuration_overlay.clone())
                
                }).find_any(|result| result.is_err());

            if let Some(result) = maybe_one_failed {
                return result;
            }

            let maybe_one_failed: Option<Result<(), CompilationError>> = self.content_mut().chapters_mut().par_iter_mut()
                .map(|chapter| {

                    chapter.compile(format, codex, compilation_configuration, compilation_configuration_overlay.clone())
                
                }).find_any(|result| result.is_err());

            if let Some(result) = maybe_one_failed {
                return result;
            }
        
        } else {

            let maybe_one_failed: Option<Result<(), CompilationError>> = self.content_mut().preamble_mut().iter_mut()
                .map(|paragraph| {

                    paragraph.compile(format, codex, compilation_configuration, compilation_configuration_overlay.clone())

                }).find(|result| result.is_err());

            if let Some(result) = maybe_one_failed {
                return result;
            }
            
            let maybe_one_failed: Option<Result<(), CompilationError>> = self.content_mut().chapters_mut().iter_mut()
                .map(|chapter| {

                    chapter.compile(format, codex, compilation_configuration, compilation_configuration_overlay.clone())
                                    
                }).find(|result| result.is_err());

            if let Some(result) = maybe_one_failed {
                return result;
            }
        }

        Ok(())

    }
}