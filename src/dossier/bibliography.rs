pub mod bibliography_record;

use std::collections::BTreeMap;
use bibliography_record::BibliographyRecord;
use getset::{Getters, Setters};
use serde::Serialize;

use crate::{codex::Codex, compilation::{compilable::Compilable, compilation_configuration::{compilation_configuration_overlay::CompilationConfigurationOverLay, CompilationConfiguration}, compilation_error::CompilationError, compilation_outcome::CompilationOutcome}, mmo::uri::{NUri, UriError}, output_format::OutputFormat, text::compilable_string::{compilable_string_part::CompilableStringPart, CompilableString}};

use super::dossier_configuration::dossier_configuration_bibliography::DossierConfigurationBibliography;


pub const BIBLIOGRAPHY_FICTITIOUS_DOCUMENT: &str = "bibliography";


#[derive(Debug, Clone, Getters, Setters, Serialize)]
pub struct Bibliography {

    #[getset(get = "pub", set = "pub")]
    title: String,

    #[getset(get = "pub", set = "pub")]
    content: BTreeMap<String, BibliographyRecord>,

}

impl Bibliography {
    pub fn new(title: String, content: BTreeMap<String, BibliographyRecord>) -> Self {
        Self {
            title,
            content,
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

    pub fn get_reference_from_key(&self, target_key: &str) -> Option<Result<NUri, UriError>> {
        if let Some(_) = self.content.get(target_key) {
            return Some(NUri::of_internal_from_without_sharp(&target_key, Some(&BIBLIOGRAPHY_FICTITIOUS_DOCUMENT)))
        }

        None
    }
}

impl From<&DossierConfigurationBibliography> for Bibliography {
    fn from(dcb: &DossierConfigurationBibliography) -> Self {
        Self {
            title: dcb.title().clone(),
            content: dcb.records().clone(),
        }
    }
}

impl Compilable for Bibliography {
    fn standard_compile(&mut self, format: &OutputFormat, codex: &Codex, compilation_configuration: &CompilationConfiguration, compilation_configuration_overlay: CompilationConfigurationOverLay) -> Result<CompilationOutcome, CompilationError> {
        log::info!("compiling bibliography...");

        match format {
            OutputFormat::Html => {
                let mut compilation_result = CompilableString::new_empty();

                let mut compiled_title = CompilableStringPart::from(self.title.clone());

                compiled_title.compile(format, codex, compilation_configuration, compilation_configuration_overlay.clone())?;

                compilation_result.parts_mut().push(CompilableStringPart::new_fixed(String::from(r#"<section class="bibliography"><div class="bibliography-title">"#)));
                compilation_result.parts_mut().append(compiled_title.parts_mut());
                compilation_result.parts_mut().push(CompilableStringPart::new_fixed(String::from(r#"</div><ul class="bibliography-body">"#)));
        
                for (bib_key, bib_record) in self.content().iter() {
                    compilation_result.parts_mut().push(CompilableStringPart::new_fixed(format!(
                        r#"<div class="bibliography-item" id="{}"><div class="bibliography-item-title">{}</div>"#,
                        NUri::of_internal_from_without_sharp(bib_key,
                            Some(&BIBLIOGRAPHY_FICTITIOUS_DOCUMENT))?.build_without_internal_sharp(),
                            bib_record.title()
                        )));
                
                    if let Some(authors) = bib_record.authors() {
        
                        compilation_result.parts_mut().push(CompilableStringPart::new_fixed(format!(r#"<div class="bibliography-item-authors">{}</div>"#, authors.join(", "))));
                    }
        
                    if let Some(year) = bib_record.year() {

                        compilation_result.parts_mut().push(CompilableStringPart::new_fixed(format!(r#"<div class="bibliography-item-year">{}</div>"#, year)));
                    }
        
                    if let Some(url) = bib_record.url() {
        
                        compilation_result.parts_mut().push(CompilableStringPart::new_fixed(format!(r#"<div class="bibliography-item-url">{}</div>"#, url)));
                    }
        
                    if let Some(description) = bib_record.description() {

                        compilation_result.parts_mut().push(CompilableStringPart::new_fixed(format!(r#"<div class="bibliography-item-description">{}</div>"#, description)));
        
                    }
        
                    compilation_result.parts_mut().push(CompilableStringPart::new_fixed(String::from(r#"</div>"#)));
                }
        
                compilation_result.parts_mut().push(CompilableStringPart::new_fixed(String::from(r#"</ul></section>"#)));
        
                log::info!("bibliography compiled");
        
                Ok(CompilationOutcome::from(&compilation_result))
            },
        }
    }
}