use std::fmt::Debug;
use regex::Regex;
use crate::{codex::modifier::standard_text_modifier::StandardTextModifier, compiler::{compilable::Compilable, compilation_configuration::{compilation_configuration_overlay::CompilationConfigurationOverLay, CompilationConfiguration}, compilation_result::{CompilationResultPart, CompilationResultPartType, CompilationResultParts}}, output_format::OutputFormat};
use super::{CompilationRule, CompilationRuleResult};


pub struct ReferenceRule {
    search_pattern: String,
    search_pattern_regex: Regex,
}

impl ReferenceRule {
    pub fn new() -> Self {
        Self {
            search_pattern: StandardTextModifier::Reference.modifier_pattern(),
            search_pattern_regex: Regex::new(&StandardTextModifier::Reference.modifier_pattern()).unwrap(),
        }
    }
}

impl Debug for ReferenceRule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ReferenceRule").field("searching_pattern", &self.search_pattern).finish()
    }
}

impl CompilationRule for ReferenceRule {

    fn search_pattern(&self) -> &String {
        &self.search_pattern
    }

    fn standard_compile(&self, compilable: &Compilable, _format: &OutputFormat, compilation_configuration: &CompilationConfiguration, _compilation_configuration_overlay: CompilationConfigurationOverLay) -> CompilationRuleResult {

        let mut compiled_parts = CompilationResultParts::new();

        for matc in self.search_pattern_regex.captures_iter(&compilable.compilable_content()) {

            let reference_key = matc.get(1).unwrap().as_str();

            if let Some(reference) = compilation_configuration.references().get(reference_key) {

                let reference_part = CompilationResultPart::new(
                    reference.clone(),
                    CompilationResultPartType::Fixed
                );

                compiled_parts.push(reference_part);

            } else {

                log::error!("reference '{}' ('{}') not found: no replacement will be applied", reference_key, matc.get(0).unwrap().as_str());

                // TODO: strict option with panic
            }

        }

        Ok(compiled_parts)

        // let content = compilable.compilable_content();

        // let compilation_result = self.search_pattern_regex.replace_all(content, |capture: &Captures| {

        //     let reference_key = capture.get(1).unwrap().as_str();

        //     if let Some(reference) = compilation_configuration.references().get(reference_key) {
        //         return String::from(reference)
        //     } else {
        //         log::error!("reference '{}' ('{}') not found: no replacement will be applied", reference_key, capture.get(0).unwrap().as_str());
        //         return String::from(content);
        //     }
        // });

        // Ok(CompilationResult::new_fixed(compilation_result.to_string()))
    }
    
    fn search_pattern_regex(&self) -> &Regex {
        &self.search_pattern_regex
    }
}
