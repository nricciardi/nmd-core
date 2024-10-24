pub mod document;
pub mod dossier_configuration;


use std::{collections::HashSet, path::PathBuf, time::Instant};
use document::chapter::heading::Heading;
use document::Document;
use getset::{Getters, MutGetters, Setters};
use rayon::iter::{IndexedParallelIterator, IntoParallelRefIterator, IntoParallelRefMutIterator, ParallelIterator};
use thiserror::Error;
use crate::{codex::Codex, compilation::{compilation_configuration::{compilation_configuration_overlay::CompilationConfigurationOverLay, CompilationConfiguration}, compilation_error::CompilationError, self_compile::SelfCompile}, load::{LoadConfiguration, LoadConfigurationOverLay, LoadError}, output_format::OutputFormat, resource::ResourceError};

use self::dossier_configuration::DossierConfiguration;
use super::{bibliography::Bibliography, table_of_contents::TableOfContents};
use serde::Serialize;


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
#[derive(Debug, Getters, MutGetters, Setters, Serialize)]
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

    /// Load dossier from its filesystem path
    pub fn load_dossier_from_path_buf(path_buf: &PathBuf, codex: &Codex, configuration: &LoadConfiguration, configuration_overlay: LoadConfigurationOverLay) -> Result<Dossier, LoadError> {
        let dossier_configuration = DossierConfiguration::try_from(path_buf)?;

        Self::load_dossier_from_dossier_configuration(&dossier_configuration, codex, configuration, configuration_overlay.clone())
    }

    /// Load dossier from its filesystem path considering only a subset of documents
    pub fn load_dossier_from_path_buf_only_documents(path_buf: &PathBuf, only_documents: &HashSet<String>, codex: &Codex, configuration: &LoadConfiguration, configuration_overlay: LoadConfigurationOverLay) -> Result<Dossier, LoadError> {
        let mut dossier_configuration = DossierConfiguration::try_from(path_buf)?;

        let d: Vec<String> = dossier_configuration.raw_documents_paths().iter()
                                                    .filter(|item| {

                                                        let file_name = PathBuf::from(*item).file_name().unwrap().to_string_lossy().to_string();

                                                        only_documents.contains(file_name.as_str())
                                                    })
                                                    .map(|item| item.clone())
                                                    .collect();

        dossier_configuration.set_raw_documents_paths(d);

        let mut configuration_overlay = configuration_overlay.clone();

        configuration_overlay.set_dossier_name(Some(dossier_configuration.name().clone()));

        Self::load_dossier_from_dossier_configuration(&dossier_configuration, codex, configuration, configuration_overlay)
    }

    /// Load dossier from its dossier configuration
    pub fn load_dossier_from_dossier_configuration(dossier_configuration: &DossierConfiguration, codex: &Codex, configuration: &LoadConfiguration, configuration_overlay: LoadConfigurationOverLay) -> Result<Dossier, LoadError> {

        // TODO: are really mandatory?
        if dossier_configuration.documents_paths().is_empty() {
            return Err(LoadError::ResourceError(ResourceError::InvalidResourceVerbose("there are no documents".to_string())))
        }

        // TODO: is really mandatory?
        if dossier_configuration.name().is_empty() {
            return Err(LoadError::ResourceError(ResourceError::InvalidResourceVerbose("there is no name".to_string())))
        }

        if dossier_configuration.compilation().parallelization() {

            let mut documents_res: Vec<Result<Document, LoadError>> = Vec::new();

            dossier_configuration.documents_paths().par_iter()
            .map(|document_path| {
                Document::load_document_from_path(&PathBuf::from(document_path), codex, configuration, configuration_overlay.clone())
            }).collect_into_vec(&mut documents_res);
            
            let error = documents_res.par_iter().find_any(|result| result.is_err());

            // handle errors
            if let Some(Err(err)) = error.as_ref() {
                return Err(err.clone())
            }

            let documents = documents_res.into_iter().map(|d| d.unwrap()).collect();

            return Ok(Dossier::new(dossier_configuration.clone(), documents))


        } else {

            let mut documents: Vec<Document> = Vec::new();

            for document_path in dossier_configuration.documents_paths() {
    
                let document = Document::load_document_from_path(&PathBuf::from(document_path), codex, configuration, configuration_overlay.clone())?;
    
                documents.push(document)
            }

            return Ok(Dossier::new(dossier_configuration.clone(), documents))
        }
    }
}


