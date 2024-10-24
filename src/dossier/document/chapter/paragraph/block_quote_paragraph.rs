use getset::{Getters, Setters};
use crate::{codex::Codex, compilable_text::{compilable_text_part::CompilableTextPart, CompilableText}, compilation::{compilation_configuration::{compilation_configuration_overlay::CompilationConfigurationOverLay, CompilationConfiguration}, compilation_error::CompilationError, compiled_text_accessor::CompiledTextAccessor, self_compile::SelfCompile}, output_format::OutputFormat, utility::{nmd_unique_identifier::NmdUniqueIdentifier, text_utility}};
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
    compiled_content: Option<CompilableText>,
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

    fn html_standard_compile(&mut self, codex: &Codex, compilation_configuration: &CompilationConfiguration, compilation_configuration_overlay: CompilationConfigurationOverLay) -> Result<(), CompilationError> {
        
        let mut compilation_result = CompilableText::new_empty();

        let mut content = format!(r#"<div class="focus-quote-block focus-quote-block-{}"{}>"#, self.extended_quote_type, text_utility::html_nuid_tag_or_nothing(self.nuid.as_ref()));
        content.push_str(&format!(r#"<div class="focus-quote-block-title focus-quote-block-{}-title"></div>"#, self.extended_quote_type));
        content.push_str(&format!(r#"<div class="focus-quote-block-description focus-quote-block-{}-description">"#, self.extended_quote_type));

        compilation_result.parts_mut().push(CompilableTextPart::new_fixed(content));

        for paragraph in self.paragraphs.iter_mut() {
            paragraph.standard_compile(&OutputFormat::Html, codex, compilation_configuration, compilation_configuration_overlay.clone())?;

            compilation_result.parts_mut().append(&mut paragraph.compiled_text().unwrap().clone().parts_mut());
        }

        compilation_result.parts_mut().push(CompilableTextPart::new_fixed(String::from("</div></div>")));

        self.compiled_content = Some(compilation_result);

        Ok(())
    }
}

impl SelfCompile for ExtendedBlockQuoteParagraph {
    fn standard_compile(&mut self, format: &OutputFormat, codex: &Codex, compilation_configuration: &CompilationConfiguration, compilation_configuration_overlay: CompilationConfigurationOverLay) -> Result<(), CompilationError> {
        
        match format {
            OutputFormat::Html => self.html_standard_compile(codex, compilation_configuration, compilation_configuration_overlay),
        }
    }
}


impl CompiledTextAccessor for ExtendedBlockQuoteParagraph {
    fn compiled_text(&self) -> Option<&CompilableText> {
        self.compiled_content.as_ref()
    }
}

impl Paragraph for ExtendedBlockQuoteParagraph {
    fn raw_content(&self) -> &String {
        &self.raw_content
    }

    fn nuid(&self) -> Option<&NmdUniqueIdentifier> {
        self.nuid.as_ref()
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

    use crate::{codex::Codex, compilation::compilation_configuration::{compilation_configuration_overlay::CompilationConfigurationOverLay, CompilationConfiguration}, load::{loader_configuration::{LoaderConfiguration, LoaderConfigurationOverLay}, paragraph_loading_rule::{block_quote_paragraph_loading_rule::BlockQuoteParagraphLoadingRule, ParagraphLoadingRule}}, output_format::OutputFormat};

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

        let mut paragraph = rule.load(&nmd_text, &codex, &LoaderConfiguration::default(), LoaderConfigurationOverLay::default()).unwrap();    
    
        paragraph.compile(&OutputFormat::Html, &codex, &CompilationConfiguration::default(), CompilationConfigurationOverLay::default()).unwrap();
    }

}


