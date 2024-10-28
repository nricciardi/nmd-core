use getset::{Getters, Setters};
use crate::{codex::Codex, compilation::{compilable::Compilable, compilation_configuration::{compilation_configuration_overlay::CompilationConfigurationOverLay, CompilationConfiguration}, compilation_error::CompilationError, compilation_outcome::CompilationOutcome}, content_bundle::ContentBundle, output_format::OutputFormat, resource::resource_reference::ResourceReference, utility::{nmd_unique_identifier::NmdUniqueIdentifier, text_utility}};
use super::Paragraph;



#[derive(Debug, Getters, Setters)]
pub struct MetadataWrapperParagraph {

    #[getset(set = "pub")]
    raw_content: String,

    #[getset(get = "pub", set = "pub")]
    content: ContentBundle,

    #[getset(set = "pub")]
    nuid: Option<NmdUniqueIdentifier>,

    #[getset(get = "pub", set = "pub")]
    raw_id: Option<String>,

    #[getset(get = "pub", set = "pub")]
    styles: Option<String>,

    #[getset(get = "pub", set = "pub")]
    classes: Option<String>,
}

impl MetadataWrapperParagraph {
    
    pub fn new(raw_content: String, content: ContentBundle, raw_id: Option<String>, styles: Option<String>, classes: Option<String>,) -> Self {
        Self {
            raw_content,
            content,
            raw_id,
            styles,
            classes,
            nuid: None,
        }
    }

    fn html_standard_compile(&mut self, codex: &Codex, compilation_configuration: &CompilationConfiguration, compilation_configuration_overlay: CompilationConfigurationOverLay) -> Result<CompilationOutcome, CompilationError> {
        
        let nuid_attr = text_utility::html_nuid_tag_or_nothing(self.nuid.as_ref());

        let id_attr;
        if let Some(ref id) = self.raw_id {
            id_attr = format!(
                r#"id="{}""#,
                ResourceReference::of_internal_from_without_sharp(id, compilation_configuration_overlay.document_name().as_ref())?.build()
            );

        } else {

            id_attr = String::new();
        }

        let mut outcome = format!(r#"<div class="{}" style="{}" {} {}>"#, self.classes.as_ref().unwrap_or(&String::new()), self.styles.as_ref().unwrap_or(&String::new()), nuid_attr, id_attr);

        outcome.push_str(&self.content.standard_compile(&OutputFormat::Html, codex, compilation_configuration, compilation_configuration_overlay.clone())?.content());

        outcome.push_str("</div>");

        Ok(CompilationOutcome::from(outcome))
    }
}

impl Compilable for MetadataWrapperParagraph {
    fn standard_compile(&mut self, format: &OutputFormat, codex: &Codex, compilation_configuration: &CompilationConfiguration, compilation_configuration_overlay: CompilationConfigurationOverLay) -> Result<CompilationOutcome, CompilationError> {
        
        match format {
            OutputFormat::Html => self.html_standard_compile(codex, compilation_configuration, compilation_configuration_overlay),
        }
    }
}



impl Paragraph for MetadataWrapperParagraph {
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
    
        paragraph.compile(&OutputFormat::Html, &codex, &CompilationConfiguration::default(), CompilationConfigurationOverLay::default()).unwrap();
    }

}


