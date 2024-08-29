use getset::{Getters, Setters};
use regex::Captures;
use crate::{compilable_text::CompilableText, compiler::{compilation_configuration::{compilation_configuration_overlay::CompilationConfigurationOverLay, CompilationConfiguration}, compilation_error::CompilationError}, output_format::OutputFormat};
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
    fn compile(&self, _captures: &Captures, compilable: &CompilableText, _format: &OutputFormat, _compilation_configuration: &CompilationConfiguration, _compilation_configuration_overlay: CompilationConfigurationOverLay) -> Result<CompilableText, CompilationError> {
        Ok(compilable.clone())
    }
}