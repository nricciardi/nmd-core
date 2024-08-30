use std::fmt::Debug;
use regex::Regex;
use crate::compilable_text::compilable_text_part::{CompilableTextPart, CompilableTextPartType};
use crate::compilable_text::CompilableText;
use crate::{codex::modifier::standard_text_modifier::StandardTextModifier, compiler::compilation_configuration::{compilation_configuration_overlay::CompilationConfigurationOverLay, CompilationConfiguration}, output_format::OutputFormat};
use super::CompilationRule;
use crate::compiler::compilation_error::CompilationError;


pub struct ReferenceRule {
    search_pattern: String,
    search_pattern_regex: Regex,
}

impl ReferenceRule {
    pub fn new() -> Self {
        Self {
            search_pattern: StandardTextModifier::Reference.modifier_pattern(),
            search_pattern_regex: StandardTextModifier::Reference.modifier_pattern_regex().clone(),
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

    fn standard_compile(&self, compilable: &CompilableText, _format: &OutputFormat, compilation_configuration: &CompilationConfiguration, _compilation_configuration_overlay: CompilationConfigurationOverLay) -> Result<CompilableText, CompilationError> {

        let mut compiled_parts = Vec::new();

        for matc in self.search_pattern_regex.captures_iter(&compilable.compilable_content()) {

            let reference_key = matc.get(1).unwrap().as_str();

            if let Some(reference) = compilation_configuration.references().get(reference_key) {

                let reference_part = CompilableTextPart::new(
                    reference.clone(),
                    CompilableTextPartType::Fixed
                );

                compiled_parts.push(reference_part);

            } else {

                log::error!("reference '{}' ('{}') not found: no replacement will be applied", reference_key, matc.get(0).unwrap().as_str());

                if compilation_configuration.strict_reference_check() {
                    return Err(CompilationError::ElaborationErrorVerbose(format!("reference '{}' ('{}') not found: no replacement will be applied", reference_key, matc.get(0).unwrap().as_str())))
                }
            }

        }

        Ok(CompilableText::new(compiled_parts))
    }
    
    fn search_pattern_regex(&self) -> &Regex {
        &self.search_pattern_regex
    }
}
