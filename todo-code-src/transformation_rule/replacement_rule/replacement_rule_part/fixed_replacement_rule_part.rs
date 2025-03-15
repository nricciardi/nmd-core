use getset::{Getters, Setters};
use regex::Captures;
use crate::{base_parameter::output_format::OutputFormat, compilation::{compilation_configuration::{compilation_configuration_overlay::CompilationConfigurationOverLay, CompilationConfiguration}, compilation_error::CompilationError}, text_compiler::text::Text};
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
    fn compile(&self, _captures: &Captures, _compilable: &Text, _format: &OutputFormat, _compilation_configuration: &CompilationConfiguration, _compilation_configuration_overlay: CompilationConfigurationOverLay) -> Result<Text, CompilationError> {
        
        // TODO
        todo!()
        // Ok(CompilableText::new(vec![
        //     CompilableTextPart::new(self.content.clone(), CompilableTextPartType::Fixed)
        // ]))
    }
}