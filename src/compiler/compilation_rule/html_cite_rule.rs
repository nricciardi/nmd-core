use std::fmt::Debug;
use regex::{Captures, Regex};
use crate::{codex::modifier::standard_text_modifier::StandardTextModifier, compiler::{compilable::Compilable, compilation_configuration::{compilation_configuration_overlay::CompilationConfigurationOverLay, CompilationConfiguration}, compilation_error::CompilationError, compilation_result::CompilationResult}, output_format::OutputFormat};
use super::CompilationRule;


pub struct HtmlCiteRule {
    search_pattern: String,
    search_pattern_regex: Regex,
}

impl HtmlCiteRule {
    pub fn new() -> Self {
        Self {
            search_pattern: StandardTextModifier::Cite.modifier_pattern(),
            search_pattern_regex: Regex::new(&StandardTextModifier::Cite.modifier_pattern()).unwrap(),
        }
    }
}

impl Debug for HtmlCiteRule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CiteRule").field("searching_pattern", &self.search_pattern).finish()
    }
}

impl CompilationRule for HtmlCiteRule {

    fn search_pattern(&self) -> &String {
        &self.search_pattern
    }

    fn standard_compile(&self, compilable: &Box<dyn Compilable>, _format: &OutputFormat, compilation_configuration: &CompilationConfiguration, _compilation_configuration_overlay: CompilationConfigurationOverLay) -> Result<CompilationResult, CompilationError> {
        
        let content = compilable.compilable_content();

        let compiled_content = self.search_pattern_regex.replace_all(content, |capture: &Captures| {

            let bib_key = capture.get(1).unwrap().as_str();

            if let Some(bibliography) = compilation_configuration.bibliography() {
                
                if let Some(n) = bibliography.get_n_from_key(bib_key) {
                    if let Some(reference) = bibliography.get_reference_from_key(bib_key) {
                        if let Ok(reference) = reference {
                            return format!(r#"<a class="cite" href="{}">{}</a>"#, reference.build(), n);
                        }
                    }
                }

                log::error!("bibliography record with key: '{}' ('{}') not found: no replacement will be applied", bib_key, capture.get(0).unwrap().as_str());
                
            } else {

                log::error!("bibliography '{}' ('{}') not found: no replacement will be applied", bib_key, capture.get(0).unwrap().as_str());
            }

            return String::from(content);
        });

        Ok(CompilationResult::new_fixed(compiled_content.to_string()))
    }
    
    fn search_pattern_regex(&self) -> &Regex {
        &self.search_pattern_regex
    }
}
