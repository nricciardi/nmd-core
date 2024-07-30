pub mod document;
pub mod dossier_configuration;

use std::{sync::{Arc, RwLock}, time::Instant};

use document::chapter::heading::Heading;
pub use document::{Document, DocumentError};
use getset::{Getters, Setters};
use rayon::iter::{IntoParallelRefMutIterator, ParallelIterator};
use thiserror::Error;

use crate::{compiler::{compilable::Compilable, compilation_configuration::{compilation_configuration_overlay::CompilationConfigurationOverLay, CompilationConfiguration}, compilation_error::CompilationError}, resource::ResourceError};

use self::dossier_configuration::DossierConfiguration;

use super::{bibliography::Bibliography, codex::Codex, output_format::OutputFormat, table_of_contents::TableOfContents};


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
#[derive(Debug, Getters, Setters)]
pub struct Dossier {

    #[getset(get = "pub", set = "pub")]
    configuration: DossierConfiguration,

    #[getset(get = "pub", set = "pub")]
    table_of_contents: Option<TableOfContents>,

    #[getset(get = "pub", set = "pub")]
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


impl Compilable for Dossier {

    fn standard_compile(&mut self, format: &OutputFormat, codex: Arc<Codex>, compilation_configuration: Arc<RwLock<CompilationConfiguration>>, compilation_configuration_overlay: Arc<Option<CompilationConfigurationOverLay>>) -> Result<(), CompilationError> {

        let parallelization = compilation_configuration.read().unwrap().parallelization();

        log::info!("parse dossier {} with ({} documents, parallelization: {})", self.name(), self.documents().len(), parallelization);

        compilation_configuration.write().unwrap().metadata_mut().set_dossier_name(Some(self.name().clone()));

        if parallelization {

            let maybe_fails = self.documents.par_iter_mut()
                .filter(|document| {

                    if compilation_configuration.read().unwrap().fast_draft() {
                        let pco = compilation_configuration_overlay.clone();
    
                        if let Some(pco) = pco.as_ref() {
    
                            if let Some(subset) = pco.parse_only_documents() {

                                let skip = !subset.contains(document.name());
            
                                if skip {
                                    log::info!("document {} parsing is skipped", document.name());
                                }

                                return !skip;
                            }
                        }
                    }

                    true
                })
                .map(|document| {

                    let parse_time = Instant::now();

                    let new_compilation_configuration: Arc<RwLock<CompilationConfiguration>> = Arc::new(RwLock::new(compilation_configuration.read().unwrap().clone()));

                    // Arc::new because parallelization on (may be override during multi-thread operations)
                    let res = document.compile(format, Arc::clone(&codex), new_compilation_configuration, Arc::clone(&compilation_configuration_overlay));

                    log::info!("document '{}' parsed in {} ms", document.name(), parse_time.elapsed().as_millis());

                    res
                })
                .find_any(|result| result.is_err());

                if let Some(Err(fail)) = maybe_fails {
                    return Err(fail)
                }
            
        } else {
            let maybe_fails = self.documents.iter_mut()
                .filter(|document| {

                    if compilation_configuration.read().unwrap().fast_draft() {
                        let pco = compilation_configuration_overlay.clone();

                        if let Some(pco) = pco.as_ref() {

                            if let Some(subset) = pco.parse_only_documents() {
            
                                
                                let skip = !subset.contains(document.name());
            
                                if skip {
                                    log::info!("document {} parsing is skipped", document.name());
                                }

                                return !skip;            
                            }
                        }
                    }

                    true
                })
                .map(|document| {
                    let parse_time = Instant::now();

                    let res = document.compile(format, Arc::clone(&codex), Arc::clone(&compilation_configuration), Arc::clone(&compilation_configuration_overlay));

                    log::info!("document '{}' parsed in {} ms", document.name(), parse_time.elapsed().as_millis());

                    res
                })
                .find(|result| result.is_err());

                if let Some(Err(fail)) = maybe_fails {
                    return Err(fail)
                }
        }

        if self.configuration.table_of_contents_configuration().include_in_output() {

            log::info!("dossier table of contents will be included in output");

            let mut headings: Vec<Heading> = Vec::new();

            for document in self.documents() {
                for chapter in document.chapters() {
                    headings.push(chapter.heading().clone());
                }
            }

            let mut table_of_contents = TableOfContents::new(
                self.configuration.table_of_contents_configuration().title().clone(),
                self.configuration.table_of_contents_configuration().page_numbers(),
                self.configuration.table_of_contents_configuration().plain(),
                self.configuration.table_of_contents_configuration().maximum_heading_level(),
                headings
            );

            table_of_contents.compile(format, Arc::clone(&codex), Arc::clone(&compilation_configuration), Arc::clone(&compilation_configuration_overlay))?;
        
            self.table_of_contents = Some(table_of_contents);
        }

        if self.configuration.bibliography().include_in_output() {
            let mut bibliography = Bibliography::new(
                self.configuration.bibliography().title().clone(),
                self.configuration.bibliography().records().clone()
            );

            bibliography.compile(format, Arc::clone(&codex), Arc::clone(&compilation_configuration), Arc::clone(&compilation_configuration_overlay))?;
        
            self.bibliography = Some(bibliography);
        }

        Ok(())
    }
}
