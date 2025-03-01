use std::fmt::Debug;
use regex::Regex;
use crate::{codex::modifier::standard_text_modifier::StandardTextModifier, compilable_text::{compilable_text_part::{CompilableTextPart, CompilableTextPartType}, CompilableText}, compilation::compilation_configuration::{compilation_configuration_overlay::CompilationConfigurationOverLay, CompilationConfiguration}, output_format::OutputFormat};
use super::CompilationRule;
use crate::compilation::compilation_error::CompilationError;


pub struct HtmlCiteRule {
    search_pattern: String,
    search_pattern_regex: Regex,
}

impl HtmlCiteRule {
    pub fn new() -> Self {
        Self {
            search_pattern: StandardTextModifier::Cite.modifier_pattern(),
            search_pattern_regex: StandardTextModifier::Cite.modifier_pattern_regex().clone(),
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

    fn standard_compile(&self, compilable: &CompilableText, _format: &OutputFormat, compilation_configuration: &CompilationConfiguration, _compilation_configuration_overlay: CompilationConfigurationOverLay) -> Result<CompilableText, CompilationError> {
        
        let mut compiled_parts = Vec::new();

        for matc in self.search_pattern_regex.captures_iter(&compilable.compilable_content()) {

            let bib_key = matc.get(1).unwrap().as_str();

            if let Some(bibliography) = compilation_configuration.bibliography() {
                if let Some(n) = bibliography.get_n_from_key(bib_key) {
                    if let Some(reference) = bibliography.get_reference_from_key(bib_key) {
                        if let Ok(reference) = reference {
                            
                            let reference_part = CompilableTextPart::new(
                                format!(r#"<a class="cite" href="{}">{}</a>"#, reference.build(), n),
                                CompilableTextPartType::Fixed
                            );
            
                            compiled_parts.push(reference_part);

                            continue;
                        }
                    }
                }

                log::error!("bibliography record with key: '{}' ('{}') not found: no replacement will be applied", bib_key, matc.get(0).unwrap().as_str());
                
                if compilation_configuration.strict_cite_check() {
                    return Err(CompilationError::ElaborationErrorVerbose(format!("bibliography record with key: '{}' ('{}') not found: no replacement will be applied", bib_key, matc.get(0).unwrap().as_str())))
                }

            } else {

                log::error!("bibliography '{}' ('{}') not found: no replacement will be applied", bib_key, matc.get(0).unwrap().as_str());

                if compilation_configuration.strict_cite_check() {
                    return Err(CompilationError::ElaborationErrorVerbose(format!("bibliography '{}' ('{}') not found: no replacement will be applied", bib_key, matc.get(0).unwrap().as_str())))
                }
            }

        }

        Ok(CompilableText::new(compiled_parts))
    }
    
    fn search_pattern_regex(&self) -> &Regex {
        &self.search_pattern_regex
    }
}
