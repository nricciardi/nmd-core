use getset::{Getters, Setters};
use regex::Captures;
use crate::{compilable_text::{compilable_text_part::{CompilableTextPart, CompilableTextPartType}, CompilableText}, compiler::{compilation_configuration::{compilation_configuration_overlay::CompilationConfigurationOverLay, CompilationConfiguration}, compilation_error::CompilationError}, output_format::OutputFormat};
use super::ReplacementRuleReplacerPart;



#[derive(Debug, Getters, Setters)]
pub struct FixedReplacementRuleReplacerPart {

    #[getset(get = "pub", set = "pub")]
    content: String
}

impl FixedReplacementRuleReplacerPart {

    pub fn new(content: String) -> Self {
        Self {
            content
        }
    }

}

impl ReplacementRuleReplacerPart for FixedReplacementRuleReplacerPart {
    fn compile(&self, _captures: &Captures, _compilable: &CompilableText, _format: &OutputFormat, _compilation_configuration: &CompilationConfiguration, _compilation_configuration_overlay: CompilationConfigurationOverLay) -> Result<CompilableText, CompilationError> {
        Ok(CompilableText::new(vec![
            CompilableTextPart::new(self.content.clone(), CompilableTextPartType::Fixed)
        ]))
    }
}