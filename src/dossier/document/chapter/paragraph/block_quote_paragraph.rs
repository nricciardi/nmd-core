use std::sync::{Arc, RwLock};

use getset::{Getters, Setters};

use crate::{codex::Codex, compiler::{compilation_configuration::{compilation_configuration_overlay::CompilationConfigurationOverLay, CompilationConfiguration}, compilation_error::CompilationError, compilation_result::CompilationResult, compilation_result_accessor::CompilationResultAccessor, self_compile::SelfCompile, Compiler}, output_format::OutputFormat, utility::nmd_unique_identifier::NmdUniqueIdentifier};

use super::Paragraph;



#[derive(Debug, Getters, Setters)]
pub struct ExtendedBlockQuoteParagraph {

    #[getset(get = "pub", set = "pub")]
    paragraphs: Vec<Box<dyn Paragraph>>,
    
    #[getset(get = "pub", set = "pub")]
    extended_quote_type: String,

    #[getset(set = "pub")]
    nuid: Option<NmdUniqueIdentifier>,

    #[getset(set = "pub")]
    raw_content: String,

    #[getset(set = "pub")]
    compiled_content: Option<CompilationResult>,
}

impl ExtendedBlockQuoteParagraph {
    
    pub fn new(raw_content: String, extended_quote_type: String, paragraphs: Vec<Box<dyn Paragraph>>) -> Self {
        Self {
            raw_content,
            paragraphs,
            extended_quote_type,
            nuid: None,
            compiled_content: None
        }
    }
}

impl SelfCompile for ExtendedBlockQuoteParagraph {
    fn standard_compile(&mut self, format: &OutputFormat, codex: &Codex, compilation_configuration: &CompilationConfiguration, compilation_configuration_overlay: Arc<RwLock<CompilationConfigurationOverLay>>) -> Result<(), CompilationError> {
        
        let mut compilation_result = CompilationResult::new_empty();

        compilation_result.add_fixed_part(format!(r#"<div class="focus-quote-block focus-quote-block-{}" {}>"#, self.extended_quote_type, self.nuid.as_ref().unwrap_or(&String::new())));
        compilation_result.add_fixed_part(format!(r#"<div class="focus-quote-block-title focus-quote-block-{}-title"></div>"#, self.extended_quote_type));
        compilation_result.add_fixed_part(format!(r#"<div class="focus-quote-block-description focus-quote-block-{}-description">"#, self.extended_quote_type));

        for paragraph in self.paragraphs.iter_mut() {
            paragraph.standard_compile(format, codex, compilation_configuration, compilation_configuration_overlay.clone())?;

            compilation_result.add_fixed_part(paragraph.compilation_result().as_ref().unwrap().content());
        }

        compilation_result.add_fixed_part(String::from("</div></div>"));

        compilation_result.apply_compile_function_to_mutable_parts(|mutable_part| Compiler::compile_str(&mutable_part.content(), &OutputFormat::Html, codex, compilation_configuration, compilation_configuration_overlay.clone()))?;

        self.compiled_content = Some(compilation_result);

        Ok(())
    }
}


impl CompilationResultAccessor for ExtendedBlockQuoteParagraph {
    fn compilation_result(&self) -> &Option<CompilationResult> {
        &self.compiled_content
    }
}

impl Paragraph for ExtendedBlockQuoteParagraph {
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


#[cfg(test)]
mod test {
    use std::sync::{Arc, RwLock};

    use crate::{codex::Codex, compiler::compilation_configuration::{compilation_configuration_overlay::CompilationConfigurationOverLay, CompilationConfiguration}, loader::{loader_configuration::{LoaderConfiguration, LoaderConfigurationOverLay}, paragraph_content_loading_rule::{block_quote_paragraph_loading_rule::BlockQuoteParagraphLoadingRule, ParagraphLoadingRule}}, output_format::OutputFormat};

    #[test]
    fn compile() {
        let nmd_text = concat!(
            "> p1a\n",
            "> p1b\n",
            ">\n",
            "> p2a\n"
        ).to_string();
        
        let codex = Codex::of_html();
        let rule = BlockQuoteParagraphLoadingRule::new();

        let mut paragraph = rule.load(&nmd_text, &codex, &LoaderConfiguration::default(), Arc::new(RwLock::new(LoaderConfigurationOverLay::default()))).unwrap();    
    
        paragraph.compile(&OutputFormat::Html, &codex, &CompilationConfiguration::default(), Arc::new(RwLock::new(CompilationConfigurationOverLay::default()))).unwrap();
    }

}


