use getset::{Getters, Setters};
use crate::{codex::Codex, compilation::{compilable::Compilable, compilation_configuration::{compilation_configuration_overlay::CompilationConfigurationOverLay, CompilationConfiguration}, compilation_error::CompilationError, compilation_outcome::CompilationOutcome}, content_bundle::ContentBundle, output_format::OutputFormat, utility::{nmd_unique_identifier::NmdUniqueIdentifier, text_utility}};
use super::Paragraph;



#[derive(Debug, Getters, Setters)]
pub struct ExtendedBlockQuoteParagraph {

    #[getset(get = "pub", set = "pub")]
    content: ContentBundle,
    
    #[getset(get = "pub", set = "pub")]
    extended_quote_type: String,

    #[getset(set = "pub")]
    nuid: Option<NmdUniqueIdentifier>,

    #[getset(set = "pub")]
    raw_content: String,
}

impl ExtendedBlockQuoteParagraph {
    
    pub fn new(raw_content: String, extended_quote_type: String, content: ContentBundle,) -> Self {
        Self {
            raw_content,
            content,
            extended_quote_type,
            nuid: None,
        }
    }

    fn html_standard_compile(&mut self, codex: &Codex, compilation_configuration: &CompilationConfiguration, compilation_configuration_overlay: CompilationConfigurationOverLay) -> Result<CompilationOutcome, CompilationError> {
        
        let mut outcome = format!(r#"<div class="focus-quote-block focus-quote-block-{}"{}>"#, self.extended_quote_type, text_utility::html_nuid_tag_or_nothing(self.nuid.as_ref()));
        outcome.push_str(&format!(r#"<div class="focus-quote-block-title focus-quote-block-{}-title"></div>"#, self.extended_quote_type));
        outcome.push_str(&format!(r#"<div class="focus-quote-block-description focus-quote-block-{}-description">"#, self.extended_quote_type));

        outcome.push_str(&self.content.compile(&OutputFormat::Html, codex, compilation_configuration, compilation_configuration_overlay.clone())?.content());

        outcome.push_str("</div></div>");

        Ok(CompilationOutcome::from(outcome))
    }
}

impl Compilable for ExtendedBlockQuoteParagraph {
    fn standard_compile(&mut self, format: &OutputFormat, codex: &Codex, compilation_configuration: &CompilationConfiguration, compilation_configuration_overlay: CompilationConfigurationOverLay) -> Result<CompilationOutcome, CompilationError> {
        
        match format {
            OutputFormat::Html => self.html_standard_compile(codex, compilation_configuration, compilation_configuration_overlay),
        }
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

    use crate::{codex::Codex, compilation::compilation_configuration::{compilation_configuration_overlay::CompilationConfigurationOverLay, CompilationConfiguration}, dossier::document::chapter::paragraph::paragraph_loading_rule::{block_quote_paragraph_loading_rule::BlockQuoteParagraphLoadingRule, ParagraphLoadingRule}, load::{LoadConfiguration, LoadConfigurationOverLay}, output_format::OutputFormat};

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

        let mut paragraph = rule.load(&nmd_text, &codex, &LoadConfiguration::default(), LoadConfigurationOverLay::default()).unwrap();    
    
        let mut cco = CompilationConfigurationOverLay::default();

        cco.set_document_name(Some(String::from("test")));

        paragraph.compile(&OutputFormat::Html, &codex, &CompilationConfiguration::default(), cco).unwrap();
    }

}


