use getset::{Getters, Setters};
use rayon::iter::{IntoParallelRefMutIterator, ParallelIterator};
use regex::{Captures, Regex};
use crate::{base_parameter::output_format::OutputFormat, codex::modifier::ModifiersBucket, compilation::{compilation_configuration::{compilation_configuration_overlay::CompilationConfigurationOverLay, CompilationConfiguration}, compilation_error::CompilationError}, text_compiler::text::{text_part::TextPart, Text}, utility::text_utility};
use super::ReplacementRuleReplacerPart;



#[derive(Debug, Getters, Setters)]
pub struct SingleCaptureGroupReplacementRuleReplacerPart {
    
    capture_group: usize,

    post_replacing: Vec<(Regex, String)>,

    incompatible_modifiers: ModifiersBucket,
}

impl SingleCaptureGroupReplacementRuleReplacerPart {

    pub fn new(capture_group: usize, post_replacing: Vec<(Regex, String)>, incompatible_modifiers: ModifiersBucket,) -> Self {
        Self {
            capture_group,
            post_replacing,
            incompatible_modifiers,
        }
    }

    pub fn with_incompatible_modifiers(mut self, incompatible_modifiers: ModifiersBucket,) -> Self {
        self.incompatible_modifiers = incompatible_modifiers;

        self
    }
}

impl ReplacementRuleReplacerPart for SingleCaptureGroupReplacementRuleReplacerPart {
    fn compile(&self, captures: &Captures, compilable: &Text, _format: &OutputFormat, _compilation_configuration: &CompilationConfiguration, _compilation_configuration_overlay: CompilationConfigurationOverLay) -> Result<Text, CompilationError> {
        
        if let Some(capture) = captures.get(self.capture_group) {

            let mut slice = compilable.parts_slice(capture.start(), capture.end())?;

            slice.par_iter_mut().for_each(|part| {
    
                if let TextPart::Compilable{ content, incompatible_modifiers } = part.part_type() {
    
                    let incompatible_modifiers = incompatible_modifiers.clone().extend(&self.incompatible_modifiers);
    
                    let new_content = text_utility::replace(part.content(), &self.post_replacing);
        
                    part.set_part_type(TextPart::Compilable { content: new_content, incompatible_modifiers });
    
                }
    
            });
    
            return Ok(Text::new(slice))
        
        } else {

            return Err(CompilationError::ElaborationErrorVerbose(format!("capture group n. {} not found in {:?} of compilable {}", self.capture_group, captures, compilable.compilable_content())))
        }
                
        
    }
}

impl From<usize> for SingleCaptureGroupReplacementRuleReplacerPart {
    fn from(value: usize) -> Self {
        Self::new(value, Vec::new(), ModifiersBucket::None)
    }
}