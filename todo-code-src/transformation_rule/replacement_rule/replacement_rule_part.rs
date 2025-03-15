pub mod closure_replacement_rule_part;
pub mod fixed_replacement_rule_part;
pub mod pass_through_replacement_rule_part;
pub mod single_capture_group_replacement_rule_part;


use regex::Captures;
use crate::{base_parameter::output_format::OutputFormat, compilation::{compilation_configuration::{compilation_configuration_overlay::CompilationConfigurationOverLay, CompilationConfiguration}, compilation_error::CompilationError}, text_compiler::text::Text};

pub trait ReplacementRuleReplacerPart: std::fmt::Debug + Sync + Send {

    /// `compilable` is the original string on which this part will apply the `compile` function,
    /// `captures` are the regex captures groups on compilable text `compilable_content`
    fn compile(&self, captures: &Captures, compilable: &Text, format: &OutputFormat, compilation_configuration: &CompilationConfiguration, compilation_configuration_overlay: CompilationConfigurationOverLay) -> Result<Text, CompilationError>;
}