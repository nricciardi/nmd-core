use getset::{Getters, Setters};
use regex::{Captures, Regex};
use crate::{codex::modifier::ModifiersBucket, compilable_text::{compilable_text_part::CompilableTextPart, CompilableText}, compiler::{compilation_configuration::{compilation_configuration_overlay::CompilationConfigurationOverLay, CompilationConfiguration}, compilation_error::CompilationError}, output_format::OutputFormat, utility::text_utility};
use super::ReplacementRuleReplacerPart;



#[derive(Debug, Getters, Setters)]
pub struct SingleCaptureGroupReplacementRuleReplacerPart {
    
    capture_group: usize,

    post_replacing: Vec<(Regex, String)>,

    incompatible_modifiers: ModifiersBucket,

}

impl SingleCaptureGroupReplacementRuleReplacerPart {

    pub fn new(capture_group: usize, post_replacing: Vec<(Regex, String)>, incompatible_modifiers: ModifiersBucket,) -> Self {
        Self {
            capture_group,
            post_replacing,
            incompatible_modifiers,
        }
    }

}

impl ReplacementRuleReplacerPart for SingleCaptureGroupReplacementRuleReplacerPart {
    fn compile(&self, captures: &Captures, _compilable: &CompilableText, _format: &OutputFormat, _compilation_configuration: &CompilationConfiguration, _compilation_configuration_overlay: CompilationConfigurationOverLay) -> Result<CompilableText, CompilationError> {
        Ok(CompilableText::from(
            CompilableTextPart::new_compilable(
                text_utility::replace(captures.get(self.capture_group).unwrap().as_str(), &self.post_replacing).to_string(),
                self.incompatible_modifiers.clone()
            )
        ))
    }
}