impl SelfCompile for Dossier {
    fn standard_compile(&mut self, format: &OutputFormat, codex: &Codex, compilation_configuration: &CompilationConfiguration, mut compilation_configuration_overlay: CompilationConfigurationOverLay) -> Result<(), CompilationError> {
    
        log::info!("compile dossier {} with ({} documents, parallelization: {})", self.name(), self.documents().len(), compilation_configuration.parallelization());

        compilation_configuration_overlay.set_dossier_name(Some(self.name().clone()));

        let fast_draft = compilation_configuration.fast_draft();

        if compilation_configuration.parallelization() {

            let compile_only_documents = compilation_configuration_overlay.compile_only_documents();

            let maybe_fails = self.documents_mut().par_iter_mut()
                .filter(|document| {
                    if fast_draft {
    
                        if let Some(subset) = compile_only_documents {

                            let skip = !subset.contains(document.name());
        
                            if skip {
                                log::info!("document {} compilation is skipped", document.name());
                            }

                            return !skip;
                        }
                    }

                    true
                })
                .map(|document| {

                    let now = Instant::now();

                    let res = document.compile(format, codex, compilation_configuration, compilation_configuration_overlay.clone());

                    log::info!("document '{}' compiled in {} ms", document.name(), now.elapsed().as_millis());

                    res
                })
                .find_any(|result| result.is_err());

                if let Some(Err(fail)) = maybe_fails {
                    return Err(fail)
                }
            
        } else {

            let compile_only_documents = compilation_configuration_overlay.compile_only_documents();

            let maybe_fails = self.documents_mut().iter_mut()
                .filter(|document| {

                    if fast_draft {

                        if let Some(subset) = compile_only_documents {

                            let skip = !subset.contains(document.name());
        
                            if skip {
                                log::info!("document {} compilation is skipped", document.name());
                            }

                            return !skip;
                        }
                    }

                    true
                })
                .map(|document| {
                    let now = Instant::now();

                    let res = document.compile(format, codex, compilation_configuration, compilation_configuration_overlay.clone());

                    log::info!("document '{}' compiled in {} ms", document.name(), now.elapsed().as_millis());

                    res
                })
                .find(|result| result.is_err());

                if let Some(Err(fail)) = maybe_fails {
                    return Err(fail)
                }
        }

        if self.configuration().table_of_contents_configuration().include_in_output() {

            log::info!("dossier table of contents will be included in output");

            let mut headings: Vec<Heading> = Vec::new();

            for document in self.documents() {
                for chapter in document.content().chapters() {
                    headings.push(chapter.header().heading().clone());
                }
            }

            let mut table_of_contents = TableOfContents::new(
                self.configuration().table_of_contents_configuration().title().clone(),
                self.configuration().table_of_contents_configuration().page_numbers(),
                self.configuration().table_of_contents_configuration().plain(),
                self.configuration().table_of_contents_configuration().maximum_heading_level(),
                headings
            );

            table_of_contents.compile(format, codex, compilation_configuration, compilation_configuration_overlay.clone())?;
        
            self.set_table_of_contents(Some(table_of_contents));
        }

        if self.configuration().bibliography().include_in_output() {
            let mut bibliography = Bibliography::new(
                self.configuration().bibliography().title().clone(),
                self.configuration().bibliography().records().clone()
            );

            bibliography.compile(format, codex, compilation_configuration, compilation_configuration_overlay.clone())?;
        
            self.set_bibliography(Some(bibliography));
        }

        Ok(())
    }
} 


#[cfg(test)]
mod test {
    use std::path::PathBuf;


    use crate::{codex::Codex, load::{LoadConfiguration, LoadConfigurationOverLay}};

    use super::Dossier;



    #[test]
    fn load_dossier() {

        let dossier_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("test-resources").join("nmd-test-dossier-1");

        let codex = Codex::of_html();

        let loader_configuration = LoadConfiguration::default();

        let _dossier = Dossier::load_dossier_from_path_buf(&dossier_path, &codex, &loader_configuration, LoadConfigurationOverLay::default()).unwrap();
    }
}