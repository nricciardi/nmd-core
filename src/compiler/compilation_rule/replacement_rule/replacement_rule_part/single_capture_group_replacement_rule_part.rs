use getset::{Getters, Setters};
use rayon::iter::{IntoParallelRefMutIterator, ParallelIterator};
use regex::{Captures, Regex};
use crate::{codex::modifier::ModifiersBucket, compilable_text::{compilable_text_part::CompilableTextPartType, CompilableText}, compiler::{compilation_configuration::{compilation_configuration_overlay::CompilationConfigurationOverLay, CompilationConfiguration}, compilation_error::CompilationError}, output_format::OutputFormat, utility::text_utility};
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
    fn compile(&self, captures: &Captures, compilable: &CompilableText, _format: &OutputFormat, _compilation_configuration: &CompilationConfiguration, _compilation_configuration_overlay: CompilationConfigurationOverLay) -> Result<CompilableText, CompilationError> {
        
        let capture = captures.get(self.capture_group).unwrap();
                
        let mut slice = compilable.parts_slice(capture.start(), capture.end())?;

        slice.par_iter_mut().for_each(|part| {

            if let CompilableTextPartType::Compilable{ incompatible_modifiers } = part.part_type() {

                let incompatible_modifiers = incompatible_modifiers.clone().extend(&self.incompatible_modifiers);

                part.set_part_type(CompilableTextPartType::Compilable { incompatible_modifiers });

                let new_content = text_utility::replace(part.content(), &self.post_replacing);
    
                part.set_content(new_content);

            }

        });

        Ok(CompilableText::new(slice))
    }
}

impl From<usize> for SingleCaptureGroupReplacementRuleReplacerPart {
    fn from(value: usize) -> Self {
        Self::new(value, Vec::new(), ModifiersBucket::None)
    }
}