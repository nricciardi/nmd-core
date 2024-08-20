use crate::{codex::Codex, compiler::compilation_configuration::{compilation_configuration_overlay::CompilationConfigurationOverLay, CompilationConfiguration}, output_format::OutputFormat};
use super::compilation_error::CompilationError;


pub trait SelfCompile {
    /// Compile string
    fn standard_compile(&mut self, format: &OutputFormat, codex: &Codex, compilation_configuration: &CompilationConfiguration, compilation_configuration_overlay: CompilationConfigurationOverLay) -> Result<(), CompilationError>;

    /// Compile string avoid time consuming operations (incomplete compilation)
    fn fast_compile(&mut self, format: &OutputFormat, codex: &Codex, compilation_configuration: &CompilationConfiguration, compilation_configuration_overlay: CompilationConfigurationOverLay) -> Result<(), CompilationError> {
        self.standard_compile(format, codex, compilation_configuration, compilation_configuration_overlay)
    }

    /// Standard or fast compilation based on `CompilationConfiguration` `fast_draft()`
    fn compile(&mut self, format: &OutputFormat, codex: &Codex, compilation_configuration: &CompilationConfiguration, compilation_configuration_overlay: CompilationConfigurationOverLay) -> Result<(), CompilationError> {

        if compilation_configuration.fast_draft() {
            return self.fast_compile(format, codex, compilation_configuration, compilation_configuration_overlay)
        }

        self.standard_compile(format, codex, compilation_configuration, compilation_configuration_overlay)
    }
}