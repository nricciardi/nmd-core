pub mod transformation_configuration;
pub mod transformation_error;
pub mod replacement_rule;
pub mod html_greek_letter_rule;
pub mod reference_rule;
pub mod html_cite_rule;
pub mod constants;


use std::fmt::Debug;
use regex::{Match, Regex};
use transformation_configuration::TransformationConfiguration;
use transformation_error::TransformationError;
use crate::mmo::bucket::Bucket;

use super::Text;


pub type TextTransformationRuleIdentifier = String;


pub trait TextTransformationRule: Send + Sync + Debug {

    fn identifier(&self) -> &TextTransformationRuleIdentifier;

    // TODO: abstract and remove these

    // fn search_pattern(&self) -> &String;

    // fn search_pattern_regex(&self) -> &Regex;

    // fn is_match(&self, content: &str) -> bool {

    //     self.search_pattern_regex().is_match(content)
    // }

    // fn find_iter<'r, 'h>(&'r self, content: &'h str) -> Vec<Match<'h>> {
    //     self.search_pattern_regex().find_iter(content).collect()
    // }

    fn incompatible_rules(&self) -> &Bucket<TextTransformationRuleIdentifier>;

    fn apply(&self, text: &mut Text, configuration: &dyn TransformationConfiguration) -> Result<(), TransformationError>;

    // /// Compile string
    // fn standard_compile(&self, compilable: &CompilableString, format: &OutputFormat, compilation_configuration: &CompilationConfiguration, compilation_configuration_overlay: CompilationConfigurationOverLay) -> Result<CompilableString, CompilationError>;

    // /// Compile string avoid time consuming operations (incomplete compilation)
    // fn fast_compile(&self, compilable: &CompilableString, format: &OutputFormat,  compilation_configuration: &CompilationConfiguration, compilation_configuration_overlay: CompilationConfigurationOverLay) -> Result<CompilableString, CompilationError> {
    //     self.standard_compile(compilable, format, compilation_configuration, compilation_configuration_overlay)
    // }

    // /// Standard or fast compilation based on `CompilationConfiguration` `fast_draft()`
    // fn compile(&self, compilable: &CompilableString, format: &OutputFormat, compilation_configuration: &CompilationConfiguration, compilation_configuration_overlay: CompilationConfigurationOverLay) -> Result<CompilableString, CompilationError> {

    //     if compilation_configuration.fast_draft() {
    //         return self.fast_compile(compilable, format, compilation_configuration, compilation_configuration_overlay)
    //     }

    //     self.standard_compile(compilable, format, compilation_configuration, compilation_configuration_overlay)
    // }

}