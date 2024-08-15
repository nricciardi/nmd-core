use std::sync::{Arc, RwLock};
use getset::{Getters, Setters};
use crate::{codex::Codex, compiler::{compilable::{Compilable, GenericCompilable}, compilation_configuration::{compilation_configuration_overlay::CompilationConfigurationOverLay, CompilationConfiguration}, compilation_error::CompilationError, compilation_result::CompilationResult, compilation_result_accessor::CompilationResultAccessor, compilation_rule::CompilationRule, self_compile::SelfCompile, Compiler}, dossier::document::chapter::paragraph::ParagraphTrait, output_format::OutputFormat, utility::nmd_unique_identifier::NmdUniqueIdentifier};


#[derive(Debug, Getters, Setters)]
pub struct ReplacementRuleParagraph {

    #[getset(set = "pub")]
    nuid: Option<NmdUniqueIdentifier>,

    #[getset(set = "pub")]
    raw_content: String,

    compilation_rule: Box<dyn CompilationRule>,

    #[getset(set = "pub")]
    compiled_content: Option<CompilationResult>,

}

impl ReplacementRuleParagraph {

    pub fn new(raw_content: String, compilation_rule: Box<dyn CompilationRule>,) -> Self {
        Self {
            raw_content,
            compilation_rule,
            nuid: None,
            compiled_content: None
        }
    }

}

impl SelfCompile for ReplacementRuleParagraph {
    fn standard_compile(&mut self, format: &OutputFormat, codex: &Codex, compilation_configuration: &CompilationConfiguration, compilation_configuration_overlay: Arc<RwLock<CompilationConfigurationOverLay>>) -> Result<(), CompilationError> {
        
        let input: Box<dyn Compilable> = Box::new(GenericCompilable::new(self.raw_content.clone(), self.nuid.clone()));

        let mut compilation_result = self.compilation_rule.compile(&input, format, codex, compilation_configuration, compilation_configuration_overlay.clone())?;
        
        compilation_result.apply_compile_function_to_mutable_parts(|mutable_part| Compiler::compile_str(&mutable_part.content(), format, codex, compilation_configuration, compilation_configuration_overlay.clone()))?;

        self.compiled_content = Some(compilation_result);

        Ok(())
    }
}


impl CompilationResultAccessor for ReplacementRuleParagraph {
    fn compilation_result(&self) -> &Option<CompilationResult> {
        &self.compiled_content
    }
}

impl ParagraphTrait for ReplacementRuleParagraph {
    fn raw_content(&self) -> &String {
        &self.raw_content
    }

    fn nuid(&self) -> &Option<NmdUniqueIdentifier> {
        &self.nuid
    }
    
    fn set_raw_content(&mut self, raw_content: String) {
        self.raw_content = raw_content;
    }
    
    fn set_nuid(&mut self, nuid: Option<NmdUniqueIdentifier>) {
        self.nuid = nuid;
    }
}

