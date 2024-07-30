pub mod bibliography_record;

use std::{collections::BTreeMap, sync::{Arc, RwLock}};

use bibliography_record::BibliographyRecord;
use getset::{Getters, Setters};
use crate::{codex::Codex, compiler::{compilable::{compilation_result_accessor::CompilationResultAccessor, Compilable}, compilation_configuration::{compilation_configuration_overlay::CompilationConfigurationOverLay, CompilationConfiguration}, compilation_error::CompilationError, compilation_result::CompilationResult, Compiler}, dossier::dossier_configuration::dossier_configuration_bibliography::DossierConfigurationBibliography, output_format::OutputFormat, resource::resource_reference::{ResourceReference, ResourceReferenceError}};

pub const BIBLIOGRAPHY_FICTITIOUS_DOCUMENT: &str = "bibliography";



#[derive(Debug, Clone, Getters, Setters)]
pub struct Bibliography {

    #[getset(get = "pub", set = "pub")]
    title: String,

    #[getset(get = "pub", set = "pub")]
    content: BTreeMap<String, BibliographyRecord>,

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
            return Some(ResourceReference::of_internal_from_without_sharp(&target_key, Some(BIBLIOGRAPHY_FICTITIOUS_DOCUMENT)))
        }

        None
    }
}

impl Compilable for Bibliography {
    fn standard_compile(&mut self, _format: &OutputFormat, codex: Arc<Codex>, compilation_configuration: Arc<RwLock<CompilationConfiguration>>, compilation_configuration_overlay: Arc<Option<CompilationConfigurationOverLay>>) -> Result<(), CompilationError> {
        
        log::info!("compiling bibliography...");

        let mut outcome = CompilationResult::new_empty();

        outcome.add_fixed_part(String::from(r#"<section class="bibliography">"#));
        outcome.add_fixed_part(String::from(r#"<div class="bibliography-title">"#));
        outcome.append_compilation_result(&mut Compiler::compile_str(&*codex, &self.title, Arc::clone(&compilation_configuration), Arc::clone(&compilation_configuration_overlay))?);
        outcome.add_fixed_part(String::from(r#"</div>"#));
        outcome.add_fixed_part(String::from(r#"<ul class="bibliography-body">"#));

        for (bib_key, bib_record) in self.content.iter() {
            outcome.add_fixed_part(format!(r#"<div class="bibliography-item" id="{}">"#, ResourceReference::of_internal_from_without_sharp(bib_key, Some(BIBLIOGRAPHY_FICTITIOUS_DOCUMENT))?.build_without_internal_sharp()));
            outcome.add_fixed_part(String::from(r#"<div class="bibliography-item-title">"#));

            outcome.add_fixed_part(bib_record.title().to_string());

            outcome.add_fixed_part(String::from(r#"</div>"#));

            if let Some(authors) = bib_record.authors() {

                outcome.add_fixed_part(String::from(r#"<div class="bibliography-item-authors">"#));
                outcome.add_fixed_part(String::from(authors.join(", ")));
                outcome.add_fixed_part(String::from(r#"</div>"#));
            }

            if let Some(year) = bib_record.year() {

                outcome.add_fixed_part(String::from(r#"<div class="bibliography-item-year">"#));
                outcome.add_fixed_part(String::from(year.to_string()));
                outcome.add_fixed_part(String::from(r#"</div>"#));
            }

            if let Some(url) = bib_record.url() {

                outcome.add_fixed_part(String::from(r#"<div class="bibliography-item-url">"#));
                outcome.add_fixed_part(String::from(url.to_string()));
                outcome.add_fixed_part(String::from(r#"</div>"#));
            }

            if let Some(description) = bib_record.description() {

                outcome.add_fixed_part(String::from(r#"<div class="bibliography-item-description">"#));
                outcome.add_fixed_part(String::from(description.to_string()));
                outcome.add_fixed_part(String::from(r#"</div>"#));
            }

            outcome.add_fixed_part(String::from(r#"</div>"#));
        }

        outcome.add_fixed_part(String::from(r#"</ul>"#));
        outcome.add_fixed_part(String::from(r#"</section>"#));

        self.compilation_result = Some(outcome);

        log::info!("bibliography compiled");

        Ok(())
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