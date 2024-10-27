pub mod chapter;


use std::path::PathBuf;

pub use chapter::Chapter;
use getset::{Getters, MutGetters, Setters};
use rayon::slice::ParallelSliceMut;
use serde::Serialize;
use thiserror::Error;
use crate::codex::Codex;
use crate::compilation::compilation_configuration::compilation_configuration_overlay::CompilationConfigurationOverLay;
use crate::compilation::compilation_configuration::CompilationConfiguration;
use crate::compilation::compilation_error::CompilationError;
use crate::compilation::compilable::Compilable;
use crate::compilation::compilation_outcome::CompilationOutcome;
use crate::content_bundle::ContentBundle;
use crate::load::{LoadConfiguration, LoadConfigurationOverLay, LoadError};
use crate::load_block::LoadBlock;
use crate::output_format::OutputFormat;
use crate::resource::disk_resource::DiskResource;
use crate::resource::{Resource, ResourceError};
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

    pub fn load_document_from_str(document_name: &str, content: &str, codex: &Codex, configuration: &LoadConfiguration, mut configuration_overlay: LoadConfigurationOverLay) -> Result<Document, LoadError> {

        log::info!("loading document '{}' from its content...", document_name);

        configuration_overlay.set_document_name(Some(document_name.to_string()));
        
        let mut blocks: Vec<LoadBlock> = LoadBlock::load_from_str(content, codex, configuration, configuration_overlay.clone())?;

        blocks.par_sort_by(|a, b| a.start().cmp(&b.start()));

        let document = Self::create_document_by_blocks(document_name, blocks)?;

        log::info!("document '{}' loaded (preamble: {}, chapters: {})", document_name, document.content().preamble().is_empty(), document.content().chapters().len());

        Ok(document)      
    }

    /// Load a document from its path (`PathBuf`). The document have to exist.
    pub fn load_document_from_path(path_buf: &PathBuf, codex: &Codex, configuration: &LoadConfiguration, configuration_overlay: LoadConfigurationOverLay) -> Result<Document, LoadError> {

        if !path_buf.exists() {
            return Err(LoadError::ResourceError(ResourceError::InvalidResourceVerbose(format!("{} not exists", path_buf.to_string_lossy())))) 
        }

        let resource = DiskResource::try_from(path_buf.clone())?;

        let content = resource.content()?;

        let document_name = resource.name();

        match Self::load_document_from_str(document_name, &content, codex, configuration, configuration_overlay.clone()) {
            Ok(document) => {
                return Ok(document)
            },
            Err(err) => return Err(LoadError::ElaborationError(err.to_string()))
        }
    }

    fn create_document_by_blocks(document_name: &str, blocks: Vec<LoadBlock>) -> Result<Document, LoadError> {

        log::debug!("create document '{}' using blocks: {:#?}", document_name, blocks);

        let content = ContentBundle::from(blocks);

        let document = Document::new(document_name.to_string(), content);

        log::debug!("document '{}' has {} chapters and preamble {}", document.name(), document.content().chapters().len(), !document.content().preamble().is_empty());

        Ok(document)
    }
}


impl Compilable for Document {
    fn standard_compile(&mut self, format: &OutputFormat, codex: &Codex, compilation_configuration: &CompilationConfiguration, mut compilation_configuration_overlay: CompilationConfigurationOverLay) -> Result<CompilationOutcome, CompilationError> {

        compilation_configuration_overlay.set_document_name(Some(self.name().clone()));

        self.content.compile(format, codex, compilation_configuration, compilation_configuration_overlay.clone())
    }
}



#[cfg(test)]
mod test {
    use crate::{codex::Codex, dossier::document::Document, load::{LoadConfiguration, LoadConfigurationOverLay}};

    #[test]
    fn chapters_from_str() {

        let codex = Codex::of_html();

        let content: String = 
r#"
preamble

# title 1a

paragraph 1a

## title 2a

paragraph 2a

# title 1b

paragraph 1b
"#.trim().to_string();

        let document = Document::load_document_from_str("test", &content, &codex, &LoadConfiguration::default(), LoadConfigurationOverLay::default()).unwrap();

        assert_eq!(document.content().preamble().len(), 1);

        assert_eq!(document.content().chapters().len(), 3);

    }
}