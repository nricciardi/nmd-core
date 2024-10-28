use getset::{Getters, Setters};
use crate::{codex::Codex, compilation::{compilable::Compilable, compilation_configuration::{compilation_configuration_overlay::CompilationConfigurationOverLay, CompilationConfiguration}, compilation_error::CompilationError, compilation_outcome::CompilationOutcome}, content_bundle::ContentBundle, output_format::OutputFormat, utility::{nmd_unique_identifier::NmdUniqueIdentifier, text_utility}};
use super::Paragraph;



#[derive(Debug, Getters, Setters)]
pub struct FocusBlockParagraph {

    #[getset(get = "pub", set = "pub")]
    content: ContentBundle,
    
    #[getset(get = "pub", set = "pub")]
    extended_quote_type: String,

    #[getset(set = "pub")]
    nuid: Option<NmdUniqueIdentifier>,

    #[getset(set = "pub")]
    raw_content: String,
}

impl FocusBlockParagraph {
    
    pub fn new(raw_content: String, extended_quote_type: String, content: ContentBundle,) -> Self {
        Self {
            raw_content,
            content,
            extended_quote_type,
            nuid: None,
        }
    }

    fn html_standard_compile(&mut self, codex: &Codex, compilation_configuration: &CompilationConfiguration, compilation_configuration_overlay: CompilationConfigurationOverLay) -> Result<CompilationOutcome, CompilationError> {

        let mut outcome = format!(r#"<div class="focus-block focus-block-{}" {}>"#, self.extended_quote_type, text_utility::html_nuid_tag_or_nothing(self.nuid.as_ref()));
        outcome.push_str(&format!(r#"<div class="focus-block-title focus-block-{}-title"></div>"#, self.extended_quote_type));
        outcome.push_str(&format!(r#"<div class="focus-block-description focus-block-{}-description">"#, self.extended_quote_type));

        outcome.push_str(&self.content.compile(&OutputFormat::Html, codex, compilation_configuration, compilation_configuration_overlay.clone())?.content());

        outcome.push_str("</div></div>");

        Ok(CompilationOutcome::from(outcome))
    }
}

impl Compilable for FocusBlockParagraph {
    fn standard_compile(&mut self, format: &OutputFormat, codex: &Codex, compilation_configuration: &CompilationConfiguration, compilation_configuration_overlay: CompilationConfigurationOverLay) -> Result<CompilationOutcome, CompilationError> {
        
        match format {
            OutputFormat::Html => self.html_standard_compile(codex, compilation_configuration, compilation_configuration_overlay),
        }
    }
}


impl Paragraph for FocusBlockParagraph {
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

    use crate::{codex::Codex, compilation::compilation_configuration::{compilation_configuration_overlay::CompilationConfigurationOverLay, CompilationConfiguration}, content_bundle::ContentBundle, load::{LoadConfiguration, LoadConfigurationOverLay}, load_block::LoadBlock, output_format::OutputFormat};

    fn load_and_compile_html(content: &str, expected_n: usize) -> String {
        
        let codex = Codex::of_html();
    
        let blocks = LoadBlock::load_from_str(content, &codex, &LoadConfiguration::default(), LoadConfigurationOverLay::default()).unwrap();

        let mut bundle = ContentBundle::from(blocks);

        assert_eq!(bundle.preamble().len(), expected_n);

        let mut compiled_content = String::new();

        let cc = CompilationConfiguration::default();
        let cco = CompilationConfigurationOverLay::default();

        for paragraph in bundle.preamble_mut() {

            let outcome = paragraph.compile(&OutputFormat::Html, &codex, &cc, cco.clone()).unwrap();
        
            compiled_content.push_str(outcome.content());
        }

        compiled_content
    }

    #[test]
    fn two_focus_block() {

        let nmd_text = concat!(
            "\n\n",
            "::: warning\n",
            "new warning\n\n",
            "multiline\n",
            ":::\n\n",
            "\n",
            "::: important\n",
            "new important\n\n",
            "multiline\n",
            ":::\n\n",
        );
        
        let compiled_content = load_and_compile_html(nmd_text, 2);

        assert_eq!(compiled_content, concat!(
            r#"<div class="focus-block focus-block-warning" ><div class="focus-block-title focus-block-warning-title"></div><div class="focus-block-description focus-block-warning-description"><p class="paragraph">new warning</p><p class="paragraph">multiline</p></div></div>"#,
            r#"<div class="focus-block focus-block-important" ><div class="focus-block-title focus-block-important-title"></div><div class="focus-block-description focus-block-important-description"><p class="paragraph">new important</p><p class="paragraph">multiline</p></div></div>"#,
        ));
    }

}


