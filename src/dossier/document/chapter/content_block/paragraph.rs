use getset::{Getters, Setters};
use crate::{codex::Codex, compilation::{compilable::Compilable, compilation_configuration::{compilation_configuration_overlay::CompilationConfigurationOverLay, CompilationConfiguration}, compilation_error::CompilationError, compilation_outcome::CompilationOutcome, compilation_rule::constants::ESCAPE_HTML}, output_format::OutputFormat, text::compilable_string::CompilableString, utility::{datastruct::nmd_unique_identifier::NmdUniqueIdentifier, text_utility}};

use super::ContentBlock;



#[derive(Debug, Getters, Setters)]
pub struct Paragraph {

    #[getset(set = "pub")]
    nuid: Option<NmdUniqueIdentifier>,

    #[getset(set = "pub")]
    raw_content: String,        // TODO: remove
}

impl Paragraph {
    
    pub fn new(raw_content: String) -> Self {
        Self {
            raw_content,
            nuid: None,
        }
    }

    fn html_standard_compile(&mut self, codex: &Codex, compilation_configuration: &CompilationConfiguration, compilation_configuration_overlay: CompilationConfigurationOverLay) -> Result<CompilationOutcome, CompilationError> {
        
        let mut outcome = format!(
            r#"<p class="paragraph"{}>"#,
            text_utility::html_nuid_tag_or_nothing(self.nuid.as_ref()),
        );

        let compiled_content = CompilableString::from(text_utility::replace(&self.raw_content, &ESCAPE_HTML)).compile(&OutputFormat::Html, codex, compilation_configuration, compilation_configuration_overlay)?;

        outcome.push_str(&compiled_content.content().trim().replace("\n", " ").replace("\r", "").replace("\t", ""));
        
        outcome.push_str("</p>");

        Ok(CompilationOutcome::from(outcome))
    }
}

impl Compilable for Paragraph {
    fn standard_compile(&mut self, format: &OutputFormat, codex: &Codex, compilation_configuration: &CompilationConfiguration, compilation_configuration_overlay: CompilationConfigurationOverLay) -> Result<CompilationOutcome, CompilationError> {
        
        match format {
            OutputFormat::Html => self.html_standard_compile(codex, compilation_configuration, compilation_configuration_overlay),
        }
    }
}


impl ContentBlock for Paragraph {

    fn nuid(&self) -> Option<&NmdUniqueIdentifier> {
        self.nuid.as_ref()
    }
    
    fn set_nuid(&mut self, nuid: Option<NmdUniqueIdentifier>) {
        self.nuid = nuid;
    }
}


#[cfg(test)]
mod test {

    use crate::{codex::Codex, compilation::compilation_configuration::{compilation_configuration_overlay::CompilationConfigurationOverLay, CompilationConfiguration}, load::load_configuration::LoadConfiguration, output_format::OutputFormat, text::content_block_loading_rule::paragraph_loading_rule::ParagraphLoadingRule};

    #[test]
    fn compile() {
        let nmd_text = concat!(
            "> p1a\n",
            "> p1b\n",
            ">\n",
            "> p2a\n"
        ).to_string();
        
        let codex = Codex::of_html();
        let rule = ParagraphLoadingRule::new();

        let mut paragraphs = rule.load(&nmd_text, &codex, LoadConfiguration::default()).unwrap();    
    
        for paragraph in &mut paragraphs {
            paragraph.compile(&OutputFormat::Html, &codex, &CompilationConfiguration::default(), CompilationConfigurationOverLay::default()).unwrap();
        }
    }

}


