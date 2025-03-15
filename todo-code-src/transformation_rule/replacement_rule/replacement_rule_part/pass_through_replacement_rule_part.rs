use getset::{Getters, Setters};
use regex::Captures;
use crate::{base_parameter::output_format::OutputFormat, compilation::{compilation_configuration::{compilation_configuration_overlay::CompilationConfigurationOverLay, CompilationConfiguration}, compilation_error::CompilationError}, text_compiler::text::Text};

use super::ReplacementRuleReplacerPart;



/// Its `compile` method returns the whole input `CompilableText` as result
#[derive(Debug, Getters, Setters)]
pub struct PassThroughReplacementRuleReplacerPart {
}

impl PassThroughReplacementRuleReplacerPart {

    pub fn new() -> Self {
        Self {
        }
    }

}

impl ReplacementRuleReplacerPart for PassThroughReplacementRuleReplacerPart {
    fn compile(&self, _captures: &Captures, compilable: &Text, _format: &OutputFormat, _compilation_configuration: &CompilationConfiguration, _compilation_configuration_overlay: CompilationConfigurationOverLay) -> Result<Text, CompilationError> {
        Ok(compilable.clone())
    }
}