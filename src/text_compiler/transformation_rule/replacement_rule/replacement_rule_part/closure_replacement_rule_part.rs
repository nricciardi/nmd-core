use std::sync::Arc;

use regex::Captures;

use crate::{compilable_text::CompilableText, compilation::{compilation_configuration::{compilation_configuration_overlay::CompilationConfigurationOverLay, CompilationConfiguration}, compilation_error::CompilationError}, output_format::OutputFormat};

use super::ReplacementRuleReplacerPart;


type Closure = Arc<dyn Sync + Send + Fn(&Captures, &CompilableText, &OutputFormat, &CompilationConfiguration, CompilationConfigurationOverLay) -> Result<CompilableText, CompilationError>>;


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
    fn compile(&self, captures: &Captures, compilable: &CompilableText, format: &OutputFormat, compilation_configuration: &CompilationConfiguration, compilation_configuration_overlay: CompilationConfigurationOverLay) -> Result<CompilableText, CompilationError> {
        (self.closure)(captures, compilable, format, compilation_configuration, compilation_configuration_overlay.clone())
    }
}