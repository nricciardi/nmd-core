pub mod bibliography_record;

use std::collections::BTreeMap;
use bibliography_record::BibliographyRecord;
use getset::{Getters, Setters};
use serde::Serialize;
use crate::{compiler::{compilation_result::CompilationResult, compilation_result_accessor::CompilationResultAccessor}, dossier::dossier_configuration::dossier_configuration_bibliography::DossierConfigurationBibliography, resource::resource_reference::{ResourceReference, ResourceReferenceError}};


pub const BIBLIOGRAPHY_FICTITIOUS_DOCUMENT: &str = "bibliography";


#[derive(Debug, Clone, Getters, Setters, Serialize)]
pub struct Bibliography {

    #[getset(get = "pub", set = "pub")]
    title: String,

    #[getset(get = "pub", set = "pub")]
    content: BTreeMap<String, BibliographyRecord>,

    #[getset(set = "pub")]
    compilation_result: Option<CompilationResult>,
}

impl Bibliography {
    pub fn new(title: String, content: BTreeMap<String, BibliographyRecord>) -> Self {
        Self {
            title,
            content,
            compilation_result: None,
        }
    }

    pub fn get_n_from_key(&self, target_key: &str) -> Option<usize> {
        for (index, key) in self.content.keys().enumerate() {
            if key == target_key {
                return Some(index + 1);
            }
        }

        None
    }

    pub fn get_reference_from_key(&self, target_key: &str) -> Option<Result<ResourceReference, ResourceReferenceError>> {
        if let Some(_) = self.content.get(target_key) {
            return Some(ResourceReference::of_internal_from_without_sharp(&target_key, Some(&BIBLIOGRAPHY_FICTITIOUS_DOCUMENT)))
        }

        None
    }
}

impl From<&DossierConfigurationBibliography> for Bibliography {
    fn from(dcb: &DossierConfigurationBibliography) -> Self {
        Self {
            title: dcb.title().clone(),
            content: dcb.records().clone(),
            compilation_result: None
        }
    }
}

impl CompilationResultAccessor for Bibliography {
    fn compilation_result(&self) -> &Option<CompilationResult> {
        &self.compilation_result
    }
}