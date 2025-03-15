use std::sync::Arc;

use regex::Captures;

use crate::{base_parameter::output_format::OutputFormat, compilation::{compilation_configuration::{compilation_configuration_overlay::CompilationConfigurationOverLay, CompilationConfiguration}, compilation_error::CompilationError}, text_compiler::text::Text};

use super::ReplacementRuleReplacerPart;


type Closure = Arc<dyn Sync + Send + Fn(&Captures, &Text, &OutputFormat, &CompilationConfiguration, CompilationConfigurationOverLay) -> Result<Text, CompilationError>>;


#[derive(Clone)]
pub struct ClosureReplacementRuleReplacerPart {

    closure: Closure,
}

impl ClosureReplacementRuleReplacerPart {

    pub fn new(closure: Closure) -> Self {
        Self {
            closure
        }
    }

}

impl std::fmt::Debug for ClosureReplacementRuleReplacerPart {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ClosureReplacementRuleReplacerPart").finish()
    }
}

impl ReplacementRuleReplacerPart for ClosureReplacementRuleReplacerPart {
    fn compile(&self, captures: &Captures, compilable: &Text, format: &OutputFormat, compilation_configuration: &CompilationConfiguration, compilation_configuration_overlay: CompilationConfigurationOverLay) -> Result<Text, CompilationError> {
        (self.closure)(captures, compilable, format, compilation_configuration, compilation_configuration_overlay.clone())
    }
}