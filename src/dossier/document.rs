pub mod chapter;
pub mod content;

use std::path::PathBuf;
use std::time::Instant;
use content::Content;
use getset::{Getters, MutGetters, Setters};
use serde::Serialize;
use thiserror::Error;
use crate::base_parameter::output_format::OutputFormat;
use crate::codex::Codex;
use crate::compilation::compilation_configuration::compilation_configuration_overlay::CompilationConfigurationOverLay;
use crate::compilation::compilation_configuration::CompilationConfiguration;
use crate::compilation::compilation_error::CompilationError;
use crate::compilation::compilable::Compilable;
use crate::compilation::compilation_outcome::CompilationOutcome;
use crate::load::load_configuration::LoadConfiguration;
use crate::load::load_error::LoadError;
use crate::mmo::MultiMediaObjectError;
use crate::utility::datastruct::resource::disk_resource::DiskResource;


#[derive(Error, Debug)]
pub enum DocumentError {
    #[error(transparent)]
    Load(#[from] LoadError),

    #[error(transparent)]
    Compilation(#[from] CompilationError),

}

#[derive(Debug, Getters, MutGetters, Setters, Serialize)]
pub struct Document {

    #[getset(get = "pub", set = "pub")]
    name: String,

    #[getset(get = "pub", get_mut = "pub", set = "pub")]
    content: Content
}


impl Document {

    pub fn new(name: String, content: Content) -> Self {
        
        Self {
            name,
            content,
        }
    }

    pub fn load_document_from_str(document_name: &str, raw_str: &str, codex: &Codex, configuration: &LoadConfiguration) -> Result<Document, LoadError> {
        
        let now = Instant::now();

        log::info!("loading document '{}' from its content...", document_name);

        // configuration.set_document_name(Some(document_name.to_string()));       // TODO: remove from configuration
        
        let content = Content::load_from_str(raw_str, codex, configuration)?;

        let document = Document::new(document_name.to_string(), content);

        log::info!("document '{}' loaded in {} ms (preamble: {}, chapters: {})", document_name, now.elapsed().as_millis(), document.content().preamble().is_empty(), document.content().chapters().len());

        Ok(document)
    }

    /// Load a document from its path (`PathBuf`). The document have to exist.
    pub fn load_document_from_path(path_buf: &PathBuf, codex: &Codex, configuration: &LoadConfiguration) -> Result<Document, LoadError> {

        if !path_buf.exists() {
            return Err(LoadError::ResourceError(MultiMediaObjectError::InvalidResourceVerbose(format!("{} not exists", path_buf.to_string_lossy())))) 
        }

        let now = Instant::now();

        let resource = DiskResource::try_from(path_buf.clone())?;

        log::info!("document in {:?} read in {} ms", path_buf, now.elapsed().as_millis());

        let content = resource.content()?;

        let document_name = resource.name();

        match Self::load_document_from_str(document_name, &content, codex, configuration) {
            Ok(document) => {
                return Ok(document)
            },
            Err(err) => return Err(LoadError::ElaborationError(err.to_string()))
        }
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
    use crate::{codex::Codex, dossier::document::Document, load::load_configuration::LoadConfiguration};

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

        let document = Document::load_document_from_str("test", &content, &codex, &LoadConfiguration::default()).unwrap();

        assert_eq!(document.content().preamble().len(), 1);

        assert_eq!(document.content().chapters().len(), 3);

    }
}