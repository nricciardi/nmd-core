pub mod closure_replacement_rule_part;
pub mod fixed_replacement_rule_part;
pub mod pass_through_replacement_rule_part;
pub mod single_capture_group_replacement_rule_part;


use regex::Captures;
use crate::{compilable_text::CompilableText, compiler::{compilation_configuration::{compilation_configuration_overlay::CompilationConfigurationOverLay, CompilationConfiguration}, compilation_error::CompilationError}, output_format::OutputFormat};

pub trait ReplacementRuleReplacerPart: std::fmt::Debug + Sync + Send {

    /// `&CompilableText` is the original text on which this part will apply the `compile` function,
    /// `captures` are the regex captures groups on compilable text `compilable_content`
    fn compile(&self, captures: &Captures, compilable: &CompilableText, format: &OutputFormat, compilation_configuration: &CompilationConfiguration, compilation_configuration_overlay: CompilationConfigurationOverLay) -> Result<CompilableText, CompilationError>;
}