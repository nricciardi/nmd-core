use std::sync::{Arc, RwLock};

use getset::{Getters, Setters};
use serde::Serialize;

use crate::{codex::Codex, compiler::{compilation_configuration::{compilation_configuration_overlay::CompilationConfigurationOverLay, CompilationConfiguration}, compilation_error::CompilationError, compilation_result::CompilationResult, compilation_result_accessor::CompilationResultAccessor, self_compile::SelfCompile, Compiler}, dossier::document::chapter::paragraph::ParagraphTrait, output_format::OutputFormat, utility::nmd_unique_identifier::NmdUniqueIdentifier};


#[derive(Debug, Getters, Setters, Serialize)]
pub struct CommonParagraph {

    #[getset(set = "pub")]
    nuid: Option<NmdUniqueIdentifier>,

    #[getset(set = "pub")]
    raw_content: String,

    #[getset(set = "pub")]
    compiled_content: Option<CompilationResult>,

}

impl CommonParagraph {

    pub fn new(raw_content: String) -> Self {
        Self {
            raw_content,
            nuid: None,
            compiled_content: None
        }
    }

}

impl SelfCompile for CommonParagraph {
    fn standard_compile(&mut self, format: &OutputFormat, codex: &Codex, compilation_configuration: &CompilationConfiguration, compilation_configuration_overlay: Arc<RwLock<CompilationConfigurationOverLay>>) -> Result<(), CompilationError> {
        let outcome = Compiler::compile_str(&self.raw_content, format, codex, compilation_configuration, compilation_configuration_overlay.clone())?;
    
        self.compiled_content = Some(outcome);

        Ok(())
    }
}


impl CompilationResultAccessor for CommonParagraph {
    fn compilation_result(&self) -> &Option<CompilationResult> {
        &self.compiled_content
    }
}

impl ParagraphTrait for CommonParagraph {
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

