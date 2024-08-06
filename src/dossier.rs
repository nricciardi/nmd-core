pub mod document;
pub mod dossier_configuration;

pub use document::{Document, DocumentError};
use getset::{Getters, MutGetters, Setters};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::resource::ResourceError;

use self::dossier_configuration::DossierConfiguration;

use super::{bibliography::Bibliography, table_of_contents::TableOfContents};


pub const ASSETS_DIR: &str = "assets";
pub const IMAGES_DIR: &str = "images";
pub const DOCUMENTS_DIR: &str = "documents";
pub const STYLES_DIR: &str = "styles";


#[derive(Error, Debug)]
pub enum DossierError {
    #[error("dossier loading failed: '{0}'")]
    Load(#[from] ResourceError)
}


/// NMD Dossier struct. It has own documents list
#[derive(Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
pub struct Dossier {

    #[getset(get = "pub", set = "pub")]
    configuration: DossierConfiguration,

    #[getset(get = "pub", set = "pub")]
    table_of_contents: Option<TableOfContents>,

    #[getset(get = "pub", set = "pub", get_mut = "pub")]
    documents: Vec<Document>,

    #[getset(get = "pub", set = "pub")]
    bibliography: Option<Bibliography>,
}

impl Dossier {

    pub fn new(configuration: DossierConfiguration, documents: Vec<Document>) -> Self {

        Self {
            configuration,
            table_of_contents: None,
            documents,
            bibliography: None,
        }
    }

    pub fn name(&self) -> &String {
        self.configuration.name()
    }

    /// replace document by name if it is found
    pub fn replace_document(&mut self, document_name: &str, new_document: Document) {
        let index = self.documents.iter().position(|document| document.name().eq(document_name));

        if let Some(index) = index {
            self.documents[index] = new_document;
        }
    }
}