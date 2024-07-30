pub mod compilation_result_accessor;

use std::sync::{Arc, RwLock};
use crate::{codex::Codex, output_format::OutputFormat};

use super::{compilation_configuration::{compilation_configuration_overlay::CompilationConfigurationOverLay, CompilationConfiguration}, compilation_error::CompilationError};

pub trait Compilable {

    /// Standard compilation, using complete rules
    fn standard_compile(&mut self, format: &OutputFormat, codex: Arc<Codex>, compilation_configuration: Arc<RwLock<CompilationConfiguration>>,
        compilation_configuration_overlay: Arc<Option<CompilationConfigurationOverLay>>) -> Result<(), CompilationError>;

    /// Fast compilation, reduce compilation time, but its result is incomplete
    fn fast_compile(&mut self, format: &OutputFormat, codex: Arc<Codex>, compilation_configuration: Arc<RwLock<CompilationConfiguration>>,
        compilation_configuration_overlay: Arc<Option<CompilationConfigurationOverLay>>) -> Result<(), CompilationError> {
            self.standard_compile(format, codex, compilation_configuration, compilation_configuration_overlay)
    }

    /// `standard_compile` or `fast_compile` based on configuration `fast_draft()` value
    fn compile(&mut self, format: &OutputFormat, codex: Arc<Codex>, compilation_configuration: Arc<RwLock<CompilationConfiguration>>,
        compilation_configuration_overlay: Arc<Option<CompilationConfigurationOverLay>>) -> Result<(), CompilationError> {
            
        if compilation_configuration.read().unwrap().fast_draft() {
            return self.fast_compile(format, codex, compilation_configuration, compilation_configuration_overlay)
        }

        self.standard_compile(format, codex, compilation_configuration, compilation_configuration_overlay)
    }
}