pub mod compiled_content_accessor;

use std::sync::{Arc, RwLock};
use crate::{codex::Codex, output_format::OutputFormat};

use super::{compilation_configuration::{compilation_configuration_overlay::CompilationConfigurationOverLay, CompilationConfiguration}, compilation_error::CompilationError};

pub trait Compilable {

    /// Standard parse, using complete rules
    fn standard_compile(&mut self, format: &OutputFormat, codex: Arc<Codex>, parsing_configuration: Arc<RwLock<CompilationConfiguration>>,
        parsing_configuration_overlay: Arc<Option<CompilationConfigurationOverLay>>) -> Result<(), CompilationError>;

    /// Fast parse, reduce parsing time, but its result is incomplete
    fn fast_compile(&mut self, format: &OutputFormat, codex: Arc<Codex>, parsing_configuration: Arc<RwLock<CompilationConfiguration>>,
        parsing_configuration_overlay: Arc<Option<CompilationConfigurationOverLay>>) -> Result<(), CompilationError> {
            self.standard_compile(format, codex, parsing_configuration, parsing_configuration_overlay)
    }

    /// `standard_parse` or `fast_parse` based on parsing configuration `fast_draft()` value
    fn compile(&mut self, format: &OutputFormat, codex: Arc<Codex>, parsing_configuration: Arc<RwLock<CompilationConfiguration>>,
        parsing_configuration_overlay: Arc<Option<CompilationConfigurationOverLay>>) -> Result<(), CompilationError> {
            
        if parsing_configuration.read().unwrap().fast_draft() {
            return self.fast_compile(format, codex, parsing_configuration, parsing_configuration_overlay)
        }

        self.standard_compile(format, codex, parsing_configuration, parsing_configuration_overlay)
    }
}