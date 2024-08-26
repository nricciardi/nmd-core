use getset::{Getters, Setters};
use crate::{codex::Codex, compilable_text::CompilableText, compiler::{compilation_configuration::{compilation_configuration_overlay::CompilationConfigurationOverLay, CompilationConfiguration}, compilation_error::CompilationError, compilation_result::CompilationResult, compilation_rule::CompilationRule, compiled_text_accessor::CompiledTextAccessor, self_compile::SelfCompile}, dossier::document::chapter::paragraph::Paragraph, output_format::OutputFormat, utility::nmd_unique_identifier::NmdUniqueIdentifier};


#[derive(Debug, Getters, Setters)]
pub struct ReplacementRuleParagraph {

    #[getset(set = "pub")]
    nuid: Option<NmdUniqueIdentifier>,

    #[getset(set = "pub")]
    raw_content: String,

    compilation_rule: Box<dyn CompilationRule>,

    #[getset(set = "pub")]
    compiled_content: Option<CompilableText>,

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
    fn standard_compile(&mut self, format: &OutputFormat, codex: &Codex, compilation_configuration: &CompilationConfiguration, compilation_configuration_overlay: CompilationConfigurationOverLay) -> Result<(), CompilationError> {
        
        // let input: Box<dyn Compilable> = Box::new(Compilable::new(
        //     CompilableContent::from([
        //         CompilationResultPart::new(
        //             self.raw_content.clone(),
        //             CompilationResultPartType::Compilable { incompatible_modifiers: ModifiersBucket::None }
        //         )
        //     ]),
        //     self.nuid.clone()
        // ));

        // let mut compilation_result = self.compilation_rule.compile(&input, format, compilation_configuration, compilation_configuration_overlay.clone())?;
        
        // compilation_result.apply_compile_function(|mutable_part| Compiler::compile_str(&mutable_part.content(), format, codex, compilation_configuration, compilation_configuration_overlay.clone()))?;

        // self.compiled_content = Some(compilation_result);

        // Ok(())

        todo!()
    }
}


impl CompiledTextAccessor for ReplacementRuleParagraph {
    fn compiled_text(&self) -> Option<&CompilableText> {
        self.compiled_content.as_ref()
    }
}

impl Paragraph for ReplacementRuleParagraph {
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

